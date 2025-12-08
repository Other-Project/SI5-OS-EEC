#include "i2c.h"

void I2C_Reader::begin(int address)
{
    Wire.begin(address);
}

bool I2C_Reader::dataAvailable()
{
    return Wire.available() > 0;
}

size_t I2C_Reader::readData(uint8_t *buffer, size_t maxLength)
{
    Wire.requestFrom(8, 6);
    size_t count = 0;
    while (Wire.available() && count < maxLength)
    {
        buffer[count++] = Wire.read();
    }
    return count;
   // request 6 bytes from peripheral device #8
}
