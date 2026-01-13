# Smart Home Alarm System

<p align=center>
    <span>Project realized by <a href="https://github.com/ElianDELMAS">Elian Delmas</a>, <a href="https://github.com/06Games">Evan Galli</a> and <a href="https://github.com/eliotmnrt">Eliot Menoret</a>
    <br/>as part of the <b>Embedded OS and Edge Computing</b> course.</span>
</p>

## Introduction

Ce projet consiste en un système d'alarme intelligent pour une maison. En cas de détection d'une présence dans la maison alors que l'alarme est activée, une signal sonore s'activera dans la maison pour effrayer l’intrus. Il sera possible pour l'habitant d'activer son alarme lorsqu'il sort de chez lui, puis de la désactiver en revenant avec un badge RFID personnel.
En plus de cela, l'habitant aura la possibilité d'ajouter de nouveaux badges RFID qu'il pourra donner à ses proches pour que ces derniers puissent eux-même désactiver l'alarme. De manière similaire, il pourra également révoquer des badges RFID pour qu'ils ne puissent plus être utilisés pour désactiver l'alarme. Ces opérations seront réalisées directement au niveau de la Raspberry Pi (Edge).

Ce système comprend :

- un capteur ultrason pour détecter une présence dans la maison
- un bouton permettant à l'habitant d'activer son alarme lorsqu'il s'absente
- un lecteur RFID (récupéré auprès de vous) pour que l'habitant s'identifie et désactive l'alarme lorsqu'il rentre chez lui
- une LED pour signaler si l'alarme est activée ou non
- un buzzer qui va émettre un son en cas de détection d'une présence dans la maison lorsque l'alarme est activée
- un afficheur LCD pour permettre à l'habitant de gérer les badges RFID acceptés pour désactiver l'alarme
- un potentiomètre et un encodeur de souris (récupéré auprès de vous) utilisé pour naviguer sur l'afficheur LCD entre les modes d'ajout et de révocation de badges RFID
- un bouton pour sélectionner le mode d'ajout ou de révocation sur l'afficheur LCD pour ajouter ou révoquer un badge RFID

## Instructions

### Prerequisites

* [Rust](https://www.rust-lang.org/tools/install) for Raspberry Pi code
* [Docker](https://docs.docker.com/engine/install/) for Rust cross-compilation
* avrdude for Arduino flashing
* avr-gcc for Arduino compilation

### Wiring

| Arduino |   Device   | Raspberry Pi 3B |
| :-----: | :--------: | :-------------: |
|   D2    |   Button   |                 |
|   D4    | Ultrasonic |                 |
|   D5    |    LED     |                 |
|   D6    |   Buzzer   |                 |
|   D7    |    RFID    |                 |
|         |   Rotary   |       A0        |
|         |   Button   |       D2        |
|         |   Screen   |      I2C-2      |
|   I2C   |            |      I2C-1      |

### Arduino

1. Connect the Arduino to your computer
2. Build and flash the Arduino using `make -C arduino upload`

### Raspberry Pi 3B

1. Flash the Raspberry Pi 3B with Raspbian Server
2. Install i2c-tools using `sudo apt install i2c-tools`
3. Configure I2C using `sudo raspi-config` (Interface Options -> I2C)
4. Check for I2C connection with `i2cdetect -y 1`
5. On your computer, build and deploy the rust code using `make -C raspberry deploy`
6. Launch the program on the Raspberry Pi 3B using `./arduino_i2c`
