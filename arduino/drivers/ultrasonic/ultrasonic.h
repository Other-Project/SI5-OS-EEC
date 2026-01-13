/*
    Ultrasonic_AVR.h
    Header file for ultrasonic ranger using AVR libraries
*/

#ifndef ULTRASONIC_AVR_H
#define ULTRASONIC_AVR_H

#include <avr/io.h>
#include <inttypes.h>

class Ultrasonic {
public:
    // Constructor now takes port registers and pin number
    // Example: Ultrasonic(&PORTB, &DDRB, &PINB, PB0)
    Ultrasonic(volatile uint8_t *port, volatile uint8_t *ddr, 
               volatile uint8_t *pin_reg, uint8_t pin_num);
    
    long duration(uint32_t timeout = 1000000L);
    long MeasureInCentimeters(uint32_t timeout = 1000000L);
    long MeasureInMillimeters(uint32_t timeout = 1000000L);
    long MeasureInInches(uint32_t timeout = 1000000L);

private:
    volatile uint8_t *_port;
    volatile uint8_t *_ddr;
    volatile uint8_t *_pin_reg;
    uint8_t _pin_mask;
};

#endif // ULTRASONIC_AVR_H
