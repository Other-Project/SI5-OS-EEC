#!/usr/bin/env python3
"""
Protocole I2C pour communication avec Arduino
Raspberry Pi côté Master
"""

from smbus2 import SMBus
import time

# Configuration
I2C_SLAVE_ADDR = 0x32
I2C_BUS = 1

# Map des registres (doit correspondre à i2c.h de l'Arduino)
REG_STATUS = 0x00
REG_EVENTS = 0x01
REG_RFID = 0x02

EVENT_BTN_PRESSED = 0x01
EVENT_MOTION_DETECTED = 0x02
EVENT_RFID_READ = 0x04

class ArduinoI2C:
    """Classe pour communiquer avec l'Arduino via I2C"""
    
    def __init__(self, bus=I2C_BUS, address=I2C_SLAVE_ADDR):
        self.bus = SMBus(bus)
        self.address = address
    
    def write_register(self, reg, value):
        """Écrit une valeur dans un registre"""
        try:
            self.bus.write_byte_data(self.address, reg, value)
            return True
        except Exception as e:
            print(f"Erreur écriture registre 0x{reg:02X}: {e}")
            return False
    
    def read_register(self, reg):
        """Lit la valeur d'un registre"""
        try:
            value = self.bus.read_byte_data(self.address, reg)
            return value
        except Exception as e:
            print(f"Erreur lecture registre 0x{reg:02X}: {e}")
            return None
    
    def read_registers(self, start_reg, count):
        """Lit plusieurs registres consécutifs"""
        try:
            values = []
            for i in range(count):
                val = self.bus.read_byte_data(self.address, start_reg + i)
                values.append(val)
            return values
        except Exception as e:
            print(f"Erreur lecture multiple: {e}")
            return None
    
    # === Fonctions de haut niveau ===
    
    def set_alarm(self, enabled):
        """Active/désactive l'alarme"""
        return self.write_register(REG_STATUS, 1 if enabled else 0)
    
    def get_alarm_state(self):
        """Lit l'état de l'alarme"""
        return bool(self.read_register(REG_STATUS))
        
    def is_motion_detected(self):
        """Vérifie si un mouvement est détecté"""
        status = self.read_register(REG_EVENTS)
        return status is not None and (status & EVENT_MOTION_DETECTED) != 0

    def get_rfid_tag(self):
        """Lit l'ID du badge RFID (6 bytes)"""
        status = self.read_register(REG_EVENTS)
        if status is None or not (status & EVENT_RFID_READ):
            return None

        values = self.read_registers(REG_RFID, 6)
        if values:
            # Convertir en string hexadécimal
            tag_id = ''.join(f'{b:02X}' for b in values)
            return tag_id
        return None
    
    def get_status(self):
        """Lit le status général du système"""
        return self.read_register(REG_STATUS)

def monitor_sensors():
    """Surveillance continue des capteurs"""
    print("Surveillance des capteurs (Ctrl+C pour arrêter)...\n")
    arduino = ArduinoI2C()
    
    try:
        while True:
            arduino.set_alarm(True)  # Activer l'alarme pour test

            # Lire tous les capteurs
            motion = arduino.is_motion_detected()
            alarm = arduino.get_alarm_state()
            rfid_tag = arduino.get_rfid_tag()

            # Afficher les valeurs
            print(f"\r[Alarme: {'ON ' if alarm else 'OFF'}] "
                  f"[Mouvement: {'OUI' if motion else 'NON'}] "
                  f"[RFID: {rfid_tag if rfid_tag else 'Aucun':16s}]", 
                  end='\n', flush=True)
            
            time.sleep(0.2)
            
    except KeyboardInterrupt:
        print("\n\nArrêt de la surveillance.")

def main():
    """Menu principal"""
    monitor_sensors()


if __name__ == "__main__":
    main()
