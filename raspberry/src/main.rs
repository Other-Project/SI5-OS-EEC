use anyhow::Result;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

mod arduino;
mod arduino_consts;

use arduino::ArduinoI2C;
use arduino_consts::SecurityState;

use crate::arduino_consts::Events;

// Config
const I2C_SLAVE_ADDR: u16 = 0x32;
const VALID_BADGES: &[&str] = &["01056DE7D658"];

fn main() -> Result<()> {
    println!("--- Contrôleur d'Alarme Rust ---");
    println!("Badges valides: {:?}", VALID_BADGES);

    let mut arduino = ArduinoI2C::new(I2C_SLAVE_ADDR)?;

    // init
    let mut current_state = SecurityState::Disarmed;
    arduino.set_system_state(current_state);

    loop {
        let events = arduino.get_events();

        let mut state_changed = false;

        // armement
        if events.contains(Events::BTN_PRESSED) {
            match current_state {
                SecurityState::Disarmed => {
                    current_state = SecurityState::Armed;
                    println!("🛑 Armement via Bouton");
                }
                _ => {}
            }
            state_changed = true;
        }

        // RFID
        if events.contains(Events::RFID_READ) {
            if let Some(uid) = arduino.read_rfid_uid() {
                if VALID_BADGES.contains(&uid.as_str()) {
                    if current_state != SecurityState::Disarmed {
                        // Si Armé ou Triggered -> On désarme
                        current_state = SecurityState::Disarmed;
                        println!("🟢 Désarmement via Badge {}", uid);
                        state_changed = true;
                    }
                } else {
                    println!("⚠️ ACCÈS REFUSÉ : Badge inconnu {}", uid);
                }
            }
        }

        // mouv detecté
        if current_state == SecurityState::Armed && events.contains(Events::MOTION_DETECTED) {
            current_state = SecurityState::Triggered;
            println!("🚨 INTRUSION DÉTECTÉE ! ALARME !");
            state_changed = true;
        }

        if state_changed {
            arduino.set_system_state(current_state);
        }

        let state_icon = match current_state {
            SecurityState::Disarmed => "🟢 VEILLE",
            SecurityState::Armed => "🛑 ARMÉE",
            SecurityState::Triggered => "🚨 SONNERIE",
        };

        println!(
            "[État: {:<10}] [Mouv: {}] [Btn: {}]",
            state_icon,
            if events.contains(Events::MOTION_DETECTED) {
                "OUI"
            } else {
                "NON"
            },
            if events.contains(Events::BTN_PRESSED) {
                "APPUI"
            } else {
                "RELÂCHÉ"
            }
        );
        io::stdout().flush()?;

        // pause pour ne pas saturer le bus I2C
        thread::sleep(Duration::from_millis(100));
    }
}
