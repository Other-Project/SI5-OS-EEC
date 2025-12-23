#include "rfid.h"

void RFID_Reader::begin(long baudRate)
{
    SoftSerial.begin(baudRate);
}

bool RFID_Reader::dataAvailable()
{
    return SoftSerial.available() > 0;
}

size_t RFID_Reader::readRawData(uint8_t *buffer, size_t maxLength)
{
    size_t count = 0;
    while (SoftSerial.available() && count < maxLength)
    {
        buffer[count++] = SoftSerial.read();
    }
    return count;
}

uint64_t RFID_Reader::readCardNumber()
{
    uint64_t cardNumber = 0;
    uint8_t parity = 0;

    while (SoftSerial.available() && SoftSerial.read() != 0x02)
        ; // wait for start byte

    while (SoftSerial.available())
    {
        char c = SoftSerial.read();

        // Decode hex character
        uint8_t value;
        if (c >= '0' && c <= '9')
            value = c - '0';
        else if (c >= 'A' && c <= 'F')
            value = c - 'A' + 10;
        else if (c >= 'a' && c <= 'f')
            value = c - 'a' + 10;
        else if (c == 0x03)
            break; // stop byte
        cardNumber = (cardNumber << 4) | value;
    }

    while (SoftSerial.available())
        SoftSerial.read(); // flush remaining bytes

    return cardNumber;
}
