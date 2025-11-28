/*
    lcd.h - Native AVR version
    Adapted from Seeed Technology Inc. Arduino library
    
    The MIT License (MIT)
*/

#ifndef __lcd_AVR_H__
#define __lcd_AVR_H__

#include <avr/io.h>
#include <util/delay.h>
#include <avr/pgmspace.h>

// Device I2C Address
#define LCD_ADDRESS     (0x7c>>1)

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
    uint8_t display_function;
    uint8_t display_control;
    uint8_t display_mode;
    uint8_t initialized;
    uint8_t num_lines;
    uint8_t curr_line;
} lcd_t;

// Function prototypes
void lcd_init(lcd_t *lcd);
void lcd_begin(lcd_t *lcd, uint8_t cols, uint8_t rows, uint8_t charsize);
void lcd_clear(lcd_t *lcd);
void lcd_home(lcd_t *lcd);
void lcd_set_cursor(lcd_t *lcd, uint8_t col, uint8_t row);
void lcd_display(lcd_t *lcd);
void lcd_no_display(lcd_t *lcd);
void lcd_cursor(lcd_t *lcd);
void lcd_no_cursor(lcd_t *lcd);
void lcd_blink(lcd_t *lcd);
void lcd_no_blink(lcd_t *lcd);
void lcd_scroll_display_left(lcd_t *lcd);
void lcd_scroll_display_right(lcd_t *lcd);
void lcd_left_to_right(lcd_t *lcd);
void lcd_right_to_left(lcd_t *lcd);
void lcd_autoscroll(lcd_t *lcd);
void lcd_no_autoscroll(lcd_t *lcd);
void lcd_create_char(lcd_t *lcd, uint8_t location, uint8_t charmap[]);
void lcd_create_char_P(lcd_t *lcd, uint8_t location, const uint8_t *charmap);
void lcd_write(lcd_t *lcd, uint8_t value);
void lcd_print(lcd_t *lcd, const char *str);
void lcd_print_P(lcd_t *lcd, const char *str);

#endif