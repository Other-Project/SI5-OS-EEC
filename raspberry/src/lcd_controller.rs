use anyhow::Result;
use log::info;
use rppal::i2c::I2c;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use crate::arduino_consts::SecurityState;
use crate::controller::AlarmController;
use crate::lcd::GroveLcd;


const GROVE_PI_ADDR: u16 = 0x04;
const ROTARY_PIN: u8 = 0; // A0
const BUTTON_PIN: u8 = 1; // A1

const MENU_TIMEOUT_SECS: u64 = 10;

#[derive(Debug, Clone, Copy, PartialEq)]
enum LcdState {
    Default,
    Menu,
    AddCard,
    RemoveCard,
}

#[derive(Debug)]
enum MenuOption {
    AddCard,
    RemoveCard,
}

impl MenuOption {
    fn as_str(&self) -> &str {
        match self {
            MenuOption::AddCard => "Add Card",
            MenuOption::RemoveCard => "Remove Card",
        }
    }
}

pub enum GrovePiCommand {
    DigitalRead = 1,
    DigitalWrite = 2,
    AnalogRead = 3,
    AnalogWrite = 4,
    PinMode = 5,
}

pub enum GrovePiPinMode {
    Input = 0,
    Output = 1,
}

pub struct GrovePi {
    bus: Mutex<I2c>,
}

