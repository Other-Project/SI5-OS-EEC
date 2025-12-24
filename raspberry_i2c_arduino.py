#!/usr/bin/env python3

#from smbus2 import SMBus
from raspberry_i2c_lib import i2c_device
import time

# Configuration
I2C_SLAVE_ADDR = 0x32
I2C_BUS = 1

# Map des registres (doit correspondre à i2c.h de l'Arduino)
REG_STATUS          = 0x00
REG_ALARM_STATE     = 0x01
REG_LED_CMD         = 0x02
REG_MOTION_DETECTED = 0x03
REG_BUZZER_CMD      = 0x04
REG_DISTANCE_H      = 0x05
REG_DISTANCE_L      = 0x06
REG_RFID_STATUS     = 0x07
REG_RFID_ID_0       = 0x08
REG_RFID_ID_1       = 0x09
REG_RFID_ID_2       = 0x0A
REG_RFID_ID_3       = 0x0B
REG_RFID_ID_4       = 0x0C
REG_RFID_ID_5       = 0x0D
#REG_RFID_ID_6       = 0x0E
#REG_RFID_ID_7       = 0x0F
#REG_ROTARY_ANGLE    = 0x10
REG_BUTTON_STATE    = 0x11
REG_COMMAND         = 0x12
REG_ERROR_CODE      = 0x13


class Arduino(i2c_device):
    def __init__(self, bus=I2C_BUS, address=I2C_SLAVE_ADDR):
        super().__init__(address, bus)
    
    # Active/désactive l'alarme
    def set_alarm(self, enabled):
        return self.write_register(REG_ALARM_STATE, 1 if enabled else 0)
    
    # Lit l'état de l'alarme
    def get_alarm_state(self):
        return bool(self.read_register(REG_ALARM_STATE))
    
    # Active/désactive le buzzer
    def set_buzzer(self, enabled):
        return self.write_register(REG_BUZZER_CMD, 1 if enabled else 0)
    
    # Active/désactive la LED
    def set_led(self, enabled):
        return self.write_register(REG_LED_CMD, 1 if enabled else 0)
    
    # Lit la distance du capteur ultrason (en mm)
    def get_distance(self):
        values = self.read_registers(REG_DISTANCE_H, 2)
        if values:
            distance = (values[0] << 8) | values[1]
            return distance
        return None
    
    # Vérifie si un mouvement est détecté
    def is_motion_detected(self):
        return bool(self.read_register(REG_MOTION_DETECTED))
    
    # Lit l'ID du badge RFID (8 bytes)
    def get_rfid_tag(self):
        status = self.read_register(REG_RFID_STATUS)
        if not status:
            return None
        
        values = self.read_registers(REG_RFID_ID_0, 6)
        if values:
            
            tag_id = ''.join(f'{b:02X}' for b in values) # Convertir en string hexadécimal
            return tag_id
        return None
    
    """# Lit l'angle du potentiomètre (0-300°)
    def get_rotary_angle(self):
        value = self.read_register(REG_ROTARY_ANGLE)
        if value is not None:
            # Convertir de 0-255 vers 0-300°
            angle = (value * 300) / 255
            return angle
        return None"""

    # Lit le status général du système
    def get_status(self):
        return self.read_register(REG_STATUS)


def test_basic_communication():
    """Test basique de communication"""
    print("Test de communication I2C avec Arduino...")
    arduino = Arduino()
    
    # Test 1: Lire le status
    print("\n1. Lecture du status système")
    status = arduino.get_status()
    print(f"   Status: 0x{status:02X}" if status is not None else "   Erreur")
    
    # Test 2: Contrôle LED
    print("\n2. Test LED")
    print("   Allumage LED...")
    arduino.set_led(True)
    time.sleep(2)
    print("   Extinction LED...")
    arduino.set_led(False)
    
    # Test 3: Contrôle buzzer
    print("\n3. Test buzzer (bip court)")
    arduino.set_buzzer(True)
    time.sleep(0.5)
    arduino.set_buzzer(False)
    
    # Test 4: Lecture distance
    print("\n4. Lecture capteur ultrason")
    distance = arduino.get_distance()
    print(f"   Distance: {distance} mm" if distance else "   Erreur")
    
    # Test 5: État alarme
    print("\n5. Contrôle alarme")
    print("   Activation alarme...")
    arduino.set_alarm(True)
    time.sleep(1)
    state = arduino.get_alarm_state()
    print(f"   État alarme: {'ACTIVÉE' if state else 'DÉSACTIVÉE'}")
    time.sleep(2)
    print("   Désactivation alarme...")
    arduino.set_alarm(False)


