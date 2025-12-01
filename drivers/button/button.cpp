#include "button.h"
#include <avr/interrupt.h>

// Pointeurs globaux pour retrouver nos objets Bouton depuis les interruptions
static Button* btnOnPin2 = nullptr;
static Button* btnOnPin3 = nullptr;

Button::Button(uint8_t pin) {
    _pin = pin;
    _semaphore = NULL;
    _lastPressTime = 0;
}

void Button::init() {
    // Création du sémaphore FreeRTOS
    if (_semaphore == NULL) _semaphore = xSemaphoreCreateBinary();
    if (_pin == 2) {
        btnOnPin2 = this;           
        DDRD &= ~(1 << DDD2);      
        PORTD |= (1 << PORTD2); 
        EICRA |= (1 << ISC01);
        EIMSK |= (1 << INT0);
    } 
    else if (_pin == 3) {
        btnOnPin3 = this;
        DDRD &= ~(1 << DDD3);
        PORTD |= (1 << PORTD3);
        EICRA |= (1 << ISC11);
        EIMSK |= (1 << INT1);
    }
}

bool Button::waitForPress(TickType_t timeout) {
    if (_semaphore == NULL) return false;
    return xSemaphoreTake(_semaphore, timeout);
}

void Button::_isrHandler() {
    // Anti-rebond simple (200ms)
    TickType_t now = xTaskGetTickCountFromISR();
    if ((now - _lastPressTime) > pdMS_TO_TICKS(200)) {
        _lastPressTime = now;
        BaseType_t xHigherPriorityTaskWoken = pdFALSE;
        xSemaphoreGiveFromISR(_semaphore, &xHigherPriorityTaskWoken);
        if (xHigherPriorityTaskWoken) taskYIELD();
    }
}

// --- Les Interruptions Système (ISR) ---

// Interruption pour la Pin 2
ISR(INT0_vect) {
    if (btnOnPin2) btnOnPin2->_isrHandler();
}

// Interruption pour la Pin 3
ISR(INT1_vect) {
    if (btnOnPin3) btnOnPin3->_isrHandler();
}