impl GrovePi {
    pub fn new() -> Result<Self> {
        let mut bus = I2c::new()?;
        bus.set_slave_address(GROVE_PI_ADDR)?;
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

pub struct LcdController {
    lcd: GroveLcd,
    grove_pi: GrovePi,
    state: LcdState,
    menu_options: Vec<MenuOption>,
    selected_option: usize,
    last_activity: Instant,
    controller: std::rc::Rc<std::cell::RefCell<AlarmController>>,
    current_text: [Option<String>; 2],
    button_was_pressed: bool,
}

impl LcdController {
    pub fn new(controller: std::rc::Rc<std::cell::RefCell<AlarmController>>) -> Result<Self> {
        let lcd = GroveLcd::new()?;
        let grove_pi = GrovePi::new()?;
        grove_pi.pin_mode(BUTTON_PIN, GrovePiPinMode::Input)?;

        Ok(Self {
            lcd,
            grove_pi,
            state: LcdState::Default,
            menu_options: vec![MenuOption::AddCard, MenuOption::RemoveCard],
            selected_option: 0,
            last_activity: Instant::now(),
            controller,
            current_text: [None, None],
            button_was_pressed: false,
        })
    }

    pub fn poll(&mut self) -> Result<()> {
        match self.state {
            LcdState::Default => {
                self.update_default_screen()?;
                if self.is_button_pressed()? {
                    info!("Button pressed, entering menu");
                    self.enter_menu()?;
                }
            }
            LcdState::Menu => {
                self.update_menu_screen()?;
                if self.is_button_pressed()? {
                    self.select_menu_option()?;
                } else {
                    self.update_menu_selection()?;
                }
                if self.last_activity.elapsed() > Duration::from_secs(MENU_TIMEOUT_SECS) {
                    self.exit_menu()?;
                }
            }
            LcdState::AddCard => {
                self.update_add_card_screen()?;
                // Wait for RFID
                let uid = {
                    let controller = self.controller.borrow();
                    controller.last_rfid().map(|s| s.to_string())
                };
                if let Some(uid) = uid {
                    self.add_card(&uid)?;
                    self.exit_menu()?;
                }
                if self.last_activity.elapsed() > Duration::from_secs(MENU_TIMEOUT_SECS) {
                    self.exit_menu()?;
                }
            }
            LcdState::RemoveCard => {
                self.update_remove_card_screen()?;
                let uid = {
                    let controller = self.controller.borrow();
                    controller.last_rfid().map(|s| s.to_string())
                };
                if let Some(uid) = uid {
                    self.remove_card(&uid)?;
                    self.exit_menu()?;
                }
                if self.last_activity.elapsed() > Duration::from_secs(MENU_TIMEOUT_SECS) {
                    self.exit_menu()?;
                }
            }
        }
        Ok(())
    }

    fn is_button_pressed(&mut self) -> Result<bool> {
        let pressed = self.grove_pi.analog_read(BUTTON_PIN).map(|v| v > 512)?;
        let was_pressed = self.button_was_pressed;
        self.button_was_pressed = pressed;
        Ok(pressed && !was_pressed) // Trigger on press edge
    }

    fn get_rotary_value(&self) -> Result<f32> {
        let val = self.grove_pi.analog_read(ROTARY_PIN)?;
        Ok(val as f32 / 1023.0) // 0.0 to 1.0
    }

    fn update_default_screen(&mut self) -> Result<()> {
        let controller = self.controller.borrow();
        let line1 = format!("{:?}", controller.current_state);
        let line2 = match controller.current_state {
            SecurityState::Armed | SecurityState::Triggered => "Scan to disarm".to_string(),
            SecurityState::Disarmed => "Use btn for menu".to_string(),
        };
        if self.current_text[0].as_ref() != Some(&line1)
            || self.current_text[1].as_ref() != Some(&line2)
        {
            self.lcd.clear()?;
            self.lcd.set_cursor(0, 0)?;
            self.lcd.print(&line1)?;
            self.lcd.set_cursor(0, 1)?;
            self.lcd.print(&line2)?;
            self.current_text = [Some(line1), Some(line2)];
        }
        Ok(())
    }

    fn enter_menu(&mut self) -> Result<()> {
        self.state = LcdState::Menu;
        self.selected_option = 0;
        self.last_activity = Instant::now();
        self.update_menu_screen()?;
        Ok(())
    }

    fn exit_menu(&mut self) -> Result<()> {
        self.state = LcdState::Default;
        self.update_default_screen()?;
        Ok(())
    }

    fn update_menu_screen(&mut self) -> Result<()> {
        let line1 = "Menu:".to_string();
        let line2 = self.menu_options[self.selected_option].as_str().to_string();
        if self.current_text[0].as_ref() != Some(&line1)
            || self.current_text[1].as_ref() != Some(&line2)
        {
            self.lcd.clear()?;
            self.lcd.set_cursor(0, 0)?;
            self.lcd.print(&line1)?;
            self.lcd.set_cursor(0, 1)?;
            self.lcd.print(&line2)?;
            self.current_text = [Some(line1), Some(line2)];
        }
        Ok(())
    }

    fn update_menu_selection(&mut self) -> Result<()> {
        let rotary = self.get_rotary_value()?;
        let new_selection = (rotary * self.menu_options.len() as f32) as usize;
        if new_selection != self.selected_option {
            self.selected_option = new_selection.min(self.menu_options.len() - 1);
            self.last_activity = Instant::now();
            self.update_menu_screen()?;
        }
        Ok(())
    }

    fn select_menu_option(&mut self) -> Result<()> {
        match self.menu_options[self.selected_option] {
            MenuOption::AddCard => {
                self.state = LcdState::AddCard;
                self.last_activity = Instant::now();
                self.update_add_card_screen()?;
            }
            MenuOption::RemoveCard => {
                self.state = LcdState::RemoveCard;
                self.last_activity = Instant::now();
                self.update_remove_card_screen()?;
            }
        }
        Ok(())
    }

    fn update_add_card_screen(&mut self) -> Result<()> {
        let line1 = "Add Card".to_string();
        let line2 = "Scan RFID card".to_string();
        if self.current_text[0].as_ref() != Some(&line1)
            || self.current_text[1].as_ref() != Some(&line2)
        {
            self.lcd.clear()?;
            self.lcd.set_cursor(0, 0)?;
            self.lcd.print(&line1)?;
            self.lcd.set_cursor(0, 1)?;
            self.lcd.print(&line2)?;
            self.current_text = [Some(line1), Some(line2)];
        }
        Ok(())
    }

    fn add_card(&self, uid: &str) -> Result<()> {
        let controller = self.controller.borrow_mut();
        let badge_manager = controller.badge_manager();
        if badge_manager.is_valid_badge(uid).unwrap_or(false) {
            // Already exists
            self.lcd.clear()?;
            self.lcd.set_cursor(0, 0)?;
            self.lcd.print("Card already")?;
            self.lcd.set_cursor(0, 1)?;
            self.lcd.print("exists")?;
            thread::sleep(Duration::from_secs(2));
        } else {
            // Add with default name
            badge_manager.add_badge_with_expiry(uid, &format!("Card {}", uid), None)?;
            self.lcd.clear()?;
            self.lcd.set_cursor(0, 0)?;
            self.lcd.print("Card added")?;
            thread::sleep(Duration::from_secs(2));
        }
        Ok(())
    }

    fn update_remove_card_screen(&mut self) -> Result<()> {
        let line1 = "Remove Card".to_string();
        let line2 = "Scan RFID card".to_string();
        if self.current_text[0].as_ref() != Some(&line1)
            || self.current_text[1].as_ref() != Some(&line2)
        {
            self.lcd.clear()?;
            self.lcd.set_cursor(0, 0)?;
            self.lcd.print(&line1)?;
            self.lcd.set_cursor(0, 1)?;
            self.lcd.print(&line2)?;
            self.current_text = [Some(line1), Some(line2)];
        }
        Ok(())
    }

    fn remove_card(&self, uid: &str) -> Result<()> {
        let controller = self.controller.borrow_mut();
        let badge_manager = controller.badge_manager();
        if badge_manager.is_valid_badge(uid).unwrap_or(false) {
            badge_manager.remove_badge(uid)?;
            self.lcd.clear()?;
            self.lcd.set_cursor(0, 0)?;
            self.lcd.print("Card removed")?;
            thread::sleep(Duration::from_secs(2));
        } else {
            self.lcd.clear()?;
            self.lcd.set_cursor(0, 0)?;
            self.lcd.print("Card not found")?;
            thread::sleep(Duration::from_secs(2));
        }
        Ok(())
    }
}
