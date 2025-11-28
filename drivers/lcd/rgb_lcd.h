/*
    rgb_lcd.h - Native AVR version
    Adapted from Seeed Technology Inc. Arduino library
    
    The MIT License (MIT)
*/

#ifndef __RGB_LCD_AVR_H__
#define __RGB_LCD_AVR_H__

#include <avr/io.h>
#include <util/delay.h>
#include <avr/pgmspace.h>

// Device I2C Address
#define LCD_ADDRESS     (0x7c>>1)
#define RGB_ADDRESS     (0xc4>>1)
#define RGB_ADDRESS_V5  (0x30)

// Color definitions
#define WHITE           0
#define RED             1
#define GREEN           2
#define BLUE            3

#define REG_MODE1       0x00
#define REG_MODE2       0x01
#define REG_OUTPUT      0x08

// LCD Commands
#define LCD_CLEARDISPLAY 0x01
#define LCD_RETURNHOME 0x02
#define LCD_ENTRYMODESET 0x04
#define LCD_DISPLAYCONTROL 0x08
#define LCD_CURSORSHIFT 0x10
#define LCD_FUNCTIONSET 0x20
#define LCD_SETCGRAMADDR 0x40
#define LCD_SETDDRAMADDR 0x80

// Flags for display entry mode
#define LCD_ENTRYRIGHT 0x00
#define LCD_ENTRYLEFT 0x02
#define LCD_ENTRYSHIFTINCREMENT 0x01
#define LCD_ENTRYSHIFTDECREMENT 0x00

// Flags for display on/off control
#define LCD_DISPLAYON 0x04
#define LCD_DISPLAYOFF 0x00
#define LCD_CURSORON 0x02
#define LCD_CURSOROFF 0x00
#define LCD_BLINKON 0x01
#define LCD_BLINKOFF 0x00

// Flags for display/cursor shift
#define LCD_DISPLAYMOVE 0x08
#define LCD_CURSORMOVE 0x00
#define LCD_MOVERIGHT 0x04
#define LCD_MOVELEFT 0x00

// Flags for function set
#define LCD_8BITMODE 0x10
#define LCD_4BITMODE 0x00
#define LCD_2LINE 0x08
#define LCD_1LINE 0x00
#define LCD_5x10DOTS 0x04
#define LCD_5x8DOTS 0x00

// LCD structure
typedef struct {
    uint8_t rgb_chip_addr;
    uint8_t display_function;
    uint8_t display_control;
    uint8_t display_mode;
    uint8_t initialized;
    uint8_t num_lines;
    uint8_t curr_line;
} rgb_lcd_t;

// Function prototypes
void rgb_lcd_init(rgb_lcd_t *lcd);
void rgb_lcd_begin(rgb_lcd_t *lcd, uint8_t cols, uint8_t rows, uint8_t charsize);
void rgb_lcd_clear(rgb_lcd_t *lcd);
void rgb_lcd_home(rgb_lcd_t *lcd);
void rgb_lcd_set_cursor(rgb_lcd_t *lcd, uint8_t col, uint8_t row);
void rgb_lcd_display(rgb_lcd_t *lcd);
void rgb_lcd_no_display(rgb_lcd_t *lcd);
void rgb_lcd_cursor(rgb_lcd_t *lcd);
void rgb_lcd_no_cursor(rgb_lcd_t *lcd);
void rgb_lcd_blink(rgb_lcd_t *lcd);
void rgb_lcd_no_blink(rgb_lcd_t *lcd);
void rgb_lcd_scroll_display_left(rgb_lcd_t *lcd);
void rgb_lcd_scroll_display_right(rgb_lcd_t *lcd);
void rgb_lcd_left_to_right(rgb_lcd_t *lcd);
void rgb_lcd_right_to_left(rgb_lcd_t *lcd);
void rgb_lcd_autoscroll(rgb_lcd_t *lcd);
void rgb_lcd_no_autoscroll(rgb_lcd_t *lcd);
void rgb_lcd_create_char(rgb_lcd_t *lcd, uint8_t location, uint8_t charmap[]);
void rgb_lcd_create_char_P(rgb_lcd_t *lcd, uint8_t location, const uint8_t *charmap);
void rgb_lcd_write(rgb_lcd_t *lcd, uint8_t value);
void rgb_lcd_print(rgb_lcd_t *lcd, const char *str);
void rgb_lcd_print_P(rgb_lcd_t *lcd, const char *str);
void rgb_lcd_set_rgb(rgb_lcd_t *lcd, uint8_t r, uint8_t g, uint8_t b);
void rgb_lcd_set_pwm(rgb_lcd_t *lcd, uint8_t color, uint8_t pwm);
void rgb_lcd_set_color(rgb_lcd_t *lcd, uint8_t color);
void rgb_lcd_set_color_white(rgb_lcd_t *lcd);
void rgb_lcd_set_color_all(rgb_lcd_t *lcd);
void rgb_lcd_blink_led(rgb_lcd_t *lcd);
void rgb_lcd_no_blink_led(rgb_lcd_t *lcd);

// I2C/TWI functions
/*void twi_init(void);
void twi_start(void);
void twi_stop(void);
void twi_write(uint8_t data);
uint8_t twi_get_status(void);*/

#endif