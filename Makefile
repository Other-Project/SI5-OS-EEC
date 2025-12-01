# Project Configuration
PROJECT := EEE_Security_Alarm
MCU := atmega328p
F_CPU := 16000000L
BUILD_DIR := Build

# Toolchain
CC := avr-gcc
CXX := avr-g++
OBJCOPY := avr-objcopy
AVRDUDE := avrdude

# Arduino Configuration
ARDUINO_VERSION := 10812
ARDUINO_BOARD := ARDUINO_AVR_UNO
ARDUINO_ARCH := ARDUINO_ARCH_AVR

# Compiler Flags
COMMON_FLAGS := -g -Os -w -mmcu=$(MCU) -DF_CPU=$(F_CPU) \
                -DARDUINO=$(ARDUINO_VERSION) \
                -D$(ARDUINO_BOARD) \
                -D$(ARDUINO_ARCH) \
                -ffunction-sections -fdata-sections \
                -MMD -MP -flto

CFLAGS := $(COMMON_FLAGS) -std=gnu11 -fno-fat-lto-objects

CXXFLAGS := $(COMMON_FLAGS) -std=gnu++11 \
            -fpermissive -fno-exceptions \
            -fno-threadsafe-statics \
            -Wno-error=narrowing

# Include Paths
INCLUDES := -I. \
            -IarduinoLibsAndCore/cores/arduino \
            -IarduinoLibsAndCore/libraries/Wire/src \
            -IarduinoLibsAndCore/libraries/Wire/src/utility \
            -IarduinoLibsAndCore/libraries/SoftwareSerial/src \
            -IarduinoLibsAndCore/variants/standard \
            -IFreeRTOS-Kernel/include \
            -IFreeRTOS-Kernel/portable/GCC/ATMega328

# Source Directories
FREERTOS_DIR := FreeRTOS-Kernel
FREERTOS_PORT_DIR := $(FREERTOS_DIR)/portable/GCC/ATMega328
FREERTOS_MEM_DIR := $(FREERTOS_DIR)/portable/MemMang
ARDUINO_LIBS_DIR := arduinoLibsAndCore/libraries

# Source Files
C_SOURCES := \
    $(FREERTOS_DIR)/timers.c \
    $(FREERTOS_DIR)/tasks.c \
    $(FREERTOS_DIR)/queue.c \
    $(FREERTOS_DIR)/list.c \
    $(FREERTOS_DIR)/croutine.c \
    $(FREERTOS_MEM_DIR)/heap_1.c \
    $(FREERTOS_PORT_DIR)/port.c \
    $(ARDUINO_LIBS_DIR)/Wire/src/utility/twi.c

CXX_SOURCES := \
    main.cpp \
    $(ARDUINO_LIBS_DIR)/Wire/src/Wire.cpp \
    $(ARDUINO_LIBS_DIR)/SoftwareSerial/src/SoftwareSerial.cpp \
    drivers/lcd/lcd.cpp \
    drivers/rfid/rfid.cpp \
    drivers/ultrasonic/ultrasonic.cpp  \
    drivers/buzzer/buzzer.cpp

# Generate object file names
C_OBJECTS := $(addprefix $(BUILD_DIR)/, $(C_SOURCES:.c=.o))
CXX_OBJECTS := $(addprefix $(BUILD_DIR)/, $(CXX_SOURCES:.cpp=.o))
ALL_OBJECTS := $(C_OBJECTS) $(CXX_OBJECTS)

# Dependency files
DEPS := $(ALL_OBJECTS:.o=.d)

# Linker Flags
LDFLAGS := -w -Os -g -flto -fuse-linker-plugin \
           -Wl,--gc-sections -mmcu=$(MCU) \
           -static -L./arduinoLibsAndCore -lres

# Upload Configuration
PORT := /dev/ttyACM0
BAUD_RATE := 115200
PROGRAMMER := arduino

# Default Target
.PHONY: all
all: $(BUILD_DIR)/$(PROJECT).hex

# Link
$(BUILD_DIR)/$(PROJECT).elf: $(ALL_OBJECTS)
	@echo "Linking $@"
	@$(CXX) $(ALL_OBJECTS) $(LDFLAGS) -o $@
	@echo "Build complete: $@"

# Compile C sources
$(BUILD_DIR)/%.o: %.c
	@echo "Compiling $<"
	@mkdir -p $(dir $@)
	@$(CC) -c $(CFLAGS) $(INCLUDES) $< -o $@

# Compile C++ sources
$(BUILD_DIR)/%.o: %.cpp
	@echo "Compiling $<"
	@mkdir -p $(dir $@)
	@$(CXX) -c $(CXXFLAGS) $(INCLUDES) $< -o $@

# Generate HEX file
$(BUILD_DIR)/$(PROJECT).hex: $(BUILD_DIR)/$(PROJECT).elf
	@echo "Creating HEX file: $@"
	@$(OBJCOPY) -O ihex -R .eeprom $< $@

# Upload to Arduino
.PHONY: upload
upload: $(BUILD_DIR)/$(PROJECT).hex
	@echo "Uploading to $(PORT)..."
	$(AVRDUDE) -F -V -c $(PROGRAMMER) -p ATMEGA328P \
	           -P $(PORT) -b $(BAUD_RATE) \
	           -U flash:w:$<

# Clean build artifacts
.PHONY: clean
clean:
	@echo "Cleaning build directory..."
	@rm -rf $(BUILD_DIR)

# Serial Monitor
.PHONY: monitor
monitor:
	screen -S arduinoMonitor $(PORT) 9600

.PHONY: kill-monitor
kill-monitor:
	screen -XS arduinoMonitor quit

# Size report
.PHONY: size
size: $(BUILD_DIR)/$(PROJECT).elf
	@avr-size --format=avr --mcu=$(MCU) $<

# Print variables (for debugging)
.PHONY: print-%
print-%:
	@echo '$*=$($*)'

# Include dependency files
-include $(DEPS)

.DEFAULT_GOAL := all
