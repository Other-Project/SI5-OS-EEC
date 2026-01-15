use anyhow::Result;
use rppal::i2c::I2c;

use crate::arduino_consts::{Events, Register, SecurityState};
const READ_FLAG: u8 = 0x80;

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
        let buf = [reg as u8, value];
        self.bus.write(&buf).is_ok()
    }

    fn read_register(&mut self, reg: Register) -> Option<u8> {
        self.read_registers(reg, 1).and_then(|vals| vals.get(0).cloned())
    }

   fn read_registers(&mut self, start_reg: Register, count: u8) -> Option<Vec<u8>> {
    self.bus.write(&[start_reg as u8 | READ_FLAG, count]).or_else(|e| { println!("Err: {}", e); Err(e) }).ok()?; // Request read

    let mut buf = vec![0u8; count as usize + 1]; // +1 for checksum
    self.bus.read(&mut buf).ok()?;

    let (values, checksum_byte) = buf.split_at(count as usize);
    let received_checksum = checksum_byte[0];
    let computed_checksum: u8 = values.iter().fold(0u16, |acc, &b| acc + b as u16) as u8;

    if computed_checksum != received_checksum {
        None
    } else {
        Some(values.to_vec())
    }
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
