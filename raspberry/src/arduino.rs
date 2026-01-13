use anyhow::Result;
use rppal::i2c::I2c;

use crate::arduino_consts::{Events, Register, SecurityState};

pub struct ArduinoI2C {
    bus: I2c,
}

impl ArduinoI2C {
    pub fn new(i2c_slave_addr: u16) -> Result<Self> {
        let mut bus = I2c::new()?;
        bus.set_slave_address(i2c_slave_addr)?;
        Ok(Self { bus })
    }

    fn write_register(&mut self, reg: Register, value: u8) -> bool {
        self.bus.smbus_write_byte(reg as u8, value).is_ok()
    }

    fn read_register(&mut self, reg: Register) -> Option<u8> {
        self.bus.smbus_read_byte(reg as u8).ok()
    }

    fn read_registers(&mut self, start_reg: Register, count: u8) -> Option<Vec<u8>> {
        let start = start_reg as u8;
        let mut values = Vec::with_capacity(count as usize);
        for i in 0..count {
            match self.bus.smbus_read_byte(start + i) {
                Ok(val) => values.push(val),
                Err(_) => return None,
            }
        }
        Some(values)
    }

    pub fn set_system_state(&mut self, state: SecurityState) {
        self.write_register(Register::Status, state as u8);
    }

    pub fn get_events(&mut self) -> Events {
        self.read_register(Register::Events)
            .map(Events::from_bits_truncate)
            .unwrap_or_else(Events::empty)
    }

    pub fn read_rfid_uid(&mut self) -> Option<String> {
        if let Some(values) = self.read_registers(Register::Rfid, 6) {
            let hex: String = values.iter().map(|b| format!("{:02X}", b)).collect();
            if hex == "000000000000" {
                return None;
            }
            return Some(hex);
        }
        None
    }
}
