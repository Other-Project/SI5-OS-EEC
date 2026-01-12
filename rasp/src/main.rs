use rppal::i2c::I2c;
use std::thread;
use std::time::Duration;
use std::io::{self, Write};
use anyhow::Result;

// Configuration
const I2C_SLAVE_ADDR: u16 = 0x32;
// Note: rppal détecte automatiquement le bus 1 sur les RPi récents

// Map des registres
const REG_STATUS: u8 = 0x00;
const REG_EVENTS: u8 = 0x01;
const REG_RFID: u8 = 0x02;

// Events flags
const EVENT_MOTION_DETECTED: u8 = 0x02;
const EVENT_RFID_READ: u8 = 0x04;

struct ArduinoI2C {
    bus: I2c,
}

impl ArduinoI2C {
    /// Initialise la connexion I2C
    fn new() -> Result<Self> {
        let mut bus = I2c::new()?;
        bus.set_slave_address(I2C_SLAVE_ADDR)?;
        Ok(Self { bus })
    }

    /// Écrit une valeur dans un registre
    fn write_register(&mut self, reg: u8, value: u8) -> bool {
        match self.bus.smbus_write_byte(reg, value) {
            Ok(_) => true,
            Err(e) => {
                eprintln!("Erreur écriture registre 0x{:02X}: {}", reg, e);
                false
            }
        }
    }

    /// Lit la valeur d'un registre
    fn read_register(&mut self, reg: u8) -> Option<u8> {
        match self.bus.smbus_read_byte(reg) {
            Ok(val) => Some(val),
            Err(e) => {
                eprintln!("Erreur lecture registre 0x{:02X}: {}", reg, e);
                None
            }
        }
    }

    /// Lit plusieurs registres consécutifs (Logique boucle comme ton Python)
    fn read_registers(&mut self, start_reg: u8, count: u8) -> Option<Vec<u8>> {
        let mut values = Vec::with_capacity(count as usize);
        for i in 0..count {
            match self.bus.smbus_read_byte(start_reg + i) {
                Ok(val) => values.push(val),
                Err(e) => {
                    eprintln!("Erreur lecture multiple: {}", e);
                    return None;
                }
            }
        }
        Some(values)
    }

    // === Fonctions de haut niveau ===

    fn set_alarm(&mut self, enabled: bool) -> bool {
        self.write_register(REG_STATUS, if enabled { 1 } else { 0 })
    }

    fn get_alarm_state(&mut self) -> Option<u8> {
        self.read_register(REG_STATUS)
    }

    fn is_motion_detected(&mut self) -> bool {
        if let Some(status) = self.read_register(REG_EVENTS) {
            (status & EVENT_MOTION_DETECTED) != 0
        } else {
            false
        }
    }

    fn get_rfid_tag(&mut self) -> Option<String> {
        let status = self.read_register(REG_EVENTS)?;
        
        if (status & EVENT_RFID_READ) == 0 {
            return None;
        }

        // Lecture des 6 bytes du RFID
        if let Some(values) = self.read_registers(REG_RFID, 6) {
            // Conversion en string hexadécimal (ex: "A1B2C3...")
            let hex_string: String = values.iter()
                .map(|b| format!("{:02X}", b))
                .collect();
            return Some(hex_string);
        }
        
        None
    }
}

fn monitor_sensors() -> Result<()> {
    println!("Surveillance des capteurs (Ctrl+C pour arrêter)...\n");

    // Initialisation
    let mut arduino = match ArduinoI2C::new() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Impossible d'initialiser le bus I2C: {}", e);
            return Ok(());
        }
    };

    // Activer l'alarme pour tester
    arduino.set_alarm(true);

    loop {
        // Lire tous les capteurs
        let motion = arduino.is_motion_detected();
        let alarm = arduino.get_alarm_state().unwrap_or(0); // 0 par défaut si erreur
        let rfid_tag = arduino.get_rfid_tag();

        // Formatage de l'état de l'alarme
        let alarm_str = match alarm {
            1 => "ON ",
            2 => "TRIGGERED ",
            _ => "OFF",
        };

        let motion_str = if motion { "OUI" } else { "NON" };
        let rfid_str = rfid_tag.unwrap_or_else(|| "Aucun".to_string());

        // Affichage avec retour chariot (\r) pour écraser la ligne
        print!(
            "\r[Alarme: {}] [Mouvement: {}] [RFID: {:<16}]",
            alarm_str, motion_str, rfid_str
        );
        
        // Force l'affichage immédiat (flush stdout)
        io::stdout().flush()?;

        thread::sleep(Duration::from_millis(200));
    }
}

fn main() {
    if let Err(e) = monitor_sensors() {
        eprintln!("Erreur critique: {}", e);
    }
}