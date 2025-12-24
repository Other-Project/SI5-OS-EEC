from smbus2 import SMBus
from time import *

class i2c_device:
    def __init__(self, addr, bus):
      self.addr = addr
      self.bus = SMBus(bus)

    # Écrit une valeur dans un registre
    def write_register(self, reg, value):
        try:
            self.bus.write_byte_data(self.address, reg, value)
            sleep(0.0001)
            return True
        except Exception as e:
            print(f"Erreur écriture registre 0x{reg:02X}: {e}")
            return False

    # Lit la valeur d'un registre
    def read_register(self, reg):
        try:
            value = self.bus.read_byte_data(self.address, reg)
            sleep(0.0001)
            return value
        except Exception as e:
            print(f"Erreur lecture registre 0x{reg:02X}: {e}")
            return None

    # Lit plusieurs registres consécutifs
    def read_registers(self, start_reg, count):
        try:
            values = []
            for i in range(count):
                val = self.bus.read_byte_data(self.address, start_reg + i)
                values.append(val)
            sleep(0.0001)
            return values
        except Exception as e:
            print(f"Erreur lecture multiple: {e}")
            return None

    # Écrit une simple commande
    def write_cmd(self, cmd):
        self.bus.write_byte(self.addr, cmd)
        sleep(0.0001)
"""
    # Write a command and argument
   def write_cmd_arg(self, cmd, data):
      self.bus.write_byte_data(self.addr, cmd, data)
      sleep(0.0001)

    # Write a block of data
   def write_block_data(self, cmd, data):
      self.bus.write_block_data(self.addr, cmd, data)
      sleep(0.0001)

    # Read a single byte
   def read(self):
      return self.bus.read_byte(self.addr)

    # Read
   def read_data(self, cmd):
      return self.bus.read_byte_data(self.addr, cmd)

    # Read a block of data
   def read_block_data(self, cmd):
      return self.bus.read_block_data(self.addr, cmd)
"""