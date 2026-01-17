use rppal::i2c::I2c;
use std::sync::Mutex;
use anyhow::Result;
use std::thread;
use std::time::{Duration};

#[allow(unused)]
pub enum GrovePiCommand {
    DigitalRead = 1,
    DigitalWrite = 2,
    AnalogRead = 3,
    AnalogWrite = 4,
    PinMode = 5,
}

#[allow(unused)]
pub enum GrovePiPinMode {
    Input = 0,
    Output = 1,
}

pub struct GrovePi {
    bus: Mutex<I2c>,
}

#[allow(unused)]
impl GrovePi {
    pub fn new(addr: u16) -> Result<Self> {
        let mut bus = I2c::new()?;
        bus.set_slave_address(addr)?;
        Ok(Self {
            bus: Mutex::new(bus),
        })
    }

    pub fn pin_mode(&self, pin: u8, mode: GrovePiPinMode) -> Result<()> {
        let mut bus = self.bus.lock().unwrap();
        bus.write(&[GrovePiCommand::PinMode as u8, pin, mode as u8, 0])?;
        thread::sleep(Duration::from_millis(50));
        Ok(())
    }

    pub fn analog_read(&self, pin: u8) -> Result<u16> {
        let mut bus = self.bus.lock().unwrap();
        bus.write(&[GrovePiCommand::AnalogRead as u8, pin, 0, 0])?;
        thread::sleep(Duration::from_millis(10));
        let mut buf = [0u8; 3];
        bus.read(&mut buf)?;
        Ok(((buf[1] as u16) << 8) | buf[2] as u16)
    }

    pub fn digital_read(&self, pin: u8) -> Result<bool> {
        let mut bus = self.bus.lock().unwrap();
        bus.write(&[GrovePiCommand::DigitalRead as u8, pin, 0, 0])?;
        thread::sleep(Duration::from_millis(10));
        let mut buf = [0u8; 1];
        bus.read(&mut buf)?;
        Ok(buf[0] > 0 && buf[0] != 255)
    }
}