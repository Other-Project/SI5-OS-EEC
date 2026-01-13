use bitflags::bitflags;

#[allow(unused)]
pub enum Register {
    Status = 0x00,
    Events = 0x01,
    Rfid = 0x02,
    UltrasonicDistance = 0x08,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SecurityState {
    Disarmed = 0,
    Armed = 1,
    Triggered = 2,
}

bitflags! {
    #[derive(Clone, Copy)]
    pub struct Events: u8 {
        const BTN_PRESSED = 0x01;
        const MOTION_DETECTED = 0x02;
        const RFID_READ = 0x04;
    }
}
