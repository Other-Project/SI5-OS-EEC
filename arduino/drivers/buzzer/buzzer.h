#ifndef BUZZER_H
#define BUZZER_H

#include <avr/io.h>

class Buzzer {
public:
    /**
     * Constructeur
     * @param ddr_reg Pointeur vers le registre de direction (ex: &DDRB)
     * @param port_reg Pointeur vers le registre de port (ex: &PORTB)
     * @param pin_mask Masque binaire de la pin (ex: 1 << 0 pour PB0)
     */
    Buzzer(volatile uint8_t *ddr_reg, volatile uint8_t *port_reg, uint8_t pin_mask);

    // Initialise la pin en sortie
    void init();

    // Active le buzzer (Met la pin à l'état HAUT)
    void on();

    // Désactive le buzzer (Met la pin à l'état BAS)
    void off();

    // Inverse l'état du buzzer (Utile pour les buzzers passifs)
    void toggle();

private:
    volatile uint8_t *_ddr_reg;
    volatile uint8_t *_port_reg;
    uint8_t _pin_mask;
};

#endif