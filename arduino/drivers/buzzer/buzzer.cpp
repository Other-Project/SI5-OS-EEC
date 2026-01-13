#include "buzzer.h"

Buzzer::Buzzer(volatile uint8_t *ddr_reg, volatile uint8_t *port_reg, uint8_t pin_mask)
    : _ddr_reg(ddr_reg), _port_reg(port_reg), _pin_mask(pin_mask) 
{
}

void Buzzer::init() {
    // Configure la pin en sortie (Data Direction Register)
    *_ddr_reg |= _pin_mask;
    // S'assure que le buzzer est éteint au départ
    off();
}

void Buzzer::on() {
    // Met le bit correspondant à 1 dans le registre PORT
    *_port_reg |= _pin_mask;
}

void Buzzer::off() {
    // Met le bit correspondant à 0 dans le registre PORT
    *_port_reg &= ~_pin_mask;
}

void Buzzer::toggle() {
    // Inverse l'état du bit dans le registre PORT
    *_port_reg ^= _pin_mask;
}