def monitor_sensors():
    """Surveillance continue des capteurs"""
    print("Surveillance des capteurs (Ctrl+C pour arrêter)...\n")
    arduino = Arduino()
    
    try:
        while True:
            # Lire tous les capteurs
            distance = arduino.get_distance()
            motion = arduino.is_motion_detected()
            alarm = arduino.get_alarm_state()
            rfid_tag = arduino.get_rfid_tag()
            
            # Afficher les valeurs
            print(f"\r[Alarme: {'ON ' if alarm else 'OFF'}] "
                  f"[Distance: {distance:4d}mm] "
                  f"[Mouvement: {'OUI' if motion else 'NON'}] "
                  f"[RFID: {rfid_tag if rfid_tag else 'Aucun':16s}]", 
                  end='', flush=True)
            
            time.sleep(0.2)
            
    except KeyboardInterrupt:
        print("\n\nArrêt de la surveillance.")


def alarm_system_demo():
    """Démo du système d'alarme complet"""
    print("=== Démonstration du système d'alarme ===\n")
    arduino = Arduino()
    
    # Base de données simple de badges autorisés
    authorized_tags = set()
    
    print("Phase 1: Enregistrement de badges RFID")
    print("Veuillez scanner un badge à autoriser (timeout 10s)...")
    
    start_time = time.time()
    while time.time() - start_time < 10:
        tag = arduino.get_rfid_tag()
        if tag and tag not in authorized_tags:
            authorized_tags.add(tag)
            print(f"\n✓ Badge enregistré: {tag}")
            arduino.set_buzzer(True)
            time.sleep(0.2)
            arduino.set_buzzer(False)
            break
        time.sleep(0.1)
    
    print(f"\nBadges autorisés: {len(authorized_tags)}")
    print("\nPhase 2: Activation du système d'alarme")
    print("Activation dans 3 secondes...")
    
    for i in range(3, 0, -1):
        print(f"{i}...")
        time.sleep(1)
    
    arduino.set_alarm(True)
    print("✓ ALARME ACTIVÉE\n")
    
    print("Phase 3: Surveillance")
    print("Le système surveille les mouvements.")
    print("Scannez un badge autorisé pour désactiver.\n")
    
    alarm_triggered = False
    
    try:
        while True:
            # Vérifier les mouvements
            motion = arduino.is_motion_detected()
            
            if motion and not alarm_triggered:
                print("⚠ MOUVEMENT DÉTECTÉ! Buzzer activé!")
                alarm_triggered = True
            
            # Vérifier les badges RFID
            tag = arduino.get_rfid_tag()
            if tag:
                if tag in authorized_tags:
                    print(f"\n✓ Badge autorisé détecté: {tag}")
                    print("Désactivation de l'alarme...")
                    arduino.set_alarm(False)
                    arduino.set_buzzer(False)
                    print("✓ Alarme désactivée\n")
                    break
                else:
                    print(f"\n✗ Badge NON autorisé: {tag}")
                    print("Alarme maintenue!")
            
            time.sleep(0.1)
            
    except KeyboardInterrupt:
        print("\n\nArrêt forcé.")
        arduino.set_alarm(False)
        arduino.set_buzzer(False)


def main():
    """Menu principal"""
    print("=== Contrôle Arduino via I2C ===")
    print("1. Test basique de communication")
    print("2. Surveillance des capteurs")
    print("3. Démo système d'alarme complet")
    print("4. Quitter")
    
    choice = input("\nChoix: ")
    
    if choice == "1":
        test_basic_communication()
    elif choice == "2":
        monitor_sensors()
    elif choice == "3":
        alarm_system_demo()
    else:
        print("Au revoir!")


if __name__ == "__main__":
    main()