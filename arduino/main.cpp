#include "FreeRTOS.h"
#include "task.h"
#include <avr/io.h>
#include <Wire.h>
#include "drivers/lcd/lcd.h"
#include "drivers/rfid/rfid.h"
#include "drivers/buzzer/buzzer.h"
#include "drivers/ultrasonic/ultrasonic.h"
#include "drivers/button/button.h"
#include "drivers/rotary_angle/rotary_angle.h"
#include "drivers/i2c/i2c.h"
#include "consts.h"
#include <event_groups.h>

// Event group
EventGroupHandle_t xSystemStateGroup;
#define BIT_BLINK_ENABLE (1 << 0)
#define BIT_ULTRASONIC_ENABLE (1 << 1)

// Tasks
static void vUltrasonicTask(void *pvParameters);
static void vBlinkTask(void *pvParameters);
static void vReadRfid(void *pvParameters);
static void vButtonTask(void *pvParameters);

static TaskHandle_t xUltrasonicHandle = NULL;
static TaskHandle_t xBlinkHandle = NULL;

// Peripherals
static RFID_Reader rfid(7, 8);
static Buzzer buzzer(&DDRD, &PORTD, _BV(PD6));
static Buzzer led(&DDRD, &PORTD, _BV(PD5));
static Button button(&PIND, &DDRD, &PORTD, _BV(PD2));

void setEventFlag(uint8_t event, bool enable)
{
    uint8_t currentEvents = I2C_Protocol::getRegister(REG_EVENTS);
    if (enable)
        currentEvents |= event;
    else
        currentEvents &= ~event;
    I2C_Protocol::setRegister(REG_EVENTS, currentEvents);
}

void onStatusChange(uint8_t newStatus)
{
    xEventGroupClearBits(xSystemStateGroup, BIT_BLINK_ENABLE | BIT_ULTRASONIC_ENABLE);

    switch (newStatus)
    {
    case STATUS_DISARMED:
        led.off();
        buzzer.off();
        break;

    case STATUS_ARMED:
        xEventGroupSetBits(xSystemStateGroup, BIT_ULTRASONIC_ENABLE);
        led.on();
        buzzer.off();
        break;

    case STATUS_TRIGGERED:
        xEventGroupSetBits(xSystemStateGroup, BIT_BLINK_ENABLE);
        led.on();
        buzzer.on();
        break;

    default:
        break;
    }
}

void onI2CCommand(uint8_t reg, uint8_t value)
{
    switch (reg)
    {
    case REG_STATUS:
        onStatusChange(value);
        break;
    default:
        break;
    }
}

int main(void)
{
    // Initialize Wire (required before I2C_Protocol)
    Wire.begin();

    // Initialize I2C in slave mode (address 0x32)
    I2C_Protocol::init(0x32);

    // Register callbacks for commands coming from the Raspberry Pi
    I2C_Protocol::registerCallback(onI2CCommand);

    // Set initial register values
    I2C_Protocol::setRegister(REG_STATUS, STATUS_DISARMED);
    I2C_Protocol::setRegister(REG_ULTRASONIC_DISTANCE, 4); // 20 mm

    // Initialize peripherals
    led.init();
    buzzer.init();
    button.init();
    rfid.begin(9600);

    // Create event group
    xSystemStateGroup = xEventGroupCreate();

    // Create tasks
    xTaskCreate(vUltrasonicTask, "ultrasonic", configMINIMAL_STACK_SIZE, NULL, 1U, &xUltrasonicHandle);
    xTaskCreate(vBlinkTask, "blink", configMINIMAL_STACK_SIZE, NULL, 1U, &xBlinkHandle);
    xTaskCreate(vReadRfid, "rfid", configMINIMAL_STACK_SIZE, NULL, 1U, NULL);
    xTaskCreate(vButtonTask, "button", configMINIMAL_STACK_SIZE, NULL, 1U, NULL);

    // Start scheduler
    vTaskStartScheduler();

    return 0;
}

static void vUltrasonicTask(void *pvParameters)
{
    Ultrasonic ultrasonic(&PORTD, &DDRD, &PIND, PD4);
    TickType_t xLastWakeUpTime = xTaskGetTickCount();

    while (1)
    {
        // Wait for BIT_ULTRASONIC_ENABLE to be set
        xEventGroupWaitBits(xSystemStateGroup, BIT_ULTRASONIC_ENABLE, pdFALSE, pdFALSE, portMAX_DELAY);

        uint16_t distance_mm = ultrasonic.MeasureInMillimeters();
        uint16_t max_distance_mm = I2C_Protocol::getRegister(REG_ULTRASONIC_DISTANCE) * 5;
        bool motion = distance_mm > max_distance_mm;
        setEventFlag(EVENT_MOTION_DETECTED, motion);
        vTaskDelayUntil(&xLastWakeUpTime, 200 / portTICK_PERIOD_MS);
    }
}

static void vReadRfid(void *pvParameters)
{
    TickType_t xLastWakeUpTime = xTaskGetTickCount();
    while (1)
    {
        setEventFlag(EVENT_RFID_READ, false);
        if (rfid.dataAvailable())
        {
            uint64_t cardNumber = rfid.readCardNumber(); // 48 bits card number
            if (cardNumber != 0)
            {
                // Store card number in I2C register
                for (int i = 0; i < 6; i++)
                {
                    uint8_t byteValue = (cardNumber >> (8 * (5 - i))) & 0xFF;
                    I2C_Protocol::setRegister(REG_RFID + i, byteValue);
                }

                // Set RFID read event
                setEventFlag(EVENT_RFID_READ, true);
                vTaskDelay(100 / portTICK_PERIOD_MS);
                continue;
            }
        }
        vTaskDelayUntil(&xLastWakeUpTime, 100 / portTICK_PERIOD_MS);
    }
}

static void vButtonTask(void *pvParameters)
{
    while (1)
    {
        setEventFlag(EVENT_BTN_PRESSED, button.isPressed());
        vTaskDelay(100 / portTICK_PERIOD_MS);
    }
}

static void vBlinkTask(void *pvParameters)
{
    TickType_t xLastWakeUpTime = xTaskGetTickCount();

    while (1)
    {
        // Wait for BIT_BLINK_ENABLE to be set
        xEventGroupWaitBits(xSystemStateGroup, BIT_BLINK_ENABLE, pdFALSE, pdFALSE, portMAX_DELAY);

        led.toggle();
        buzzer.toggle();
        vTaskDelayUntil(&xLastWakeUpTime, 500 / portTICK_PERIOD_MS);
    }
}
