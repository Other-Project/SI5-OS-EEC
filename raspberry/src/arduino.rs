use anyhow::Result;
use rppal::i2c::I2c;

use crate::arduino_consts::*;
use crate::I2C_SLAVE_ADDR;

pub struct ArduinoI2C {
    bus: I2c,
}

impl ArduinoI2C {
    pub fn new() -> Result<Self> {
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

    pub fn set_system_state(&mut self, state: SecurityState) {
        self.write_register(REG_STATUS, state as u8);
    }

    pub fn get_events(&mut self) -> (bool, bool, bool) {
        if let Some(status) = self.read_register(REG_EVENTS) {
            let btn = (status & EVENT_BTN_PRESSED) != 0;
            let motion = (status & EVENT_MOTION_DETECTED) != 0;
            let rfid = (status & EVENT_RFID_READ) != 0;
            return (btn, motion, rfid);
        }
        (false, false, false)
    }

    pub fn read_rfid_uid(&mut self) -> Option<String> {
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
