/*
    lcd.c - Native AVR version
    Adapted from Seeed Technology Inc. Arduino library

    The MIT License (MIT)
*/

#include "lcd.h"
#include <string.h>

// TWI Status
#define TW_STATUS_MASK 0xF8

// Internal helper functions
static void command(uint8_t value);
static void i2c_send_byte(uint8_t addr, uint8_t dta);
static void i2c_send_bytes(uint8_t addr, uint8_t *dta, uint8_t len);

// ========== TWI/I2C Implementation ==========

void LCD::twi_init(void)
{
// Set SCL frequency = F_CPU / (16 + 2 * TWBR * prescaler)
// For 100kHz with F_CPU = 16MHz: TWBR = 72, prescaler = 1
#if F_CPU == 16000000UL
    TWBR = 72;
#elif F_CPU == 8000000UL
    TWBR = 32;
#else
    TWBR = ((F_CPU / 100000UL) - 16) / 2;
#endif

    TWSR = 0x00;        // Prescaler = 1
    TWCR = (1 << TWEN); // Enable TWI
}

void LCD::twi_start(void)
{
    TWCR = (1 << TWINT) | (1 << TWSTA) | (1 << TWEN);
    while (!(TWCR & (1 << TWINT)))
        ; // Wait for TWINT flag
}

void LCD::twi_stop(void)
{
    TWCR = (1 << TWINT) | (1 << TWSTO) | (1 << TWEN);
    while (TWCR & (1 << TWSTO))
        ; // Wait for stop condition
}

void LCD::twi_write(uint8_t data)
{
    TWDR = data;
    TWCR = (1 << TWINT) | (1 << TWEN);
    while (!(TWCR & (1 << TWINT)))
        ; // Wait for TWINT flag
}

uint8_t LCD::twi_get_status(void)
{
    return (TWSR & TW_STATUS_MASK);
}

// ========== I2C Helper Functions ==========

void LCD::i2c_send_byte(uint8_t addr, uint8_t dta)
{
    twi_start();
    twi_write((addr << 1) | 0); // Address + write bit
    twi_write(dta);
    twi_stop();
}

void LCD::i2c_send_bytes(uint8_t addr, uint8_t *dta, uint8_t len)
{
    twi_start();
    twi_write((addr << 1) | 0); // Address + write bit
    for (uint8_t i = 0; i < len; i++)
    {
        twi_write(dta[i]);
    }
    twi_stop();
}

// ========== LCD Internal Functions ==========

void LCD::command(uint8_t value)
{
    if (!initialized)
        return;

    uint8_t dta[2] = {0x80, value};
    i2c_send_bytes(LCD_ADDRESS, dta, 2);
}

// ========== Public LCD Functions ==========

void LCD::begin(uint8_t cols, uint8_t rows, uint8_t charsize)
{
    initialized = 1;

    if (rows > 1)
    {
        display_function |= LCD_2LINE;
    }
    num_lines = rows;
    curr_line = 0;

    // For some 1 line displays you can select a 10 pixel high font
    if ((charsize != 0) && (rows == 1))
    {
        display_function |= LCD_5x10DOTS;
    }

    // Wait for LCD to power up (>40ms after Vcc rises above 2.7V)
    _delay_ms(50);

    // Initialization sequence according to HD44780 datasheet
    command(LCD_FUNCTIONSET | display_function);
    _delay_us(4500);

    command(LCD_FUNCTIONSET | display_function);
    _delay_us(150);

    command(LCD_FUNCTIONSET | display_function);
    command(LCD_FUNCTIONSET | display_function);

    // Turn display on with no cursor or blinking
    display_control = LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF;
    display();

    // Clear display
    clear();

    // Set default text direction (left to right)
    display_mode = LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT;
    command(LCD_ENTRYMODESET | display_mode);
}

void LCD::clear()
{
    command(LCD_CLEARDISPLAY);
    _delay_ms(2);
}

void LCD::home()
{
    command(LCD_RETURNHOME);
    _delay_ms(2);
}

void LCD::set_cursor(uint8_t col, uint8_t row)
{
    if (!initialized)
        return;

    col = (row == 0 ? col | 0x80 : col | 0xc0);
    uint8_t dta[2] = {0x80, col};
    i2c_send_bytes(LCD_ADDRESS, dta, 2);
}

void LCD::no_display()
{
    display_control &= ~LCD_DISPLAYON;
    command(LCD_DISPLAYCONTROL | display_control);
}

void LCD::display()
{
    display_control |= LCD_DISPLAYON;
    command(LCD_DISPLAYCONTROL | display_control);
}

void LCD::no_cursor()
{
    display_control &= ~LCD_CURSORON;
    command(LCD_DISPLAYCONTROL | display_control);
}

void LCD::cursor()
{
    display_control |= LCD_CURSORON;
    command(LCD_DISPLAYCONTROL | display_control);
}

void LCD::no_blink()
{
    display_control &= ~LCD_BLINKON;
    command(LCD_DISPLAYCONTROL | display_control);
}

void LCD::blink()
{
    display_control |= LCD_BLINKON;
    command(LCD_DISPLAYCONTROL | display_control);
}

void LCD::scroll_display_left()
{
    command(LCD_CURSORSHIFT | LCD_DISPLAYMOVE | LCD_MOVELEFT);
}

void LCD::scroll_display_right()
{
    command(LCD_CURSORSHIFT | LCD_DISPLAYMOVE | LCD_MOVERIGHT);
}

void LCD::left_to_right()
{
    display_mode |= LCD_ENTRYLEFT;
    command(LCD_ENTRYMODESET | display_mode);
}

void LCD::right_to_left()
{
    display_mode &= ~LCD_ENTRYLEFT;
    command(LCD_ENTRYMODESET | display_mode);
}

void LCD::autoscroll()
{
    display_mode |= LCD_ENTRYSHIFTINCREMENT;
    command(LCD_ENTRYMODESET | display_mode);
}

void LCD::no_autoscroll()
{
    display_mode &= ~LCD_ENTRYSHIFTINCREMENT;
    command(LCD_ENTRYMODESET | display_mode);
}

void LCD::create_char(uint8_t location, uint8_t charmap[])
{
    location &= 0x7; // Only 8 locations 0-7
    command(LCD_SETCGRAMADDR | (location << 3));

    uint8_t dta[9];
    dta[0] = 0x40;
    for (uint8_t i = 0; i < 8; i++)
    {
        dta[i + 1] = charmap[i];
    }
    i2c_send_bytes(LCD_ADDRESS, dta, 9);
}

void LCD::create_char_P(uint8_t location, const uint8_t *charmap)
{
    location &= 0x7; // Only 8 locations 0-7
    command(LCD_SETCGRAMADDR | (location << 3));

    uint8_t dta[9];
    dta[0] = 0x40;
    for (uint8_t i = 0; i < 8; i++)
    {
        dta[i + 1] = pgm_read_byte(&charmap[i]);
    }
    i2c_send_bytes(LCD_ADDRESS, dta, 9);
}

void LCD::write(uint8_t value)
{
    if (!initialized)
        return;

    uint8_t dta[2] = {0x40, value};
    i2c_send_bytes(LCD_ADDRESS, dta, 2);
}

void LCD::print(const unsigned char *str)
{
    while (*str)
    {
        write(*str++);
    }
}

void LCD::print_P(const unsigned char *str)
{
    char c;
    while ((c = pgm_read_byte(str++)))
    {
        write(c);
    }
}
