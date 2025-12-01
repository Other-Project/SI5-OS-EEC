#include "rfid.h"

void RFID_Reader::begin(long baudRate)
{
    SoftSerial.begin(baudRate);
}

bool RFID_Reader::dataAvailable()
{
    return SoftSerial.available() > 0;
}

size_t RFID_Reader::readData(uint8_t *buffer, size_t maxLength)
{
    size_t count = 0;
    while (SoftSerial.available() && count < maxLength)
    {
        buffer[count++] = SoftSerial.read();
    }
    return count;
}
