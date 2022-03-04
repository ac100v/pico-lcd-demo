//! #UC1701

use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;

pub struct UC1701<SPI, CD>
where
    SPI: spi::Write<u8>,
    CD: OutputPin,
{
    spi: SPI,
    cd: CD,
}

impl<SPI, CD> UC1701<SPI, CD>
where
    SPI: spi::Write<u8>,
    CD: OutputPin,
{
    pub fn new(spi: SPI, cd: CD) -> Self {
        UC1701 { spi, cd }
    }

    pub fn init(&mut self, delay: &mut dyn DelayUs<u32>) {
        let cmds = [
            0xe2, // 15. System Reset
            0x40, //  6. Set Scroll Line (SL=0)
            0xa1, // 13. Set SEG Direction (MX=1)
            0xc0, // 14. Set COM Direction (MY=0)
            0xa2, // 17. Set LCD Bias Ratio (BR=0)
            0x2c, //  5. Set Power Control (PC=0b100)
            0x2e, //  5. Set Power Control (PC=0b110)
            0x2f, //  5. Set Power Control (PC=0b111)
            0xf8, // 22. Set Booster Ratio (NOP)
            0x00, //     ...
            0x23, //  8. Set VLCD Resistor Ratio (PC=0b011)
            0x81, //  9. Set Electronic Volume
            0x2e, //     ... (PM=0x2e)
            0xac, // 20. Set Static Indicator OFF (NOP)
            0x00, // ?
            0xa6, // 11. Set Inverse Display (DC0=0)
            0xa4, // 10. Set All Pixel ON (DC1=0)
            0xaf, // 12. Set Display Enable (DC2=1)
            0xb0, //  7. Set Page Address (PA=0)
            0x00, //  4. Set Column Address (CA[3:0]=0b0000)
            0x10, //  4. Set Column Address (CA[7:4]=0b0000)
        ];

        for cmd in cmds {
            self.write_command(cmd);
            delay.delay_us(10000);
        }

        for p in 0..8 {
            self.write_command(0xb0 + p);
            self.write_command(0x00);
            self.write_command(0x10);
            let data = [0u8; 132];
            self.write_data(&data);
        }
        self.write_command(0xaf);
    }

    pub fn write_command(&mut self, cmd: u8) {
        let _ = self.cd.set_low();
        let buf: [u8; 1] = [cmd];
        let _ = self.spi.write(&buf);
    }

    pub fn write_data(&mut self, data: &[u8]) {
        let _ = self.cd.set_high();
        let _ = self.spi.write(data);
    }

    pub fn set_page(&mut self, p: u32) {
        self.write_command(0xb0 + p as u8); //  7. Set Page Address
        self.write_command(0x04); //  8. Set Column Address
        self.write_command(0x10); //  ...
    }
}
