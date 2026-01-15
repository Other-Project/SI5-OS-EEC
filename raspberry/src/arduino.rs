use anyhow::Result;
use rppal::i2c::I2c;
use std::sync::Mutex;

use crate::arduino_consts::{Events, Register, SecurityState};
const READ_FLAG: u8 = 0x80;

pub struct ArduinoI2C {
    bus: Mutex<I2c>,
}

impl ArduinoI2C {
    pub fn new(i2c_slave_addr: u16) -> Result<Self> {
        let mut bus = I2c::new()?;
        bus.set_slave_address(i2c_slave_addr)?;
        Ok(Self { bus: Mutex::new(bus) })
    }

    fn write_register(&mut self, reg: Register, value: u8) -> Result<()> {
        let buf = [reg as u8, value];
        let mut bus = self.bus.lock().unwrap();
        bus.write(&buf)?;
        Ok(())
    }

    fn read_register(&mut self, reg: Register) -> Result<u8> {
        self.read_registers(reg, 1).map(|vals| {
            vals.get(0)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No data"))
        })?
    }

    fn read_registers(&mut self, start_reg: Register, count: u8) -> Result<Vec<u8>> {
        let mut bus = self.bus.lock().unwrap();
        bus.write(&[start_reg as u8 | READ_FLAG, count])?; // Request read

        let mut buf = vec![0u8; count as usize + 1]; // +1 for checksum
        bus.read(&mut buf)?;

        let (values, checksum_byte) = buf.split_at(count as usize);
        let received_checksum = checksum_byte[0];
        let computed_checksum: u8 = values.iter().fold(0u16, |acc, &b| acc + b as u16) as u8;

        if computed_checksum != received_checksum {
            Err(anyhow::anyhow!(
                "Checksum mismatch for register {:?}: computed {}, received {}, values: {:?}",
                start_reg,
                computed_checksum,
                received_checksum,
                values
            ))
        } else {
            Ok(values.to_vec())
        }
    }

    pub fn set_system_state(&mut self, state: SecurityState) -> Result<()> {
        self.write_register(Register::Status, state as u8)
    }

    pub fn get_events(&mut self) -> Result<Events> {
        self.read_register(Register::Events)
            .map(Events::from_bits_truncate)
    }

    pub fn read_rfid_uid(&mut self) -> Result<String> {
        let values = self.read_registers(Register::Rfid, 6)?;
        let hex: String = values.iter().map(|b| format!("{:02X}", b)).collect();
        Ok(hex)
    }
}
