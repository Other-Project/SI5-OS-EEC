#!/usr/bin/env python3

from raspberry_i2c_lib import i2c_device
from time import sleep

# LCD Address
ADDRESS = 0x3E
I2C_BUS = 1

# Commandes de base
LCD_CLEARDISPLAY = 0x01
LCD_RETURNHOME = 0x02
LCD_ENTRYMODESET = 0x04
LCD_DISPLAYCONTROL = 0x08
LCD_CURSORSHIFT = 0x10
LCD_FUNCTIONSET = 0x20
LCD_SETCGRAMADDR = 0x40
LCD_SETDDRAMADDR = 0x80

# Flags pour display entry mode
LCD_ENTRYLEFT = 0x02
LCD_ENTRYSHIFTINCREMENT = 0x01

# Flags pour display on/off control
LCD_DISPLAYON = 0x04
LCD_CURSOROFF = 0x00
LCD_BLINKOFF = 0x00

# Flags pour function set
LCD_2LINE = 0x08
LCD_5x8DOTS = 0x00

class Lcd(i2c_device):
    def __init__(self, address=ADDRESS, bus=I2C_BUS):
        super().__init__(address, bus)

        sleep(0.2)
        
        self.lcd_write_cmd(LCD_FUNCTIONSET | LCD_2LINE | LCD_5x8DOTS)
        sleep(0.2)
        self.lcd_write_cmd(LCD_FUNCTIONSET | LCD_2LINE | LCD_5x8DOTS)
        sleep(0.2)
        
        self.lcd_write_cmd(LCD_DISPLAYCONTROL | LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF)
        self.lcd_clear()
        self.lcd_write_cmd(LCD_ENTRYMODESET | LCD_ENTRYLEFT | LCD_ENTRYSHIFTINCREMENT)
        sleep(0.2)

    # Registre 0x80 = Envoyer une commande
    def lcd_write_cmd(self, cmd):
        self.write_register(0x80, cmd)

    # Registre 0x40 = Envoyer une donnée (caractère)
    def lcd_write_char(self, char_value):
        self.write_register(0x40, char_value)

    # Afficher une chaîne de caractères
    def lcd_display_string(self, string, line):
        if line == 1:
            self.lcd_write_cmd(0x80 | 0x00) # Adresse début ligne 1
        elif line == 2:
            self.lcd_write_cmd(0x80 | 0x40) # Adresse début ligne 2 (+ 0x40)

        for char in string:
            self.lcd_write_char(ord(char))

    def lcd_clear(self):
        self.lcd_write_cmd(LCD_CLEARDISPLAY)
        sleep(0.0001)
        self.lcd_write_cmd(LCD_RETURNHOME)
        sleep(0.0001)

def main():
    try:
        print(f"Tentative de connexion à l'écran sur 0x{ADDRESS:02X}...")
        lcd = Lcd()
        print("Écriture sur l'écran...")
        
        lcd.lcd_display_string("Hello world", 1)
        lcd.lcd_display_string("Raspberry Pi", 2)
        
        # Petit test de clignotement du texte pour vérifier que le script tourne
        while True:
            sleep(2)
            lcd.lcd_clear()
            sleep(0.5)
            lcd.lcd_display_string("It Works!", 1)
            
    except KeyboardInterrupt:
        print("\nArrêt du programme")
        try:
            lcd.lcd_clear()
        except:
            pass

if __name__ == "__main__":
    main()