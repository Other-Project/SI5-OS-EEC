from smbus2 import SMBus
import time

I2C_SLAVE_ADDR = 0x32


# Registres (doivent correspondre à ceux de l'Arduino)
REG_SENSOR = 0x01
REG_ACTUATOR = 0x02

def main():
    with SMBus(1) as bus: # Bus 1 est le standard sur RPi
        while True:
            try:
                # 1. ÉCRIRE : Envoyer une commande (Allumer actionneur = 1)
                print("Envoi commande ON...")
                bus.write_byte_data(I2C_SLAVE_ADDR, REG_ACTUATOR, 1)
                time.sleep(3)
                
                # 2. LIRE : Demander la valeur du capteur
                # write_byte (pour placer le pointeur) puis read_byte n'est pas atomique
                # read_byte_data fait les deux automatiquement
                """valeur = bus.read_byte_data(I2C_SLAVE_ADDR, REG_SENSOR)
                print(f"Valeur capteur reçue : {valeur}")"""

                # 3. ÉCRIRE : Envoyer commande OFF
                print("Envoi commande OFF...")
                bus.write_byte_data(I2C_SLAVE_ADDR, REG_ACTUATOR, 0)
                
                time.sleep(0.25)

            except Exception as e:
                print(f"Erreur I2C : {e}")
                time.sleep(1)

if __name__ == "__main__":
    main()