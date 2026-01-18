# Smart Home Alarm System

<p align=center>
    <span>Project realized by <a href="https://github.com/ElianDELMAS">Elian Delmas</a>, <a href="https://github.com/06Games">Evan Galli</a> and <a href="https://github.com/eliotmnrt">Eliot Menoret</a>
    <br/>as part of the <b>Embedded OS and Edge Computing</b> course.</span>
    <br/>
    <br/><img src="https://github.com/user-attachments/assets/da2318a8-abe4-407c-b81d-fce52d4501f2" width="50%" />
</p>

## Introduction

This project implements a smart home alarm system designed around an Edge Computing architecture. Instead of centralizing all logic on a single device, the system adopts a distributed heterogeneous approach to optimize cognitive load and resource management.

The architecture separates real-time constraints from high-level supervision:

* The Microcontroller (Arduino with FreeRTOS): Acts as an intelligent acquisition unit. It encapsulates hardware complexity and signal processing to expose only qualified events rather than raw data.

* The Gateway (Raspberry Pi 3B+ with Raspbian): Handles system supervision, data persistence, and the user interface. Liberated from low-level polling tasks, it manages the badge database and high-level logic using a robust Rust application.

Functionally, the system detects intrusions using an ultrasonic sensor and triggers audio-visual alerts (Buzzer/LED) when armed. Users can arm the system via a physical button and disarm it using authorized RFID badges. Management operations, such as adding or revoking access badges, are performed locally via an LCD interface or on the Raspberry Pi using a TUI.

<table>
    <tr>
        <td><img width="603" height="76" alt="image" src="https://github.com/user-attachments/assets/b187d31e-57d9-4d95-bffb-f8d696937f72" /></td>
        <td><img width="1920" height="963" alt="image4" src="https://github.com/user-attachments/assets/6a4be3e2-a6e3-4360-99b3-8e1d7611c1c4" /></td>
    </tr>
    <tr>
        <td><img width="1920" height="963" alt="image2" src="https://github.com/user-attachments/assets/c0fa5978-1a22-49a8-ad71-ad33fee065fd" /></td>
        <td><img width="1920" height="963" alt="image3" src="https://github.com/user-attachments/assets/0dacfec1-3f24-4413-ae76-8410463d65c0" /></td>
    </tr>
</table>

## Instructions

### Prerequisites

* [Rust](https://www.rust-lang.org/tools/install) for Raspberry Pi code
* [Docker](https://docs.docker.com/engine/install/) for Rust cross-compilation
* `avrdude` for Arduino flashing
* `avr-binutils`, `avr-libc`, `avr-gcc` and `avr-gcc-c++` for Arduino compilation

### Wiring

| Arduino |   Device   | Raspberry Pi 3B |
| :-----: | :--------: | :-------------: |
|   D2    |   Button   |                 |
|   D4    | Ultrasonic |                 |
|   D5    |    LED     |                 |
|   D6    |   Buzzer   |                 |
|   D7    |    RFID    |                 |
|         |   Rotary   |       A0        |
|         |   Button   |       A1        |
|         |   Screen   |      I2C-2      |
|   I2C   |            |      I2C-1      |

### Arduino

1. Connect the Arduino to your computer
2. Build and flash the Arduino using `make -C arduino upload`

### Raspberry Pi 3B

1. Flash the Raspberry Pi 3B with Raspbian Server
2. Install i2c-tools using `sudo apt install i2c-tools`
3. Configure I²C using `sudo raspi-config` (Interface Options -> I2C)
4. Check for I²C connection with `i2cdetect -y 1`
5. On your computer, build and deploy the rust code using `make -C raspberry deploy`
6. Launch the program on the Raspberry Pi 3B using `./arduino_i2c --tui` (or setup `./arduino_i2c` as a service)
