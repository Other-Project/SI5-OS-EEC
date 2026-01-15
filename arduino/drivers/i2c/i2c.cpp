#include "i2c.h"
#include <string.h>
#include "FreeRTOS.h"
#include <task.h>
#include <HardwareSerial.h>

#define READ_FLAG 0x80

static volatile uint8_t g_registers[I2C_NUM_REGISTERS];
static volatile uint8_t g_register_pointer = 0;
static volatile uint8_t g_read_count = 0;
static volatile I2CCallback g_register_callback = nullptr;

void I2C_Protocol::init(uint8_t slave_address)
{
    memset((void *)g_registers, 0, I2C_NUM_REGISTERS);
    g_register_callback = nullptr;
    g_register_pointer = 0;
    g_read_count = 0;

    Wire.begin(slave_address);
    Wire.onReceive(onReceiveHandler);
    Wire.onRequest(onRequestHandler);
}

void I2C_Protocol::registerCallback(I2CCallback callback)
{
    g_register_callback = callback;
}

// Access registers without locking (for use inside ISR/Critical sections)
static inline void setRegisterInternal(uint8_t reg, uint8_t value)
{
    if (reg < I2C_NUM_REGISTERS)
        g_registers[reg] = value;
}
static inline uint8_t getRegisterInternal(uint8_t reg)
{
    if (reg < I2C_NUM_REGISTERS)
        return g_registers[reg];
    return 0;
}

// Uses locking
void I2C_Protocol::setRegister(uint8_t reg, uint8_t value)
{
    taskENTER_CRITICAL();
    setRegisterInternal(reg, value);
    taskEXIT_CRITICAL();
}

uint8_t I2C_Protocol::getRegister(uint8_t reg)
{
    taskENTER_CRITICAL();
    uint8_t value = getRegisterInternal(reg);
    taskEXIT_CRITICAL();
    return value;
}

// ---------------------------------------------------------
// HANDLERS (Must be fast, No Serial, No Blocked Waits)
// ---------------------------------------------------------

void I2C_Protocol::onReceiveHandler(int numBytes)
{
    if (numBytes < 2)
        return;

    taskENTER_CRITICAL();

    uint8_t ptr_byte = Wire.read();
    uint8_t val_byte = Wire.read();

    // CASE A: Setup for Read (Pointer | 0x80)
    if (ptr_byte & READ_FLAG)
    {
        g_register_pointer = ptr_byte & ~READ_FLAG;
        g_read_count = val_byte;
        // Clear buffer just in case
        while (Wire.available())
            Wire.read();
        taskEXIT_CRITICAL();
        return;
    }

    // CASE B: Write Data
    int data_len = numBytes - 2;
    if (data_len > I2C_NUM_REGISTERS || data_len < 0)
    {
        taskEXIT_CRITICAL();
        return;
    }

    // Verify Checksum
    uint8_t values[data_len];
    uint16_t checksum = 0;
    for (int i = 0; i < data_len; i++)
    {
        values[i] = Wire.read();
        checksum += values[i];
    }
    if ((checksum & 0xFF) != val_byte)
    {
        taskEXIT_CRITICAL();
        return;
    }

    // Apply Changes
    for (int i = 0; i < data_len; i++)
        setRegisterInternal(g_register_pointer + i, values[i]);

    uint8_t start_reg = g_register_pointer;
    // Reset State
    g_register_pointer = 0;
    g_read_count = 0;

    taskEXIT_CRITICAL();

    // --- Interrupts Enabled Here ---
    // Trigger Callback (Careful: this runs in ISR context!)
    if (g_register_callback)
    {
        for (int i = 0; i < data_len; i++)
            g_register_callback(start_reg + i, values[i]);
    }
}

void I2C_Protocol::onRequestHandler()
{
    taskENTER_CRITICAL();

    uint8_t count = g_read_count;
    uint8_t start_reg = g_register_pointer;

    // Protection against 0 count
    if (count == 0)
    {
        Wire.write(0xFF);
        taskEXIT_CRITICAL();
        return;
    }

    uint16_t checksum = 0;
    for (uint8_t i = 0; i < count; i++)
    {
        uint8_t val = getRegisterInternal(start_reg + i);
        checksum += val;
        Wire.write(val);
    }
    Wire.write(checksum & 0xFF);

    // Auto-reset after read
    g_register_pointer = 0;
    g_read_count = 0;

    taskEXIT_CRITICAL();
}