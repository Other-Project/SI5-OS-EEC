#include "rotary_angle.h"
#include <avr/io.h>
#include <util/delay.h>

// ADC initialization helper
static void adc_init() {
    // Référence AVcc, résultat sur 10 bits, ADC activé
    ADMUX = (1 << REFS0); // Référence AVcc (5V)
    ADCSRA = (1 << ADEN)  | // Activation ADC
             (1 << ADPS2) | (1 << ADPS1) | (1 << ADPS0); // Prescaler = 128 (16 MHz / 128 = 125kHz)
}

// ADC read helper fonction sur une pin analogique (0..5)
static uint16_t adc_read(uint8_t channel) {
    // Sélection de la pin ADC, masque les bits MUX[3:0]
    ADMUX = (ADMUX & 0xF0) | (channel & 0x0F);

    // Démarre la conversion
    ADCSRA |= (1 << ADSC);

    // Attend la fin de conversion
    while (ADCSRA & (1 << ADSC));

    // Retourne la valeur 10 bits
    return ADCW;
}

// Constructeur
RotaryAngle::RotaryAngle(uint8_t analogPin)
    : _analogPin(analogPin) {
    // On ne fait rien ici
}

void RotaryAngle::init() {
    adc_init();
    // Pas besoin d'autre init matériel spécifique
}

uint16_t RotaryAngle::readRaw() {
    return adc_read(_analogPin);
}

uint16_t RotaryAngle::readDegrees() {
    uint16_t raw = readRaw();
    // La plage ADC est 0 - 1023, la plage angle 0 - 300 degrés
    uint16_t degrees = (raw * 300) / 1023;
    return degrees;
}