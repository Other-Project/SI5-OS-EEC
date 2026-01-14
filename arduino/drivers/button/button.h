#ifndef BUTTON_HPP
#define BUTTON_HPP

#include <avr/io.h>
#include "FreeRTOS.h"
#include "task.h"
#include "semphr.h"

class Button
{
public:
    /**
     * @brief Constructor for bare-metal port usage
     * @param pinPort Pointer to the PIN register (e.g., &PINB)
     * @param ddrPort Pointer to the DDR register (e.g., &DDRB)
     * @param portReg Pointer to the PORT register (e.g., &PORTB)
     * @param pinMask Bit mask for the pin (e.g., _BV(PB0))
     */
    Button(volatile uint8_t *pinPort, volatile uint8_t *ddrPort, volatile uint8_t *portReg, uint8_t pinMask);

    /**
     * @brief Initializes the hardware pin and FreeRTOS objects.
     * Must be called before the scheduler starts or inside a setup task.
     */
    void init();

    /**
     * @brief Returns the current debounced state of the button.
     * @return true if pressed, false if released.
     */
    bool isPressed() const;

    /**
     * @brief Blocks the calling task until the button is pressed (debounced).
     * * @param timeout The maximum time to wait in ticks (default: portMAX_DELAY)
     * @return true if button was pressed, false if timeout occurred.
     */
    bool waitForPress(TickType_t timeout = portMAX_DELAY);

private:
    // Hardware registers
    volatile uint8_t *_pinReg;
    volatile uint8_t *_ddrReg;
    volatile uint8_t *_portReg;
    uint8_t _pinMask;

    // RTOS synchronization
    SemaphoreHandle_t _pressSignal;

    // Debouncing state
    uint8_t _history; // Used for bit-shifting debounce history
    bool _isPressed;  // Logical state

    // The static polling task function
    static void pollTask(void *pvParameters);
};

#endif // BUTTON_HPP