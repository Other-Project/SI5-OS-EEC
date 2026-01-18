use anyhow::Result;
use rppal::i2c::I2c;
use std::{sync::Mutex, thread, time::Duration};

use crate::arduino_consts::{Events, Register, SecurityState};
const READ_FLAG: u8 = 0x80;

pub struct ArduinoI2C {
    bus: Mutex<I2c>,
}

impl ArduinoI2C {
    pub fn new(i2c_slave_addr: u16) -> Result<Self> {
        let mut bus = I2c::new()?;
        bus.set_slave_address(i2c_slave_addr)?;
        Ok(Self {
            bus: Mutex::new(bus),
        })
    }

    fn write_register(&self, reg: Register, value: u8) -> Result<()> {
        self.write_registers(reg, &[value])
    }

    fn write_registers(&self, start_reg: Register, values: &[u8]) -> Result<()> {
        let checksum: u8 = values.iter().fold(0u16, |acc, &b| acc + b as u16) as u8;
        let mut buf = Vec::with_capacity(2 + values.len() + 1);
        buf.push(start_reg as u8 & !READ_FLAG);
        buf.push(checksum);
        buf.extend_from_slice(values);

        let mut bus = self.bus.lock().unwrap();
        let write_count = bus.write(&buf)?;
        if write_count != buf.len() {
            return Err(anyhow::anyhow!(
                "Expected to write {} bytes, but wrote {} bytes",
                buf.len(),
                write_count
            ));
        }
        Ok(())
    }

    fn read_register(&self, reg: Register) -> Result<u8> {
        self.read_registers(reg, 1).map(|vals| {
            vals.get(0)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No data"))
        })?
    }

    fn read_registers(&self, start_reg: Register, count: u8) -> Result<Vec<u8>> {
        let mut bus = self.bus.lock().unwrap();
        let write_count = bus.write(&[start_reg as u8 | READ_FLAG, count])?; // Request read
        if write_count != 2 {
            return Err(anyhow::anyhow!(
                "Expected to write 2 bytes, but wrote {} bytes",
                write_count
            ));
        }

        thread::sleep(Duration::from_micros(500)); // Wait for Arduino to prepare data

        let mut buf = vec![0u8; count as usize + 1]; // +1 for checksum
        let read_count = bus.read(&mut buf)?;

        if read_count != buf.len() {
            return Err(anyhow::anyhow!(
                "Expected to read {} bytes, but got {} bytes",
                buf.len(),
                read_count
            ));
        }

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

    pub fn get_system_state(&self) -> Result<SecurityState> {
        let val = self.read_register(Register::Status)?;
        SecurityState::try_from(val).map_err(anyhow::Error::msg)
    }

    pub fn set_system_state(&self, state: SecurityState) -> Result<()> {
        self.write_register(Register::Status, state as u8)
    }

    pub fn get_events(&self) -> Result<Events> {
        self.read_register(Register::Events)
            .map(Events::from_bits_truncate)
    }

    pub fn read_rfid_uid(&self) -> Result<String> {
        let values = self.read_registers(Register::Rfid, 6)?;
        let hex: String = values.iter().map(|b| format!("{:02X}", b)).collect();
        Ok(hex)
    }

    pub fn read_ultrasonic_distance(&self) -> Result<f32> {
        let val = self.read_register(Register::UltrasonicDistance)?;
        Ok(val as f32 / 2.0)
    }

    pub fn set_ultrasonic_distance(&self, distance: f32) -> Result<()> {
        self.write_register(Register::UltrasonicDistance, (distance * 2.0) as u8)
    }

    pub fn get_all_registers(&self) -> Result<Vec<u8>> {
        self.read_registers(Register::Status,  Register::UltrasonicDistance as u8 + 1)
    }
}
