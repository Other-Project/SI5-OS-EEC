/*
    lcd.c - Native AVR version
    Adapted from Seeed Technology Inc. Arduino library
    
    The MIT License (MIT)
*/

#include "lcd.h"
#include <string.h>

// TWI Status
#define TW_STATUS_MASK  0xF8

// Internal helper functions
static void lcd_command(lcd_t *lcd, uint8_t value);
static void i2c_send_byte(uint8_t addr, uint8_t dta);
static void i2c_send_bytes(uint8_t addr, uint8_t *dta, uint8_t len);

// ========== TWI/I2C Implementation ==========

void lcd_twi_init(void) {
    // Set SCL frequency = F_CPU / (16 + 2 * TWBR * prescaler)
    // For 100kHz with F_CPU = 16MHz: TWBR = 72, prescaler = 1
    #if F_CPU == 16000000UL
        TWBR = 72;
    #elif F_CPU == 8000000UL
        TWBR = 32;
    #else
        TWBR = ((F_CPU / 100000UL) - 16) / 2;
    #endif
    
    TWSR = 0x00;  // Prescaler = 1
    TWCR = (1 << TWEN);  // Enable TWI
}

void lcd_twi_start(void) {
    TWCR = (1 << TWINT) | (1 << TWSTA) | (1 << TWEN);
    while (!(TWCR & (1 << TWINT)));  // Wait for TWINT flag
}

void lcd_twi_stop(void) {
    TWCR = (1 << TWINT) | (1 << TWSTO) | (1 << TWEN);
    while (TWCR & (1 << TWSTO));  // Wait for stop condition
}

void lcd_twi_write(uint8_t data) {
    TWDR = data;
    TWCR = (1 << TWINT) | (1 << TWEN);
    while (!(TWCR & (1 << TWINT)));  // Wait for TWINT flag
}

uint8_t lcd_twi_get_status(void) {
    return (TWSR & TW_STATUS_MASK);
}

// ========== I2C Helper Functions ==========

static void i2c_send_byte(uint8_t addr, uint8_t dta) {
    lcd_twi_start();
    lcd_twi_write((addr << 1) | 0);  // Address + write bit
    lcd_twi_write(dta);
    lcd_twi_stop();
}

static void i2c_send_bytes(uint8_t addr, uint8_t *dta, uint8_t len) {
    lcd_twi_start();
    lcd_twi_write((addr << 1) | 0);  // Address + write bit
    for (uint8_t i = 0; i < len; i++) {
        lcd_twi_write(dta[i]);
    }
    lcd_twi_stop();
}

// ========== LCD Internal Functions ==========

static void lcd_command(lcd_t *lcd, uint8_t value) {
    if (!lcd->initialized) return;
    
    uint8_t dta[2] = {0x80, value};
    i2c_send_bytes(LCD_ADDRESS, dta, 2);
}

// ========== Public LCD Functions ==========

void lcd_init(lcd_t *lcd) {
    memset(lcd, 0, sizeof(lcd_t));
    lcd_twi_init();
}

void lcd_begin(lcd_t *lcd, uint8_t cols, uint8_t rows, uint8_t charsize) {
    lcd->initialized = 1;
    
    if (rows > 1) {
        lcd->display_function |= LCD_2LINE;
    }
    lcd->num_lines = rows;
    lcd->curr_line = 0;
    
    // For some 1 line displays you can select a 10 pixel high font
    if ((charsize != 0) && (rows == 1)) {
        lcd->display_function |= LCD_5x10DOTS;
    }
    
    // Wait for LCD to power up (>40ms after Vcc rises above 2.7V)
    _delay_ms(50);
    
    // Initialization sequence according to HD44780 datasheet
    lcd_command(lcd, LCD_FUNCTIONSET | lcd->display_function);
    _delay_us(4500);
    
    lcd_command(lcd, LCD_FUNCTIONSET | lcd->display_function);
    _delay_us(150);
    
    lcd_command(lcd, LCD_FUNCTIONSET | lcd->display_function);
    lcd_command(lcd, LCD_FUNCTIONSET | lcd->display_function);
    
    // Turn display on with no cursor or blinking
    lcd->display_control = LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF;
    lcd_display(lcd);
    
    // Clear display
    lcd_clear(lcd);
    
    // Set default text direction (left to right)
    lcd->display_mode = LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT;
    lcd_command(lcd, LCD_ENTRYMODESET | lcd->display_mode);
}

void lcd_clear(lcd_t *lcd) {
    lcd_command(lcd, LCD_CLEARDISPLAY);
    _delay_ms(2);
}

