#pragma region Register map

// System status register
#define REG_STATUS 0x00

// Event flags register
#define REG_EVENTS 0x01

// RFID tag data registers (0x02 to 0x07)
#define REG_RFID 0x02

/// Ultrasonic distance register (in steps of 5 mm)
#define REG_ULTRASONIC_DISTANCE 0x08

#pragma endregion

#pragma region System status

// The alarm is disarmed (won't trigger)
#define STATUS_DISARMED 0x00

// The alarm is armed (will trigger on motion)
#define STATUS_ARMED 0x01

// The alarm has been triggered
#define STATUS_TRIGGERED 0x02

#pragma endregion

#pragma region Event flags

// Button pressed event
#define EVENT_BTN_PRESSED 0x01
// Motion detected event
#define EVENT_MOTION_DETECTED 0x02
// RFID tag read event
#define EVENT_RFID_READ 0x04

#pragma endregion
