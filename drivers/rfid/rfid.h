#ifndef RFID_H
#define RFID_H

#include <SoftwareSerial.h>

class RFID_Reader
{
private:
    SoftwareSerial SoftSerial;

public:
    RFID_Reader(int rxPin, int txPin) : SoftSerial(rxPin, txPin) {}
    void begin(long baudRate = 9600);
    bool dataAvailable();
    size_t readData(uint8_t *buffer, size_t maxLength);
};

#endif
