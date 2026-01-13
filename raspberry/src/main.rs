use anyhow::Result;
use rppal::i2c::I2c;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

// Config
const I2C_SLAVE_ADDR: u16 = 0x32;

const REG_STATUS: u8 = 0x00;
const REG_EVENTS: u8 = 0x01;
const REG_RFID: u8 = 0x02;

const EVENT_BTN_PRESSED: u8 = 0x01;
const EVENT_MOTION_DETECTED: u8 = 0x02;
const EVENT_RFID_READ: u8 = 0x04;

// Liste des badges autorisés
const VALID_BADGES: &[&str] = &["01056DE7D658"];

#[derive(Debug, PartialEq, Clone, Copy)]
enum SecurityState {
    Disarmed = 0,  // Veille
    Armed = 1,     // Surveillance
    Triggered = 2, // ALERTE (Sonnerie)
}

struct ArduinoI2C {
    bus: I2c,
}

impl ArduinoI2C {
    fn new() -> Result<Self> {
        let mut bus = I2c::new()?;
        bus.set_slave_address(I2C_SLAVE_ADDR)?;
        Ok(Self { bus })
    }

    fn write_register(&mut self, reg: u8, value: u8) -> bool {
        self.bus.smbus_write_byte(reg, value).is_ok()
    }

    fn read_register(&mut self, reg: u8) -> Option<u8> {
        self.bus.smbus_read_byte(reg).ok()
    }

    fn read_registers(&mut self, start_reg: u8, count: u8) -> Option<Vec<u8>> {
        let mut values = Vec::with_capacity(count as usize);
        for i in 0..count {
            match self.bus.smbus_read_byte(start_reg + i) {
                Ok(val) => values.push(val),
                Err(_) => return None,
            }
        }
        Some(values)
    }

    fn set_system_state(&mut self, state: SecurityState) {
        self.write_register(REG_STATUS, state as u8);
    }

    fn get_events(&mut self) -> (bool, bool, bool) {
        if let Some(status) = self.read_register(REG_EVENTS) {
            let btn = (status & EVENT_BTN_PRESSED) != 0;
            let motion = (status & EVENT_MOTION_DETECTED) != 0;
            let rfid = (status & EVENT_RFID_READ) != 0;
            return (btn, motion, rfid);
        }
        (false, false, false)
    }

    fn read_rfid_uid(&mut self) -> Option<String> {
        if let Some(values) = self.read_registers(REG_RFID, 6) {
            let hex: String = values.iter().map(|b| format!("{:02X}", b)).collect();
            if hex == "000000000000" {
                return None;
            }
            return Some(hex);
        }
        None
    }
}

fn main() -> Result<()> {
    println!("--- Contrôleur d'Alarme Rust ---");
    println!("Badges valides: {:?}", VALID_BADGES);

    let mut arduino = ArduinoI2C::new()?;
    
    // init
    let mut current_state = SecurityState::Disarmed;
    arduino.set_system_state(current_state);

    loop {
        let (btn_pressed, motion_detected, rfid_ready) = arduino.get_events();
        
        let mut state_changed = false;

        // armement
        if btn_pressed {
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
        if rfid_ready {
            if let Some(uid) = arduino.read_rfid_uid() {
                if VALID_BADGES.contains(&uid.as_str()) {
                    // badge valide
                    match current_state {
                        SecurityState::Disarmed => {
                            current_state = SecurityState::Armed;
                            println!("🛑 Armement via Badge {}", uid);
                        }
                        _ => {
                            // Si Armé ou Triggered -> On désarme
                            current_state = SecurityState::Disarmed;
                            println!("🟢 Désarmement via Badge {}", uid);
                        }
                    }
                    state_changed = true;
                } else {
                    println!("⚠️ ACCÈS REFUSÉ : Badge inconnu {}", uid);
                }
            }
        }

        // mouv detecté
        if current_state == SecurityState::Armed && motion_detected {
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
            if motion_detected { "OUI" } else { "NON" },
            if btn_pressed { "APPUI" } else { "RELÂCHÉ" }
        );
        io::stdout().flush()?;

        // pause pour ne pas saturer le bus I2C
        thread::sleep(Duration::from_millis(100));
    }
}
