#ifndef ROTARY_ANGLE_H
#define ROTARY_ANGLE_H

#include <avr/io.h>
#include "FreeRTOS.h"
#include "task.h"

// Classe driver pour le capteur Grove Rotary Angle Sensor.
// Le capteur est connecté sur une entrée analogique (ex: A0).
// Le driver fournit une méthode pour lire la valeur raw (0-1023) du capteur.

class RotaryAngle {
public:
    // Constructeur : indique la pin analogique (0 pour A0, 1 pour A1, etc)
    RotaryAngle(uint8_t analogPin);

    // Initialise la pin (analogique)
    void init();

    // Lit la valeur analogique brute (0 - 1023)
    uint16_t readRaw();

    // Lit la valeur en degrés (0 - 300 approx), calculée à partir de la lecture analogique
    long readDegrees();

private:
    uint8_t _analogPin;
};

#endif // ROTARY_ANGLE_H