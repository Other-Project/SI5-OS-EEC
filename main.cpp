/*
 * --------------------------------------------------------------------------------
 * "THE BEER-WARE LICENSE" (Revision 42):
 * <julien.deantoni@univ-cotedazur.fr> wrote this file.
 * As long as you retain this notice you can do whatever you want with this stuff.
 * If we meet some day, and you think this stuff is worth it,
 * you can buy me a beer in return.  Julien Deantoni
 * --------------------------------------------------------------------------------
 */

/******************************************************************************
 * Header file inclusions.
 ******************************************************************************/

#include "FreeRTOS.h"
#include "task.h" /* RTOS task related API prototypes. */
#include <avr/io.h>
#include "drivers/lcd/lcd.h"
#include "drivers/rfid/rfid.h"
#include "drivers/buzzer/buzzer.h"
#include "drivers/ultrasonic/ultrasonic.h"
#include "drivers/button/button.h"
#include "drivers/rotary_angle/rotary_angle.h"
#include "drivers/i2c/i2c.h"

/******************************************************************************
 * Private macro definitions.
 ******************************************************************************/

// Each task is assigned a priority from 0 to ( configMAX_PRIORITIES - 1 ), where configMAX_PRIORITIES is defined within FreeRTOSConfig.h.
//  Low priority numbers denote low priority tasks. The idle task has priority zero (tskIDLE_PRIORITY).
#define mainLED_TASK_PRIORITY (tskIDLE_PRIORITY)

// tasks handler defined after the main
static void vReadRfid(void *pvParameters);
static void vBuzzerTask(void *pvParameters);
static void vUltrasonicTask(void *pvParameters);
static void vRotaryAngleTask(void *pvParameters);
static void vI2CTask(void *pvParameters);

// constant to ease the reading....
/*const uint8_t redLed   = _BV(PD2);
const uint8_t greenLed = _BV(PD3);*/

static RFID_Reader rfid(7, 8);
static uint8_t buffer[16];
//static LCD lcd = LCD();
static Buzzer grooveBuzzer(&DDRD, &PORTD, _BV(PD6));
static Button myButton(2); // uniquement pin 2 ou 3
static RotaryAngle rotaryAngle(0); // A0
static I2C_Reader i2cReader;

int main(void)
{
    // Initialize the LCD
    /*lcd.begin(16, 2, LCD_2LINE);
    lcd.clear();*/

    i2cReader.begin();

    // Initialize the button
    myButton.init();

    rfid.begin(9600);

    rotaryAngle.init();

    /*xTaskCreate(
        vReadRfid,
        "rfid",
        configMINIMAL_STACK_SIZE,
        NULL,
        1U,
        NULL);*/

    xTaskCreate(
        vBuzzerTask,
        "buzzer",
        configMINIMAL_STACK_SIZE,
        NULL,
        1U,
        NULL);

    /*xTaskCreate(
        vUltrasonicTask,
        "ultrasonic",
        configMINIMAL_STACK_SIZE,
        NULL,
        1U,
        NULL);*/

    xTaskCreate(
        vRotaryAngleTask,
        "rotary",
        configMINIMAL_STACK_SIZE,
        NULL,
        1U,
        NULL);

    xTaskCreate(
        vI2CTask,
        "i2c",
        configMINIMAL_STACK_SIZE,
        NULL,
        1U,
        NULL);

    // Start scheduler.
    vTaskStartScheduler();

    return 0;
}

static void vReadRfid(void *pvParameters)
{
    TickType_t xLastWakeUpTime = xTaskGetTickCount();
    size_t length;
    while (1)
    {
        if (rfid.dataAvailable())
        {
            buffer[0] = '\0';
            length = rfid.readData(buffer, sizeof(buffer) - 1);
            if (length == 14)
            {
                buffer[length - 1] = '\0'; // Ignore ending character
                /*lcd.clear();
                lcd.print(buffer + 1); // Skip starting character*/
                vTaskDelayUntil(&xLastWakeUpTime, 2000 / portTICK_PERIOD_MS);
                continue;
            }
        }

        /*lcd.clear();
        lcd.print("No RFID Data");*/
        vTaskDelayUntil(&xLastWakeUpTime, 50 / portTICK_PERIOD_MS); // Polling delay
    }
}

static void vUltrasonicTask(void *pvParameters)
{
    // Connected to D4 on the Base Shield
    Ultrasonic ultrasonic(&PORTD, &DDRD, &PIND, PD4);
    TickType_t xLastWakeUpTime = xTaskGetTickCount();
    while (1)
    {
        long distance_cm = ultrasonic.MeasureInMillimeters();
        snprintf((char *)buffer, sizeof(buffer), "Dist: %ld mm", distance_cm % 100);
        /*lcd.clear();
        lcd.print(buffer);*/
        // Here you can add code to display the distance on the LCD or process it further
        vTaskDelayUntil(&xLastWakeUpTime, 1000 / portTICK_PERIOD_MS); // Polling delay
    }
}

static void vBuzzerTask(void *pvParameters)
{
    // Initialisation matérielle (direction des I/O)
    grooveBuzzer.init();

    while (1)
    {
        //wait for button press
        myButton.waitForPress();

        // Bip
        grooveBuzzer.on();
        vTaskDelay(5000 / portTICK_PERIOD_MS); // Son pendant 100ms
        grooveBuzzer.off();
    }
}

static void vRotaryAngleTask(void *pvParameters)
{
    TickType_t xLastWakeUpTime = xTaskGetTickCount();
    char displayBuffer[16];

    while (1)
    {
        long angle = rotaryAngle.readDegrees();

        //lcd.clear();
        // Affiche l'angle en degrés (ex: "Angle: 123.4 deg")
        snprintf(displayBuffer, sizeof(displayBuffer), "Angle: %ld deg", angle);
        //lcd.print((uint8_t *)displayBuffer);

        // Rafraichissement toutes les 500ms
        vTaskDelayUntil(&xLastWakeUpTime, 500 / portTICK_PERIOD_MS);
    }
}

static void vI2CTask(void *pvParameters)
{
    TickType_t xLastWakeUpTime = xTaskGetTickCount();
    size_t length;
    while (1)
    {
        if (i2cReader.dataAvailable())
        {
            buffer[0] = '\0';
            length = i2cReader.readData(buffer, sizeof(buffer) - 1);
            if (length == 14)
            {
                buffer[length - 1] = '\0'; // Ignore ending character
                /*lcd.clear();
                lcd.print(buffer + 1); // Skip starting character*/
                vTaskDelayUntil(&xLastWakeUpTime, 2000 / portTICK_PERIOD_MS);
                continue;
            }
        }

        /*lcd.clear();
        lcd.print("No RFID Data");*/
        vTaskDelayUntil(&xLastWakeUpTime, 50 / portTICK_PERIOD_MS); // Polling delay
    }
}
