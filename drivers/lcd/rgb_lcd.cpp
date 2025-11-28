/*
    rgb_lcd.c - Native AVR version
    Adapted from Seeed Technology Inc. Arduino library
    
    Configure F_CPU before including this file or in makefile
    Example: #define F_CPU 16000000UL
    
    The MIT License (MIT)
*/

#include "rgb_lcd.h"
#include <string.h>
//#include "../../arduinoLibsAndCore/libraries/Wire/src/Wire.h"

// TWI Status codes
#define TW_START        0x08
#define TW_REP_START    0x10
#define TW_MT_SLA_ACK   0x18
#define TW_MT_SLA_NACK  0x20
#define TW_MT_DATA_ACK  0x28
#define TW_MT_DATA_NACK 0x30
#define TW_STATUS_MASK  0xF8

// Color lookup table
static const uint8_t color_define[4][3] PROGMEM = {
    {255, 255, 255},  // white
    {255, 0, 0},      // red
    {0, 255, 0},      // green
    {0, 0, 255}       // blue
};

// Internal helper functions
static void lcd_command(rgb_lcd_t *lcd, uint8_t value);
static void lcd_set_reg(rgb_lcd_t *lcd, uint8_t reg, uint8_t dat);
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

static void lcd_command(rgb_lcd_t *lcd, uint8_t value) {
    if (!lcd->initialized) return;
    
    uint8_t dta[2] = {0x80, value};
    i2c_send_bytes(LCD_ADDRESS, dta, 2);
}

static void lcd_set_reg(rgb_lcd_t *lcd, uint8_t reg, uint8_t dat) {
    if (!lcd->initialized) return;
    
    lcd_twi_start();
    lcd_twi_write((lcd->rgb_chip_addr << 1) | 0);
    lcd_twi_write(reg);
    lcd_twi_write(dat);
    lcd_twi_stop();
}

// ========== Public LCD Functions ==========

void rgb_lcd_init(rgb_lcd_t *lcd) {
    memset(lcd, 0, sizeof(rgb_lcd_t));
    lcd_twi_init();
}

void rgb_lcd_begin(rgb_lcd_t *lcd, uint8_t cols, uint8_t rows, uint8_t charsize) {
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
    rgb_lcd_display(lcd);
    
    // Clear display
    rgb_lcd_clear(lcd);
    
    // Set default text direction (left to right)
    lcd->display_mode = LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT;
    lcd_command(lcd, LCD_ENTRYMODESET | lcd->display_mode);
    
    // Detect RGB chip version
    lcd_twi_start();
    lcd_twi_write((RGB_ADDRESS_V5 << 1) | 0);
    uint8_t status = lcd_twi_get_status();
    lcd_twi_stop();
    
    if (status == TW_MT_SLA_ACK) {
        // V5 chip detected
        lcd->rgb_chip_addr = RGB_ADDRESS_V5;
        lcd_set_reg(lcd, 0x00, 0x07);  // Reset chip
        _delay_us(200);
        lcd_set_reg(lcd, 0x04, 0x15);  // Set LEDs always on
    } else {
        // Original chip
        lcd->rgb_chip_addr = RGB_ADDRESS;
        lcd_set_reg(lcd, REG_MODE1, 0);
        lcd_set_reg(lcd, REG_OUTPUT, 0xFF);
        lcd_set_reg(lcd, REG_MODE2, 0x20);
    }
    
    rgb_lcd_set_color_white(lcd);
}

void rgb_lcd_clear(rgb_lcd_t *lcd) {
    lcd_command(lcd, LCD_CLEARDISPLAY);
    _delay_ms(2);
}

void rgb_lcd_home(rgb_lcd_t *lcd) {
    lcd_command(lcd, LCD_RETURNHOME);
    _delay_ms(2);
}

void rgb_lcd_set_cursor(rgb_lcd_t *lcd, uint8_t col, uint8_t row) {
    if (!lcd->initialized) return;
    
    col = (row == 0 ? col | 0x80 : col | 0xc0);
    uint8_t dta[2] = {0x80, col};
    i2c_send_bytes(LCD_ADDRESS, dta, 2);
}

void rgb_lcd_no_display(rgb_lcd_t *lcd) {
    lcd->display_control &= ~LCD_DISPLAYON;
    lcd_command(lcd, LCD_DISPLAYCONTROL | lcd->display_control);
}

void rgb_lcd_display(rgb_lcd_t *lcd) {
    lcd->display_control |= LCD_DISPLAYON;
    lcd_command(lcd, LCD_DISPLAYCONTROL | lcd->display_control);
}

void rgb_lcd_no_cursor(rgb_lcd_t *lcd) {
    lcd->display_control &= ~LCD_CURSORON;
    lcd_command(lcd, LCD_DISPLAYCONTROL | lcd->display_control);
}

void rgb_lcd_cursor(rgb_lcd_t *lcd) {
    lcd->display_control |= LCD_CURSORON;
    lcd_command(lcd, LCD_DISPLAYCONTROL | lcd->display_control);
}

void rgb_lcd_no_blink(rgb_lcd_t *lcd) {
    lcd->display_control &= ~LCD_BLINKON;
    lcd_command(lcd, LCD_DISPLAYCONTROL | lcd->display_control);
}

void rgb_lcd_blink(rgb_lcd_t *lcd) {
    lcd->display_control |= LCD_BLINKON;
    lcd_command(lcd, LCD_DISPLAYCONTROL | lcd->display_control);
}

void rgb_lcd_scroll_display_left(rgb_lcd_t *lcd) {
    lcd_command(lcd, LCD_CURSORSHIFT | LCD_DISPLAYMOVE | LCD_MOVELEFT);
}

