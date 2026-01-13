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
        let mut state_changed = false;
        let mut generated_msg: Option<String> = None;

        // armement
        if events.contains(Events::BTN_PRESSED) {
            match self.current_state {
                SecurityState::Disarmed => {
                    self.current_state = SecurityState::Armed;
                    generated_msg = Some("🛑 Armed via Button".to_string());
                }
                _ => {}
            }
            state_changed = true;
        }

        // RFID
        if events.contains(Events::RFID_READ) {
            if let Some(uid) = self.arduino.read_rfid_uid() {
                self.last_rfid = Some(uid.clone());
                if VALID_BADGES.contains(&uid.as_str()) {
                    if self.current_state != SecurityState::Disarmed {
                        self.current_state = SecurityState::Disarmed;
                        generated_msg = Some(format!("🟢 Disarmed via Badge {}", uid));
                        state_changed = true;
                    }
                } else {
                    generated_msg = Some(format!("⚠️ ACCESS DENIED: Unknown badge {}", uid));
                }
            }
        }

        // mouv detecté
        if self.current_state == SecurityState::Armed && events.contains(Events::MOTION_DETECTED) {
            self.current_state = SecurityState::Triggered;
            generated_msg = Some("🚨 INTRUSION DETECTED! ALARM!".to_string());
            state_changed = true;
        }

        if state_changed {
            self.arduino.set_system_state(self.current_state);
        }

        if let Some(m) = generated_msg {
            self.push_message(m);
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