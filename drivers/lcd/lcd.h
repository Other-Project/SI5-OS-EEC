/*
    lcd.h - Native AVR version
    Adapted from Seeed Technology Inc. Arduino library

    The MIT License (MIT)
*/

#ifndef __lcd_H__
#define __lcd_H__

#include <avr/io.h>
#include <util/delay.h>
#include <avr/pgmspace.h>

// Device I2C Address
#define LCD_ADDRESS (0x7c >> 1)

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

// LCD class
class LCD
{

private:
    uint8_t display_function;
    uint8_t display_control;
    uint8_t display_mode;
    uint8_t initialized;
    uint8_t num_lines;
    uint8_t curr_line;

    void twi_init(void);
    void twi_start(void);
    void twi_stop(void);
    void twi_write(uint8_t data);
    uint8_t twi_get_status(void);
    void i2c_send_byte(uint8_t addr, uint8_t dta);
    void LCD::i2c_send_bytes(uint8_t addr, uint8_t *dta, uint8_t len);

    void command(uint8_t value);

public:
    /* Initialize lcd structure and TWI (I2C) peripheral. */
    LCD() : display_function(0),
            display_control(0),
            display_mode(0),
            initialized(0),
            num_lines(0),
            curr_line(0)
    {
        twi_init();
    }

    /*
     * Begin using the LCD: set columns, rows and character size.
     * Performs HD44780 initialization sequence and prepares the display.
     */
    void begin(uint8_t cols, uint8_t rows, uint8_t charsize);

    /* Clear the display and set cursor to home (0,0). */
    void clear();

    /* Return cursor to home position without clearing display. */
    void home();

    /* Set cursor to given column and row (0-based). */
    void set_cursor(uint8_t col, uint8_t row);

    /* Turn the LCD display on (but keeps cursor/blink settings). */
    void display();

    /* Turn the LCD display off (screen goes blank). */
    void no_display();

    /* Enable the text cursor (visible). */
    void cursor();

    /* Disable the text cursor (invisible). */
    void no_cursor();

    /* Enable blinking block cursor. */
    void blink();

    /* Disable blinking block cursor. */
    void no_blink();

    /* Scroll the whole display left by one position. */
    void scroll_display_left();

    /* Scroll the whole display right by one position. */
    void scroll_display_right();

    /* Set text direction left-to-right. */
    void left_to_right();

    /* Set text direction right-to-left. */
    void right_to_left();

    /* Enable automatic display shifting (autoscroll). */
    void autoscroll();

    /* Disable automatic display shifting (autoscroll). */
    void no_autoscroll();

    /* Create a custom character from RAM-provided byte map (8 bytes). */
    void create_char(uint8_t location, uint8_t charmap[]);

    /* Create a custom character from PROGMEM-provided byte map (8 bytes). */
    void create_char_P(uint8_t location, const uint8_t *charmap);

    /* Write a single byte/character to the current cursor position. */
    void write(uint8_t value);

    /* Write a NUL-terminated C string from RAM to the display. */
    void print(const unsigned char *str);

    /* Write a NUL-terminated string stored in PROGMEM to the display. */
    void print_P(const unsigned char *str);
};

#endif