void rgb_lcd_scroll_display_right(rgb_lcd_t *lcd) {
    lcd_command(lcd, LCD_CURSORSHIFT | LCD_DISPLAYMOVE | LCD_MOVERIGHT);
}

void rgb_lcd_left_to_right(rgb_lcd_t *lcd) {
    lcd->display_mode |= LCD_ENTRYLEFT;
    lcd_command(lcd, LCD_ENTRYMODESET | lcd->display_mode);
}

void rgb_lcd_right_to_left(rgb_lcd_t *lcd) {
    lcd->display_mode &= ~LCD_ENTRYLEFT;
    lcd_command(lcd, LCD_ENTRYMODESET | lcd->display_mode);
}

void rgb_lcd_autoscroll(rgb_lcd_t *lcd) {
    lcd->display_mode |= LCD_ENTRYSHIFTINCREMENT;
    lcd_command(lcd, LCD_ENTRYMODESET | lcd->display_mode);
}

void rgb_lcd_no_autoscroll(rgb_lcd_t *lcd) {
    lcd->display_mode &= ~LCD_ENTRYSHIFTINCREMENT;
    lcd_command(lcd, LCD_ENTRYMODESET | lcd->display_mode);
}

void rgb_lcd_create_char(rgb_lcd_t *lcd, uint8_t location, uint8_t charmap[]) {
    location &= 0x7;  // Only 8 locations 0-7
    lcd_command(lcd, LCD_SETCGRAMADDR | (location << 3));
    
    uint8_t dta[9];
    dta[0] = 0x40;
    for (uint8_t i = 0; i < 8; i++) {
        dta[i + 1] = charmap[i];
    }
    i2c_send_bytes(LCD_ADDRESS, dta, 9);
}

void rgb_lcd_create_char_P(rgb_lcd_t *lcd, uint8_t location, const uint8_t *charmap) {
    location &= 0x7;  // Only 8 locations 0-7
    lcd_command(lcd, LCD_SETCGRAMADDR | (location << 3));
    
    uint8_t dta[9];
    dta[0] = 0x40;
    for (uint8_t i = 0; i < 8; i++) {
        dta[i + 1] = pgm_read_byte(&charmap[i]);
    }
    i2c_send_bytes(LCD_ADDRESS, dta, 9);
}

void rgb_lcd_write(rgb_lcd_t *lcd, uint8_t value) {
    if (!lcd->initialized) return;
    
    uint8_t dta[2] = {0x40, value};
    i2c_send_bytes(LCD_ADDRESS, dta, 2);
}

void rgb_lcd_print(rgb_lcd_t *lcd, const char *str) {
    while (*str) {
        rgb_lcd_write(lcd, *str++);
    }
}

void rgb_lcd_print_P(rgb_lcd_t *lcd, const char *str) {
    char c;
    while ((c = pgm_read_byte(str++))) {
        rgb_lcd_write(lcd, c);
    }
}

void rgb_lcd_set_rgb(rgb_lcd_t *lcd, uint8_t r, uint8_t g, uint8_t b) {
    if (lcd->rgb_chip_addr == RGB_ADDRESS_V5) {
        lcd_set_reg(lcd, 0x06, r);
        lcd_set_reg(lcd, 0x07, g);
        lcd_set_reg(lcd, 0x08, b);
    } else {
        lcd_set_reg(lcd, 0x04, r);
        lcd_set_reg(lcd, 0x03, g);
        lcd_set_reg(lcd, 0x02, b);
    }
}

void rgb_lcd_set_pwm(rgb_lcd_t *lcd, uint8_t color, uint8_t pwm) {
    switch (color) {
        case WHITE:
            rgb_lcd_set_rgb(lcd, pwm, pwm, pwm);
            break;
        case RED:
            rgb_lcd_set_rgb(lcd, pwm, 0, 0);
            break;
        case GREEN:
            rgb_lcd_set_rgb(lcd, 0, pwm, 0);
            break;
        case BLUE:
            rgb_lcd_set_rgb(lcd, 0, 0, pwm);
            break;
    }
}

void rgb_lcd_set_color(rgb_lcd_t *lcd, uint8_t color) {
    if (color > 3) return;
    
    uint8_t r = pgm_read_byte(&color_define[color][0]);
    uint8_t g = pgm_read_byte(&color_define[color][1]);
    uint8_t b = pgm_read_byte(&color_define[color][2]);
    
    rgb_lcd_set_rgb(lcd, r, g, b);
}

void rgb_lcd_set_color_white(rgb_lcd_t *lcd) {
    rgb_lcd_set_rgb(lcd, 255, 255, 255);
}

void rgb_lcd_set_color_all(rgb_lcd_t *lcd) {
    rgb_lcd_set_rgb(lcd, 0, 0, 0);
}

void rgb_lcd_blink_led(rgb_lcd_t *lcd) {
    if (lcd->rgb_chip_addr == RGB_ADDRESS_V5) {
        lcd_set_reg(lcd, 0x04, 0x2a);  // Attach all LEDs to PWM1
        lcd_set_reg(lcd, 0x01, 0x06);  // Blink every second
        lcd_set_reg(lcd, 0x02, 0x7f);  // 50% duty cycle
    } else {
        lcd_set_reg(lcd, 0x07, 0x17);  // Blink every second
        lcd_set_reg(lcd, 0x06, 0x7f);  // 50% duty cycle
    }
}

void rgb_lcd_no_blink_led(rgb_lcd_t *lcd) {
    if (lcd->rgb_chip_addr == RGB_ADDRESS_V5) {
        lcd_set_reg(lcd, 0x04, 0x15);
    } else {
        lcd_set_reg(lcd, 0x07, 0x00);
        lcd_set_reg(lcd, 0x06, 0xff);
    }
}