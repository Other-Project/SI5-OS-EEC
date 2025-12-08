#ifndef I2C_PROTOCOL_H
#define I2C_PROTOCOL_H

#include <Wire.h>

// Definition of protocol registers
#define I2C_NUM_REGISTERS 32

// Register map (customize as needed)
#define REG_STATUS          0x00  // General system status
#define REG_ALARM_STATE     0x01  // Alarm state (0=off, 1=on)
#define REG_MOTION_DETECTED 0x02  // Motion detected (0=no, 1=yes)
#define REG_BUZZER_CMD      0x03  // Buzzer command (0=off, 1=on)
#define REG_LED_CMD         0x04  // LED command (0=off, 1=on)
#define REG_DISTANCE_H      0x05  // Ultrasonic sensor distance (MSB)
#define REG_DISTANCE_L      0x06  // Ultrasonic sensor distance (LSB)
#define REG_RFID_STATUS     0x07  // RFID status (0=no tag, 1=tag present)
#define REG_RFID_ID_0       0x08  // RFID ID byte 0
#define REG_RFID_ID_1       0x09  // RFID ID byte 1
#define REG_RFID_ID_2       0x0A  // RFID ID byte 2
#define REG_RFID_ID_3       0x0B  // RFID ID byte 3
#define REG_RFID_ID_4       0x0C  // RFID ID byte 4
#define REG_RFID_ID_5       0x0D  // RFID ID byte 5
#define REG_ROTARY_ANGLE    0x10  // Potentiometer angle
#define REG_BUTTON_STATE    0x11  // Button state (0=released, 1=pressed)
#define REG_COMMAND         0x12  // General command register
#define REG_ERROR_CODE      0x13  // Error code

// Callback type for register changes
typedef void (*I2CCallback)(uint8_t reg, uint8_t value);

class I2C_Protocol {
public:
    /**
     * Initialize the I2C protocol in slave mode with Wire
     * @param slave_address I2C address of the Arduino (0x32 by default)
     */
    static void init(uint8_t slave_address = 0x32);
    
    /**
     * Set the value of a register
     * @param reg Register number
     * @param value Value to write
     */
    static void setRegister(uint8_t reg, uint8_t value);
    
    /**
     * Read the value of a register
     * @param reg Register number
     * @return Register value
     */
    static uint8_t getRegister(uint8_t reg);
    
    /**
     * Register a callback called when a register is modified by the master
     * @param callback Function to call
     */
    static void registerCallback(I2CCallback callback);

private:
    /**
     * Wire handler called when the Master sends data
     * @param numBytes Number of bytes received
     */
    static void onReceiveHandler(int numBytes);
    
    /**
     * Wire handler called when the Master requests data
     */
    static void onRequestHandler();
};

#endif // I2C_PROTOCOL_H
