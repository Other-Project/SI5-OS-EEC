#ifndef BUTTON_H
#define BUTTON_H

#include <avr/io.h>
#include "FreeRTOS.h"
#include "semphr.h"

class Button {
public:
    // Constructeur : on choisit la pin ici (2 ou 3)
    Button(uint8_t pin);

    // Initialisation du matériel
    void init();

    // Attend un clic (bloquant, anti-rebond inclus)
    bool waitForPress(TickType_t timeout = portMAX_DELAY);

    // Méthode interne appelée par l'interruption (ne pas utiliser manuellement)
    void _isrHandler();

private:
    uint8_t _pin;
    SemaphoreHandle_t _semaphore;
    volatile TickType_t _lastPressTime;
};

#endif