void lcd_home(lcd_t *lcd) {
    lcd_command(lcd, LCD_RETURNHOME);
    _delay_ms(2);
}

void lcd_set_cursor(lcd_t *lcd, uint8_t col, uint8_t row) {
    if (!lcd->initialized) return;
    
    col = (row == 0 ? col | 0x80 : col | 0xc0);
    uint8_t dta[2] = {0x80, col};
    i2c_send_bytes(LCD_ADDRESS, dta, 2);
}

void lcd_no_display(lcd_t *lcd) {
    lcd->display_control &= ~LCD_DISPLAYON;
    lcd_command(lcd, LCD_DISPLAYCONTROL | lcd->display_control);
}

void lcd_display(lcd_t *lcd) {
    lcd->display_control |= LCD_DISPLAYON;
    lcd_command(lcd, LCD_DISPLAYCONTROL | lcd->display_control);
}

void lcd_no_cursor(lcd_t *lcd) {
    lcd->display_control &= ~LCD_CURSORON;
    lcd_command(lcd, LCD_DISPLAYCONTROL | lcd->display_control);
}

void lcd_cursor(lcd_t *lcd) {
    lcd->display_control |= LCD_CURSORON;
    lcd_command(lcd, LCD_DISPLAYCONTROL | lcd->display_control);
}

void lcd_no_blink(lcd_t *lcd) {
    lcd->display_control &= ~LCD_BLINKON;
    lcd_command(lcd, LCD_DISPLAYCONTROL | lcd->display_control);
}

void lcd_blink(lcd_t *lcd) {
    lcd->display_control |= LCD_BLINKON;
    lcd_command(lcd, LCD_DISPLAYCONTROL | lcd->display_control);
}

void lcd_scroll_display_left(lcd_t *lcd) {
    lcd_command(lcd, LCD_CURSORSHIFT | LCD_DISPLAYMOVE | LCD_MOVELEFT);
}

void lcd_scroll_display_right(lcd_t *lcd) {
    lcd_command(lcd, LCD_CURSORSHIFT | LCD_DISPLAYMOVE | LCD_MOVERIGHT);
}

void lcd_left_to_right(lcd_t *lcd) {
    lcd->display_mode |= LCD_ENTRYLEFT;
    lcd_command(lcd, LCD_ENTRYMODESET | lcd->display_mode);
}

void lcd_right_to_left(lcd_t *lcd) {
    lcd->display_mode &= ~LCD_ENTRYLEFT;
    lcd_command(lcd, LCD_ENTRYMODESET | lcd->display_mode);
}

void lcd_autoscroll(lcd_t *lcd) {
    lcd->display_mode |= LCD_ENTRYSHIFTINCREMENT;
    lcd_command(lcd, LCD_ENTRYMODESET | lcd->display_mode);
}

void lcd_no_autoscroll(lcd_t *lcd) {
    lcd->display_mode &= ~LCD_ENTRYSHIFTINCREMENT;
    lcd_command(lcd, LCD_ENTRYMODESET | lcd->display_mode);
}

void lcd_create_char(lcd_t *lcd, uint8_t location, uint8_t charmap[]) {
    location &= 0x7;  // Only 8 locations 0-7
    lcd_command(lcd, LCD_SETCGRAMADDR | (location << 3));
    
    uint8_t dta[9];
    dta[0] = 0x40;
    for (uint8_t i = 0; i < 8; i++) {
        dta[i + 1] = charmap[i];
    }
    i2c_send_bytes(LCD_ADDRESS, dta, 9);
}

void lcd_create_char_P(lcd_t *lcd, uint8_t location, const uint8_t *charmap) {
    location &= 0x7;  // Only 8 locations 0-7
    lcd_command(lcd, LCD_SETCGRAMADDR | (location << 3));
    
    uint8_t dta[9];
    dta[0] = 0x40;
    for (uint8_t i = 0; i < 8; i++) {
        dta[i + 1] = pgm_read_byte(&charmap[i]);
    }
    i2c_send_bytes(LCD_ADDRESS, dta, 9);
}

void lcd_write(lcd_t *lcd, uint8_t value) {
    if (!lcd->initialized) return;
    
    uint8_t dta[2] = {0x40, value};
    i2c_send_bytes(LCD_ADDRESS, dta, 2);
}

void lcd_print(lcd_t *lcd, const char *str) {
    while (*str) {
        lcd_write(lcd, *str++);
    }
}

void lcd_print_P(lcd_t *lcd, const char *str) {
    char c;
    while ((c = pgm_read_byte(str++))) {
        lcd_write(lcd, c);
    }
}