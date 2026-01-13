#include "i2c.h"
#include <string.h>

// Global variables for the protocol
static volatile uint8_t g_registers[I2C_NUM_REGISTERS];
static volatile uint8_t g_register_pointer = 0;
static volatile I2CCallback g_register_callback;

void I2C_Protocol::init(uint8_t slave_address)
{
    // Initialize registers to zero
    memset((void *)g_registers, 0, I2C_NUM_REGISTERS);
    memset((void *)g_register_callback, 0, sizeof(g_register_callback));

    // Initialize Wire in slave mode with the specified address
    Wire.begin(slave_address);

    // Register Wire callbacks for receive and request events
    Wire.onReceive(onReceiveHandler);
    Wire.onRequest(onRequestHandler);
}

void I2C_Protocol::setRegister(uint8_t reg, uint8_t value)
{
    if (reg < I2C_NUM_REGISTERS)
        g_registers[reg] = value;
}

uint8_t I2C_Protocol::getRegister(uint8_t reg)
{
    if (reg < I2C_NUM_REGISTERS)
        return g_registers[reg];
    return 0;
}

void I2C_Protocol::registerCallback(I2CCallback callback)
{
    g_register_callback = callback;
}

// Handler called when the Master (Raspberry Pi) writes data
void I2C_Protocol::onReceiveHandler(int numBytes)
{
    if (numBytes < 1)
        return;

    // Read register address (first byte)
    g_register_pointer = Wire.read();
    numBytes--;

    // Read data to write into consecutive registers
    while (numBytes > 0 && Wire.available())
    {
        if (g_register_pointer < I2C_NUM_REGISTERS)
        {
            uint8_t value = Wire.read();
            g_registers[g_register_pointer] = value;

            // Call the callback if defined
            if (g_register_callback != NULL)
                g_register_callback(g_register_pointer, value);

            g_register_pointer++;
        }
        else
            Wire.read(); // Consume the byte even if it cannot be stored
        numBytes--;
    }
}

// Handler called when the Master (Raspberry Pi) requests data
void I2C_Protocol::onRequestHandler()
{
    // Send the value of the pointed register
    if (g_register_pointer < I2C_NUM_REGISTERS)
    {
        Wire.write(g_registers[g_register_pointer]);
        g_register_pointer++;
    }
    else
        Wire.write(0x00); // Send 0 if out of range
}
