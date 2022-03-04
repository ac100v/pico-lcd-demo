# Rotozoomer demo for Raspberry Pi Pico

[![YouTube Video](http://img.youtube.com/vi/5_K3iJBBL_E/0.jpg)](https://www.youtube.com/watch?v=5_K3iJBBL_E)

Rotozoomer is a visual effect that rotates and scales a bitmap image.  One of the most famous rotozoomer is 'Second Reality' by Future Crew, a PC demo of 1993.

I wrote a Rust program for Ruspberry Pi Pico that performs sound and animation simultaneously.  The music is from Second Reality.  The image is the logo of Rust language.

## Pin Connections

### Audio Out

|Speaker|Pico|
|-------|----|
|+      |GP28|
|-      |GND |

Pico outputs audio signal by PWM.  You'll find the sound too weak.

### SPI LCD

I used a monochrome 128x64 SPI LCD.

https://www.aitendo.com/product/10007


|LCD|Pico|
|----------|---|
| 1 ROM_IN |(NC)|
| 2 ROM_OUT|(NC)|
| 3 ROM_SCK|(NC)|
| 4 ROM_CS |(NC)|
| 5 LEDA   |3V3|
| 6 VSS    |GND |
| 7 VDD    |3V3|
| 8 SCK    |GP2 (SPI0 SCK)|
| 9 SDA    |GP3 (SPI0 TX)|
|10 RS     |GP4|
|11 RST    |3V3|
|12 CS     |GP1 (SPI0 CS)|

The program sends LCD's native SPI commands directly.  It may be difficult to port it for other display devices.


## License

The contents of this repository are dual-licensed under the _MIT OR Apache
2.0_ License. That means you can chose either the MIT licence or the
Apache-2.0 licence when you re-use this code. See `MIT` or `APACHE2.0` for more
information on each specific licence.

The Rust logos (bitmap and vector) are owned by Mozilla and distributed under the terms of the Creative Commons Attribution license (CC-BY). 

To learn more, see https://www.rust-lang.org/policies/media-guide

## Links

Second Reality: https://github.com/mtuomi/SecondReality
