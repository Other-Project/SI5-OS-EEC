use anyhow::Result;
use rppal::i2c::I2c;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

const LCD_ADDR: u16 = 0x3E; // 0x7c >> 1

// LCD internal registers (Seeed protocol)
const LCD_REG_COMMAND: u8 = 0x80;
const LCD_REG_DATA: u8 = 0x40;

// LCD Commands
const LCD_CLEARDISPLAY: u8 = 0x01;
const LCD_ENTRYMODESET: u8 = 0x04;
const LCD_DISPLAYCONTROL: u8 = 0x08;
const LCD_FUNCTIONSET: u8 = 0x20;

// Flags
const LCD_DISPLAYON: u8 = 0x04;
const LCD_CURSOROFF: u8 = 0x00;
const LCD_BLINKOFF: u8 = 0x00;
const LCD_ENTRYLEFT: u8 = 0x02;
const LCD_ENTRYSHIFTDECREMENT: u8 = 0x00;
const LCD_2LINE: u8 = 0x08;
const LCD_5X8DOTS: u8 = 0x00;

pub struct GroveLcd {
    bus: Mutex<I2c>,
}

impl GroveLcd {
    pub fn new() -> Result<Self> {
        let mut bus = I2c::new()?;
        bus.set_slave_address(LCD_ADDR)?;
        let mut lcd = Self {
            bus: Mutex::new(bus),
        };
        lcd.initialize()?;
        Ok(lcd)
    }

    fn initialize(&mut self) -> Result<()> {
        // Wait for LCD to power up
        thread::sleep(Duration::from_millis(50));
        // Specific initialization sequence
        let display_function = LCD_2LINE | LCD_5X8DOTS;

        // Send Function Set command
        self.send_command(LCD_FUNCTIONSET | display_function)?;
        thread::sleep(Duration::from_micros(4500)); // > 4.1ms
        self.send_command(LCD_FUNCTIONSET | display_function)?;
        thread::sleep(Duration::from_micros(150));
        self.send_command(LCD_FUNCTIONSET | display_function)?;
        self.send_command(LCD_FUNCTIONSET | display_function)?;
        // Turn on display, no cursor, no blinking
        let display_control = LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF;
        self.send_command(LCD_DISPLAYCONTROL | display_control)?;
        // Clear display
        self.clear()?;
        // Entry mode (text left to right)
        let display_mode = LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT;
        self.send_command(LCD_ENTRYMODESET | display_mode)?;
        Ok(())
    }

    /// Centralized I2C write helper
    fn write_register(&self, reg: u8, val: u8) -> Result<()> {
        self.bus.lock().unwrap().smbus_write_byte(reg, val)?;
        Ok(())
    }

    /// Send a command (Register 0x80)
    fn send_command(&self, cmd: u8) -> Result<()> {
        self.write_register(LCD_REG_COMMAND, cmd)
    }

    /// Send data/character (Register 0x40)
    fn send_data(&self, data: u8) -> Result<()> {
        self.write_register(LCD_REG_DATA, data)
    }

    /// Clear the display
    pub fn clear(&self) -> Result<()> {
        self.send_command(LCD_CLEARDISPLAY)?;
        thread::sleep(Duration::from_millis(2)); // This command is slow
        Ok(())
    }

    /// Set cursor position (column 0-15, row 0-1)
    pub fn set_cursor(&self, col: u8, row: u8) -> Result<()> {
        let addr = if row == 0 {
            0x80u8.wrapping_add(col)
        } else {
            0xC0u8.wrapping_add(col)
        };
        self.send_command(addr)?;
        Ok(())
    }

    /// Print a string to the display
    pub fn print(&self, text: &str) -> Result<()> {
        for c in text.bytes() {
            self.send_data(c)?;
        }
        Ok(())
    }
}

impl Drop for GroveLcd {
    fn drop(&mut self) {
        let _ = self.clear();
    }
}
