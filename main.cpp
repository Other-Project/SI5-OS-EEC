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

// Tasks
static void vReadRfid(void *pvParameters);
static void vBuzzerTask(void *pvParameters);
static void vUltrasonicTask(void *pvParameters);
static void vRotaryAngleTask(void *pvParameters);
static void vI2CUpdateTask(void *pvParameters);

// Peripherals
static RFID_Reader rfid(7, 8);
static Buzzer grooveBuzzer(&DDRD, &PORTD, _BV(PD6));
static Buzzer led(&DDRD, &PORTD, _BV(PD5));
static Button myButton(2);
static RotaryAngle rotaryAngle(0);
static uint8_t buffer[16];

// I2C callbacks to react to commands from the Raspberry Pi
void onBuzzerCommand(uint8_t reg, uint8_t value) {
    if (value) {
        grooveBuzzer.on();
    } else {
        grooveBuzzer.off();
    }
}

void onLedCommand(uint8_t reg, uint8_t value) {
    if (value) {
        led.on();
    } else {
        led.off();
    }
}

void onAlarmCommand(uint8_t reg, uint8_t value) {
    // Update alarm state
    I2C_Protocol::setRegister(REG_ALARM_STATE, value);
}

void onI2CCommand(uint8_t reg, uint8_t value) {
    switch (reg)
    {
    case REG_BUZZER_CMD:
        onBuzzerCommand(reg, value);
        break;
    case REG_LED_CMD:
        onLedCommand(reg, value);
        break;
    case REG_ALARM_STATE:
        onAlarmCommand(reg, value);
        break;
    default:
        break;
    }
}

int main(void) {
    // Initialize Wire (required before I2C_Protocol)
    Wire.begin();
    
    // Initialize I2C in slave mode (address 0x32)
    I2C_Protocol::init(0x32);
    
    // Register callbacks for commands coming from the Raspberry Pi
    I2C_Protocol::registerCallback(onI2CCommand);
    
    // Initialize peripherals
    led.init();
    grooveBuzzer.init();
    myButton.init();
    rfid.begin(9600);
    rotaryAngle.init();
    
    // Create tasks
    //xTaskCreate(vReadRfid, "rfid", configMINIMAL_STACK_SIZE + 50, NULL, 2U, NULL);
    //xTaskCreate(vBuzzerTask, "buzzer", configMINIMAL_STACK_SIZE, NULL, 1U, NULL);
    xTaskCreate(vUltrasonicTask, "ultrasonic", configMINIMAL_STACK_SIZE, NULL, 1U, NULL);
    //xTaskCreate(vRotaryAngleTask, "rotary", configMINIMAL_STACK_SIZE, NULL, 1U, NULL);
    //xTaskCreate(vI2CUpdateTask, "i2c_update", configMINIMAL_STACK_SIZE, NULL, 1U, NULL);
    
    // Start scheduler
    vTaskStartScheduler();
    
    return 0;
}

// RFID task - reads tags and updates I2C registers
static void vReadRfid(void *pvParameters) {
    TickType_t xLastWakeUpTime = xTaskGetTickCount();
    size_t length;
    
    while (1) {
        if (rfid.dataAvailable()) {
            buffer[0] = '\0';
            length = rfid.readData(buffer, sizeof(buffer) - 1);
            
            if (length >= 10) {
                // Tag detected
                I2C_Protocol::setRegister(REG_RFID_STATUS, 1);
                
                // Copy the tag ID into the registers (max 8 bytes)
                for (uint8_t i = 0; i < 8 && (i + 1) < length; i++) {
                    I2C_Protocol::setRegister(REG_RFID_ID_0 + i, buffer[i + 1]);
                }
                
                vTaskDelayUntil(&xLastWakeUpTime, 2000 / portTICK_PERIOD_MS);
            }
        } else {
            // No tag
            I2C_Protocol::setRegister(REG_RFID_STATUS, 0);
        }
        
        vTaskDelayUntil(&xLastWakeUpTime, 100 / portTICK_PERIOD_MS);
    }
}

// Ultrasonic task - measures distance and detects motion
static void vUltrasonicTask(void *pvParameters) {
    Ultrasonic ultrasonic(&PORTD, &DDRD, &PIND, PD4);
    TickType_t xLastWakeUpTime = xTaskGetTickCount();
    
    while (1) {
        long distance_mm = ultrasonic.MeasureInMillimeters();
        
        // Store distance in 2 registers (16 bits)
        I2C_Protocol::setRegister(REG_DISTANCE_H, (distance_mm >> 8) & 0xFF);
        I2C_Protocol::setRegister(REG_DISTANCE_L, distance_mm & 0xFF);
        
        // Motion detection (distance < 1000mm)
        if (distance_mm < 1000 && distance_mm > 0) {
            I2C_Protocol::setRegister(REG_MOTION_DETECTED, 1);
        } else {
            I2C_Protocol::setRegister(REG_MOTION_DETECTED, 0);
        }
        
        vTaskDelayUntil(&xLastWakeUpTime, 200 / portTICK_PERIOD_MS);
    }
}

// Button task - toggles the alarm
static void vBuzzerTask(void *pvParameters) {
    while (1) {
        // Wait for the button press
        myButton.waitForPress();
        
        // Toggle alarm state
        uint8_t current_state = I2C_Protocol::getRegister(REG_ALARM_STATE);
        I2C_Protocol::setRegister(REG_ALARM_STATE, !current_state);
        
        // Confirmation beep
        grooveBuzzer.on();
        vTaskDelay(100 / portTICK_PERIOD_MS);
        grooveBuzzer.off();
    }
}

// Potentiometer task - reads the angle
static void vRotaryAngleTask(void *pvParameters) {
    TickType_t xLastWakeUpTime = xTaskGetTickCount();
    
    while (1) {
        long angle = rotaryAngle.readDegrees();
        
        // Store the angle in a register (0-255 for 0-300°)
        uint8_t angle_byte = (angle * 255) / 300;
        I2C_Protocol::setRegister(REG_ROTARY_ANGLE, angle_byte);
        
        vTaskDelayUntil(&xLastWakeUpTime, 200 / portTICK_PERIOD_MS);
    }
}

// I2C update task - handles alarm logic
static void vI2CUpdateTask(void *pvParameters) {
    TickType_t xLastWakeUpTime = xTaskGetTickCount();
    
    while (1) {
        // Read alarm state
        uint8_t alarm_state = I2C_Protocol::getRegister(REG_ALARM_STATE);
        uint8_t motion = I2C_Protocol::getRegister(REG_MOTION_DETECTED);
        
        // If alarm enabled AND motion detected -> trigger buzzer
        if (alarm_state && motion) {
            I2C_Protocol::setRegister(REG_BUZZER_CMD, 1);
            grooveBuzzer.on();
        } else {
            I2C_Protocol::setRegister(REG_BUZZER_CMD, 0);
            grooveBuzzer.off();
        }
        
        // Update status LED
        if (alarm_state) {
            led.on();
        } else {
            led.off();
        }
        
        I2C_Protocol::setRegister(REG_STATUS, 0x01); // System OK
        
        vTaskDelayUntil(&xLastWakeUpTime, 50 / portTICK_PERIOD_MS);
    }
}
