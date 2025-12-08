#ifndef I2C_H
#define I2C_H

#include <Wire.h>

class I2C_Reader
{
public:
    void begin(int address = 0x32);
    bool dataAvailable();
    size_t readData(uint8_t *buffer, size_t maxLength);
};

#endif
