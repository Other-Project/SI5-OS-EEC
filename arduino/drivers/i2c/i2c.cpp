#include "i2c.h"
#include <string.h>
#include "FreeRTOS.h"
#include <task.h>

#define READ_I2C() (numBytes--, Wire.read())
#define READ_FLAG 0x80

// Protocol state variables
static volatile uint8_t g_registers[I2C_NUM_REGISTERS];
static volatile uint8_t g_register_pointer;
static volatile uint8_t g_read_count;
static volatile I2CCallback g_register_callback;

void I2C_Protocol::init(uint8_t slave_address)
{
    // Initialize state variables
    memset((void *)g_registers, 0, I2C_NUM_REGISTERS);
    memset((void *)g_register_callback, 0, sizeof(g_register_callback));
    g_register_pointer = 0;
    g_read_count = 0;

    // Initialize Wire in slave mode with the specified address
    Wire.begin(slave_address);

    // Register Wire callbacks for receive and request events
    Wire.onReceive(onReceiveHandler);
    Wire.onRequest(onRequestHandler);
}

void I2C_Protocol::setRegister(uint8_t reg, uint8_t value)
{
    if (reg < I2C_NUM_REGISTERS)
    {
        taskENTER_CRITICAL();
        g_registers[reg] = value;
        taskEXIT_CRITICAL();
    }
}

uint8_t I2C_Protocol::getRegister(uint8_t reg)
{
    uint8_t value = 0;
    if (reg < I2C_NUM_REGISTERS)
    {
        taskENTER_CRITICAL();
        value = g_registers[reg];
        taskEXIT_CRITICAL();
    }
    return value;
}

void I2C_Protocol::registerCallback(I2CCallback callback)
{
    g_register_callback = callback;
}

// Handler called when the Master (Raspberry Pi) writes data
void I2C_Protocol::onReceiveHandler(int numBytes)
{
    if (numBytes < 2)
        return;

    // Read register address (first byte)
    g_register_pointer = READ_I2C();
    uint8_t value = READ_I2C();

    if (g_register_pointer & READ_FLAG)
    {
        g_register_pointer &= ~READ_FLAG;
        g_read_count = value;
        return;
    }

    uint8_t values[numBytes];
    uint16_t checksum = 0;
    for (uint8_t i = 0; i < numBytes; i++)
    {
        values[i] = READ_I2C();
        checksum += values[i];
    }
    if ((checksum & 0xFF) != value)
        return; // Checksum mismatch, ignore the write

    // Write values to registers
    for (uint8_t i = 0; i < numBytes; i++, g_register_pointer++)
    {
        I2C_Protocol::setRegister(g_register_pointer, values[i]);
        // Call the registered callback if any
        if (g_register_callback)
            g_register_callback(g_register_pointer, values[i]);
    }

    // Reset state variables
    g_register_pointer = 0;
    g_read_count = 0;
}

// Handler called when the Master (Raspberry Pi) requests data
void I2C_Protocol::onRequestHandler()
{
    uint16_t checksum = 0;
    while (g_read_count-- > 0)
    {
        uint8_t value = I2C_Protocol::getRegister(g_register_pointer++);
        checksum += value;
        Wire.write(value);
    }
    Wire.write(checksum & 0xFF);

    // Reset state variables after read
    g_register_pointer = 0;
    g_read_count = 0;
}
