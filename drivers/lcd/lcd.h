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

/* Initialize lcd structure and TWI (I2C) peripheral. */
void lcd_init(lcd_t *lcd);

/*
 * Begin using the LCD: set columns, rows and character size.
 * Performs HD44780 initialization sequence and prepares the display.
 */
void lcd_begin(lcd_t *lcd, uint8_t cols, uint8_t rows, uint8_t charsize);

/* Clear the display and set cursor to home (0,0). */
void lcd_clear(lcd_t *lcd);

/* Return cursor to home position without clearing display. */
void lcd_home(lcd_t *lcd);

/* Set cursor to given column and row (0-based). */
void lcd_set_cursor(lcd_t *lcd, uint8_t col, uint8_t row);

/* Turn the LCD display on (but keeps cursor/blink settings). */
void lcd_display(lcd_t *lcd);

/* Turn the LCD display off (screen goes blank). */
void lcd_no_display(lcd_t *lcd);

/* Enable the text cursor (visible). */
void lcd_cursor(lcd_t *lcd);

/* Disable the text cursor (invisible). */
void lcd_no_cursor(lcd_t *lcd);

/* Enable blinking block cursor. */
void lcd_blink(lcd_t *lcd);

/* Disable blinking block cursor. */
void lcd_no_blink(lcd_t *lcd);

/* Scroll the whole display left by one position. */
void lcd_scroll_display_left(lcd_t *lcd);

/* Scroll the whole display right by one position. */
void lcd_scroll_display_right(lcd_t *lcd);

/* Set text direction left-to-right. */
void lcd_left_to_right(lcd_t *lcd);

/* Set text direction right-to-left. */
void lcd_right_to_left(lcd_t *lcd);

/* Enable automatic display shifting (autoscroll). */
void lcd_autoscroll(lcd_t *lcd);

/* Disable automatic display shifting (autoscroll). */
void lcd_no_autoscroll(lcd_t *lcd);

/* Create a custom character from RAM-provided byte map (8 bytes). */
void lcd_create_char(lcd_t *lcd, uint8_t location, uint8_t charmap[]);

/* Create a custom character from PROGMEM-provided byte map (8 bytes). */
void lcd_create_char_P(lcd_t *lcd, uint8_t location, const uint8_t *charmap);

/* Write a single byte/character to the current cursor position. */
void lcd_write(lcd_t *lcd, uint8_t value);

/* Write a NUL-terminated C string from RAM to the display. */
void lcd_print(lcd_t *lcd, const char *str);

/* Write a NUL-terminated string stored in PROGMEM to the display. */
void lcd_print_P(lcd_t *lcd, const char *str);

#endif