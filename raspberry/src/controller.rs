use anyhow::Result;

use crate::arduino::ArduinoI2C;
use crate::arduino_consts::SecurityState;
use crate::arduino_consts::Events;

// Config
const I2C_SLAVE_ADDR: u16 = 0x32;
pub const VALID_BADGES: &[&str] = &["01056DE7D658"];

pub struct AlarmController {
    arduino: ArduinoI2C,
    current_state: SecurityState,
    last_events: Events,
    last_rfid: Option<String>,
    last_messages: Vec<String>,
}

impl AlarmController {
    pub fn new() -> Result<Self> {
        let mut arduino = ArduinoI2C::new(I2C_SLAVE_ADDR)?;
        let current_state = SecurityState::Disarmed;
        arduino.set_system_state(current_state);
        Ok(Self {
            arduino,
            current_state,
            last_events: Events::empty(),
            last_rfid: None,
            last_messages: Vec::new(),
        })
    }

    fn push_message(&mut self, msg: String) {
        const MAX: usize = 32;
        if !msg.is_empty() {
            self.last_messages.insert(0, msg);
            if self.last_messages.len() > MAX {
                self.last_messages.truncate(MAX);
            }
        }
    }

    pub fn poll(&mut self) -> Result<()> {
        let events = self.arduino.get_events();
        self.last_events = events.clone();
        let mut new_state = None;

        // Arming
        if events.contains(Events::BTN_PRESSED) {
            match self.current_state {
                SecurityState::Disarmed => {
                    self.push_message("🛑 Armed via Button".to_string());
                    new_state = Some(SecurityState::Armed);
                }
                _ => {}
            }
        }

        // RFID
        if events.contains(Events::RFID_READ) {
            if let Some(uid) = self.arduino.read_rfid_uid() {
                self.last_rfid = Some(uid.clone());
                if VALID_BADGES.contains(&uid.as_str()) {
                    if self.current_state != SecurityState::Disarmed {
                        self.push_message(format!("🟢 Disarmed via Badge {}", uid));
                        new_state = Some(SecurityState::Disarmed);
                    }
                } else {
                    self.push_message(format!("⚠️ ACCESS DENIED: Unknown badge {}", uid));
                }
            }
        }

        // Motion detection
        if self.current_state == SecurityState::Armed && events.contains(Events::MOTION_DETECTED) {
            self.push_message("🚨 INTRUSION DETECTED! ALARM!".to_string());
            new_state = Some(SecurityState::Triggered);
        }

        if let Some(state) = new_state {
            self.current_state = state;
            self.arduino.set_system_state(self.current_state);
        }
        Ok(())
    }

    pub fn last_rfid(&self) -> Option<&str> {
        self.last_rfid.as_deref()
    }

    pub fn last_messages(&self) -> &[String] {
        &self.last_messages
    }

    pub fn state_icon(&self) -> &'static str {
        match self.current_state {
            SecurityState::Disarmed => "🟢 DISARMED",
            SecurityState::Armed => "🛑 ARMED",
            SecurityState::Triggered => "🚨 ALARM",
        }
    }

    pub fn motion_str(&self) -> &'static str {
        if self.last_events.contains(Events::MOTION_DETECTED) {
            "YES"
        } else {
            "NO"
        }
    }

    pub fn btn_str(&self) -> &'static str {
        if self.last_events.contains(Events::BTN_PRESSED) {
            "PRESSED"
        } else {
            "RELEASED"
        }
    }
}