#include "button.h"

Button::Button(volatile uint8_t *pinPort, volatile uint8_t *ddrPort, volatile uint8_t *portReg, uint8_t pinMask)
    : _pinReg(pinPort), _ddrReg(ddrPort), _portReg(portReg), _pinMask(pinMask),
      _pressSignal(nullptr), _history(0), _isPressed(false)
{
}

void Button::init()
{
    // Configure as input with pull-up
    if (_ddrReg && _portReg)
    {
        *_ddrReg &= ~_pinMask;
        *_portReg |= _pinMask;
    }

    _pressSignal = xSemaphoreCreateBinary();

    xTaskCreate(
        pollTask,
        "BtnPoll",
        configMINIMAL_STACK_SIZE,
        this,
        tskIDLE_PRIORITY + 1,
        NULL);
}

bool Button::waitForPress(TickType_t timeout)
{
    if (_pressSignal == nullptr)
        return false;
    return (xSemaphoreTake(_pressSignal, timeout) == pdTRUE);
}

bool Button::isPressed() const
{
    return _isPressed;
}

void Button::pollTask(void *pvParameters)
{
    Button *self = static_cast<Button *>(pvParameters);
    TickType_t xLastWakeUpTime = xTaskGetTickCount();

    while (1)
    {
        // Active-low: 0 = Pressed, 1 = Released
        bool rawState = !(*self->_pinReg & self->_pinMask);

        // Debounce: keep history of last 8 polls
        self->_history = (self->_history << 1) | (rawState ? 1 : 0);

        // Check for stable states on last 4 samples
        bool stablePressed = (self->_history & 0x0F) == 0x00;
        bool stableReleased = (self->_history & 0x0F) == 0x0F;

        if (stablePressed && !self->_isPressed)
        {
            self->_isPressed = true;
            if (self->_pressSignal != nullptr)
            {
                xSemaphoreGive(self->_pressSignal);
            }
        }
        else if (stableReleased && self->_isPressed)
        {
            self->_isPressed = false;
        }

        vTaskDelayUntil(&xLastWakeUpTime, 50 / portTICK_PERIOD_MS);
    }
}