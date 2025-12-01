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
#include "drivers/ultrasonic/ultrasonic.h"

/******************************************************************************
 * Private macro definitions.
 ******************************************************************************/

// Each task is assigned a priority from 0 to ( configMAX_PRIORITIES - 1 ), where configMAX_PRIORITIES is defined within FreeRTOSConfig.h.
//  Low priority numbers denote low priority tasks. The idle task has priority zero (tskIDLE_PRIORITY).
#define mainLED_TASK_PRIORITY (tskIDLE_PRIORITY)

// tasks handler defined after the main
static void vReadRfid(void *pvParameters);
static void vUltrasonicTask(void *pvParameters);

// constant to ease the reading....
/*const uint8_t redLed   = _BV(PD2);
const uint8_t greenLed = _BV(PD3);*/

static RFID_Reader rfid(2, 3);
static uint8_t buffer[16];
static lcd_t lcd;

int main(void)
{


    // Initialize the LCD
    lcd_init(&lcd);
    lcd_begin(&lcd, 16, 2, LCD_2LINE);

    lcd_set_cursor(&lcd, 0, 0);
    lcd_print(&lcd, "Hello World!");


    rfid.begin(9600);

    xTaskCreate(
        vReadRfid,
        "rfid",
        configMINIMAL_STACK_SIZE,
        NULL,
        1U,
        NULL);

    xTaskCreate(
        vUltrasonicTask,
        "ultrasonic",
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
    while (1)
    {
        if (rfid.dataAvailable())
        {
            size_t length = rfid.readData(buffer, sizeof(buffer)-1);
            lcd_set_cursor(&lcd, 0, 1);
            buffer[length] = '\0';
            lcd_print(&lcd, buffer);
        }
        else {
            lcd_set_cursor(&lcd, 0, 1);
            lcd_print(&lcd, "No RFID Data");
        }
        vTaskDelayUntil(&xLastWakeUpTime, 100 / portTICK_PERIOD_MS); // Polling delay
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
        lcd_clear(&lcd);
        lcd_set_cursor(&lcd, 0, 0);
        lcd_print(&lcd, buffer);
        // Here you can add code to display the distance on the LCD or process it further
        vTaskDelayUntil(&xLastWakeUpTime, 1000 / portTICK_PERIOD_MS); // Polling delay
    }
}
