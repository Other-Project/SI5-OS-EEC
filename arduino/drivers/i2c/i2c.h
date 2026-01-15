#ifndef I2C_PROTOCOL_H
#define I2C_PROTOCOL_H

#include <Wire.h>

// Definition of protocol registers
#define I2C_NUM_REGISTERS 32

// Callback type for register changes
typedef void (*I2CCallback)(uint8_t reg, uint8_t value);

class I2C_Protocol
{
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
