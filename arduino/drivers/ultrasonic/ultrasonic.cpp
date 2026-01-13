/*
    Ultrasonic_AVR.cpp
    A library for ultrasonic ranger using AVR libraries
    
    Adapted from Seeed Technology Inc. original
    Uses AVR libraries instead of Arduino
*/

#include <avr/io.h>
#include <util/delay.h>
#include <stdio.h>
#include <string.h>
#include <inttypes.h>
#include "ultrasonic.h"

// Helper macros for pin manipulation
#define SET_OUTPUT(port, pin) DDR ## port |= (1 << pin)
#define SET_INPUT(port, pin) DDR ## port &= ~(1 << pin)
#define SET_HIGH(port, pin) PORT ## port |= (1 << pin)
#define SET_LOW(port, pin) PORT ## port &= ~(1 << pin)
#define READ_PIN(port, pin) (PIN ## port & (1 << pin))

// Microsecond timer functions (assumes 16MHz clock)
static inline uint32_t micros() {
    // This is a simplified version - you may need to implement a proper timer
    // For accurate timing, use Timer0 or Timer1 with overflow interrupts
    return (TCNT1 * 4); // Assumes Timer1 with prescaler 64: (64/16MHz) * 1000000 = 4
}

static void delayMicroseconds(uint16_t us) {
    while (us--) {
        _delay_us(1);
    }
}

static uint32_t pulseIn(volatile uint8_t *port_reg, volatile uint8_t *pin_reg, 
                        uint8_t pin_mask, uint8_t state, uint32_t timeout) {
    uint32_t begin = micros();
    uint32_t pulse_start, pulse_end;
    
    // Wait for any previous pulse to end
    while ((*pin_reg & pin_mask) == state) {
        if ((micros() - begin) >= timeout) {
            return 0;
        }
    }
    
    // Wait for the pulse to start
    while ((*pin_reg & pin_mask) != state) {
        if ((micros() - begin) >= timeout) {
            return 0;
        }
    }
    pulse_start = micros();
    
    // Wait for the pulse to stop
    while ((*pin_reg & pin_mask) == state) {
        if ((micros() - begin) >= timeout) {
            return 0;
        }
    }
    pulse_end = micros();
    
    return pulse_end - pulse_start;
}

Ultrasonic::Ultrasonic(volatile uint8_t *port, volatile uint8_t *ddr, 
                       volatile uint8_t *pin_reg, uint8_t pin_num) {
    _port = port;
    _ddr = ddr;
    _pin_reg = pin_reg;
    _pin_mask = (1 << pin_num);
}

long Ultrasonic::duration(uint32_t timeout) {
    // Set pin as output
    *_ddr |= _pin_mask;
    
    // Send trigger pulse
    *_port &= ~_pin_mask;  // LOW
    delayMicroseconds(2);
    *_port |= _pin_mask;   // HIGH
    delayMicroseconds(5);
    *_port &= ~_pin_mask;  // LOW
    
    // Set pin as input
    *_ddr &= ~_pin_mask;
    
    // Measure pulse duration
    long dur = pulseIn(_port, _pin_reg, _pin_mask, _pin_mask, timeout);
    return dur;
}

/*The measured distance from the range 0 to 400 Centimeters*/
long Ultrasonic::MeasureInCentimeters(uint32_t timeout) {
    long RangeInCentimeters;
    RangeInCentimeters = duration(timeout) / 29 / 2;
    return RangeInCentimeters;
}

/*The measured distance from the range 0 to 4000 Millimeters*/
long Ultrasonic::MeasureInMillimeters(uint32_t timeout) {
    long RangeInMillimeters;
    RangeInMillimeters = duration(timeout) * (10 / 2) / 29;
    return RangeInMillimeters;
}

/*The measured distance from the range 0 to 157 Inches*/
long Ultrasonic::MeasureInInches(uint32_t timeout) {
    long RangeInInches;
    RangeInInches = duration(timeout) / 74 / 2;
    return RangeInInches;
}
