pub const REG_STATUS: u8 = 0x00;
pub const REG_EVENTS: u8 = 0x01;
pub const REG_RFID: u8 = 0x02;

pub const EVENT_BTN_PRESSED: u8 = 0x01;
pub const EVENT_MOTION_DETECTED: u8 = 0x02;
pub const EVENT_RFID_READ: u8 = 0x04;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SecurityState {
    Disarmed = 0,  // Veille
    Armed = 1,     // Surveillance
    Triggered = 2, // ALERTE (Sonnerie)
}
