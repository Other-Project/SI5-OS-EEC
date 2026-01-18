use anyhow::Result;
use discord_webhook_lib::DiscordMessage;
use log::debug;
use log::error;
use log::info;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::arduino::ArduinoI2C;
use crate::arduino_consts::Events;
use crate::arduino_consts::SecurityState;
use crate::badges::BadgeManager;

// Config
const I2C_SLAVE_ADDR: u16 = 0x32;
const BADGE_DB_PATH: &str = "badges.db";
const RFID_TIMEOUT_SECS: u64 = 5; // Reset RFID after 5 seconds

pub struct AlarmController {
    arduino: ArduinoI2C,
    pub current_state: SecurityState,
    last_events: Events,
    last_rfid: Option<String>,
    last_rfid_time: u64,
    badge_manager: BadgeManager,
}

impl AlarmController {
    pub fn new() -> Result<Self> {
        let arduino = ArduinoI2C::new(I2C_SLAVE_ADDR)?;
        debug!("Initial registers: {:?}", arduino.get_all_registers()?);
        let current_state = arduino.get_system_state()?;

        let badge_manager = BadgeManager::new(BADGE_DB_PATH)?;
        badge_manager.cleanup_expired_badges()?;

        Ok(Self {
            arduino,
            current_state,
            last_events: Events::empty(),
            last_rfid: None,
            last_rfid_time: 0,
            badge_manager,
        })
    }

    pub fn poll(&mut self) -> Result<()> {
        let old_state = self.current_state;
        self.current_state = self.arduino.get_system_state()?;
        let events = self.arduino.get_events()?;
        self.last_events = events.clone();
        let mut new_state = None;

        // Check if RFID timeout has expired
        if let Some(_) = self.last_rfid {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if now - self.last_rfid_time > RFID_TIMEOUT_SECS {
                self.last_rfid = None;
            }
        }

        // Arming
        if events.contains(Events::BTN_PRESSED) && old_state == SecurityState::Disarmed {
            info!("🛑 Armed via Button");
            new_state = Some(SecurityState::Armed);
        }

        // RFID
        if events.contains(Events::RFID_READ) {
            let uid = self.arduino.read_rfid_uid()?;
            self.last_rfid = Some(uid.clone());
            self.last_rfid_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            // Check if badge is valid using database
            match self.badge_manager.is_valid_badge(&uid) {
                Ok(true) => {
                    if old_state != SecurityState::Disarmed {
                        if let Ok(Some(badge)) = self.badge_manager.get_badge(&uid) {
                            info!("🟢 Disarmed via Badge {}", badge.name);
                            self.badge_manager.update_last_used(&uid).ok();
                        }
                        new_state = Some(SecurityState::Disarmed);
                    }
                }
                _ => {
                    info!("⚠️ ACCESS DENIED: Unknown or disabled badge {}", uid);
                }
            }
        }

        // Motion detection
        if old_state == SecurityState::Armed && events.contains(Events::MOTION_DETECTED) {
            info!("🚨 INTRUSION DETECTED! ALARM!");
            new_state = Some(SecurityState::Triggered);

            let mut builder = DiscordMessage::builder("https://discord.com/api/webhooks/1462535169779957904/KMjxbYxz6YSAeqNAY5n3WjWbX0NNIEb75XGcv5y2UwcUvVlGfFNic8iRWLHI_ze62Y7r");
            builder.add_field("username", "Alarm System");
            builder.add_field("content", "🚨 INTRUSION DETECTED! ALARM!");
            let dhm = builder.build();
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = dhm.send().await {
                    error!("Failed to send Discord alert: {}", e);
                }
            });
        }

        if let Some(state) = new_state {
            if state != self.current_state {
                self.current_state = state;
                self.arduino.set_system_state(self.current_state)?;
            }
        }

        Ok(())
    }

    pub fn last_rfid(&self) -> Option<&str> {
        self.last_rfid.as_deref()
    }

    pub fn badge_manager(&self) -> &BadgeManager {
        &self.badge_manager
    }

    pub fn state(&self) -> SecurityState {
        self.current_state
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

    pub fn get_last_rfid(&self) -> Option<&str> {
        self.last_rfid.as_deref()
    }

    pub fn get_ultrasonic_distance(&self) -> Result<f32> {
        self.arduino.read_ultrasonic_distance()
    }

    pub fn set_ultrasonic_distance(&self, distance: f32) -> Result<()> {
        self.arduino.set_ultrasonic_distance(distance)
    }
}
