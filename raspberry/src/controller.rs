use anyhow::Result;
use log::debug;
use log::info;

use crate::arduino::ArduinoI2C;
use crate::arduino_consts::Events;
use crate::arduino_consts::SecurityState;
use crate::lcd::GroveLcd;

// Config
const I2C_SLAVE_ADDR: u16 = 0x32;
pub const VALID_BADGES: &[&str] = &["01056DE7D658"];

pub struct AlarmController {
    arduino: ArduinoI2C,
    screen: GroveLcd,
    current_state: SecurityState,
    last_events: Events,
    last_rfid: Option<String>,
}

impl AlarmController {
    pub fn new() -> Result<Self> {
        let mut arduino = ArduinoI2C::new(I2C_SLAVE_ADDR)?;
        debug!("Initial registers: {:?}", arduino.get_all_registers()?);
        let current_state = arduino.get_system_state()?;

        let screen = GroveLcd::new()?;
        screen.clear()?;
        screen.set_cursor(0, 0)?;
        screen.print("Systeme Actif")?;
        screen.set_cursor(0, 1)?;
        screen.print("Initialisation..")?;

        Ok(Self {
            arduino,
            screen,
            current_state,
            last_events: Events::empty(),
            last_rfid: None,
        })
    }

    pub fn poll(&mut self) -> Result<()> {
        let old_state = self.current_state;
        self.current_state = self.arduino.get_system_state()?;
        let events = self.arduino.get_events()?;
        self.last_events = events.clone();
        let mut new_state = None;

        // Arming
        if events.contains(Events::BTN_PRESSED) && old_state == SecurityState::Disarmed {
            info!("🛑 Armed via Button");
            new_state = Some(SecurityState::Armed);
        }

        // RFID
        if events.contains(Events::RFID_READ) {
            let uid = self.arduino.read_rfid_uid()?;
            self.last_rfid = Some(uid.clone());
            if VALID_BADGES.contains(&uid.as_str()) {
                if old_state != SecurityState::Disarmed {
                    info!("🟢 Disarmed via Badge {}", uid);
                    new_state = Some(SecurityState::Disarmed);
                }
            } else {
                info!("⚠️ ACCESS DENIED: Unknown badge {}", uid);
            }
        }

        // Motion detection
        if old_state == SecurityState::Armed && events.contains(Events::MOTION_DETECTED) {
            info!("🚨 INTRUSION DETECTED! ALARM!");
            new_state = Some(SecurityState::Triggered);
        }

        if let Some(state) = new_state {
            if state != self.current_state {
                self.current_state = state;
                self.arduino.set_system_state(self.current_state)?;
            }
        }

        if old_state != self.current_state {
            self.screen.clear()?;
            self.screen.set_cursor(0, 0)?;
            self.screen
                .print(format!("{:?}", self.current_state).as_str())?;
        }
        
        Ok(())
    }

    pub fn last_rfid(&self) -> Option<&str> {
        self.last_rfid.as_deref()
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
