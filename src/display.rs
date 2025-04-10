use cortex_m::asm;
use stm32f4::stm32f401;

use super::constants::CLK_HZ;

/*
    ST7735 Display

    CON|PIN|NOTE
    ============
    VCC|3.3|
    GND|GND|
    GND|GND|
    NC |   |
    NC |   |
    NC |   |
    CLK|PA5|SPI1_SCK
    SDA|PA7|SPI1_MOSI
    RS |PA4|Data/Command select (GPIO)
    RST|PA1|Reset line (GPIO)
    CS |PA0|Chip Select (GPIO)
*/

pub trait Display {

    /// Setup and turn on the display
    fn calibrate(&self);

    /// Fill in the display with a solid color
    fn fill(&self, color: Option<u32>);
}

pub struct ST7735<'a> {
    spi: stm32f401::SPI1,
    gpio: &'a stm32f401::GPIOA,
    width: u32,
    height: u32
}

impl<'a> Display for ST7735<'a> {
    fn calibrate(&self) {
        const SWRESET: u8 = 0x01;
        const SLPOUT: u8 = 0x11;
        const DISPON: u8 = 0x29;

        // TODO - replace display with one that supports:
        // * Setting COLMOD to 16-bit RGB with a display that supports it
        // * Clearing display ram before turning on display

        // Disable Chip Select (CS)
        self.display_cs_disable();

        // Enable Reset (RST)
        self.display_rst_enable();

        // ~120ms delay
        asm::delay(CLK_HZ / 1000 * 120);

        // Disable Reset (RST)
        self.display_rst_disable();

        // Put Register Select (RS) into command mode
        self.display_rs_command_mode();

        // ~120ms delay
        asm::delay(CLK_HZ / 1000 * 120);

        self.display_cs_enable();

        // Software reset
        self.display_rs_command_mode();
        self.spi1_write(SWRESET);
        asm::delay(CLK_HZ / 1000 * 120); // ~120ms

        // Wake up display (from reset sleep)
        self.spi1_write(SLPOUT);
        asm::delay(CLK_HZ / 1000 * 120); // ~120ms

        // Turn on the display
        self.display_rs_command_mode();
        self.spi1_write(DISPON);
        asm::delay(CLK_HZ / 1000 * 120); // ~120ms

        // Clear display
        self.fill(None);
    }

    fn fill(&self, color: Option<u32>) {
        const CASET: u8 = 0x2A;
        const RASET: u8 = 0x2B;
        const RAMWR: u8 = 0x2C;
        const NOP: u8 = 0x00;

        const WHITE: u32 = 0xFFFFFF;

        let color = color.unwrap_or(WHITE);

        self.display_cs_enable();

        // Draw sequence fails without this
        self.display_rs_command_mode();
        self.spi1_write(NOP);

        // Set column range
        self.display_rs_command_mode();
        self.spi1_write(CASET);
        self.display_rs_data_mode();
        // Set x0
        self.spi1_write(0x00); // MSB
        self.spi1_write(0x00); // LSB
        // Set x1
        self.spi1_write(0x00); // MSB
        self.spi1_write((self.width - 1) as u8); // LSB

        // Set row range
        self.display_rs_command_mode();
        self.spi1_write(RASET);
        self.display_rs_data_mode();
        // Set y0
        self.spi1_write(0x00); // MSB
        self.spi1_write(0x00); // LSB
        // Set y1
        self.spi1_write(0x00); // MSB
        self.spi1_write((self.height - 1) as u8); // LSB

        // Write to the display
        self.display_rs_command_mode();
        self.spi1_write(RAMWR);
        self.display_rs_data_mode();

        // Fill in display
        for _ in 0..self.height {
            for _ in 0..self.width {
                self.spi1_write(((color >> 16) & 0xFF) as u8); // R
                self.spi1_write(((color >> 8) & 0xFF) as u8); // G
                self.spi1_write((color & 0xFF) as u8); // B
            }
        }

        self.display_rs_command_mode();
        self.display_cs_disable();
    }
}

impl<'a> ST7735<'a> {
    pub fn new(
        rcc: &stm32f401::RCC,
        gpioa: &'a stm32f401::GPIOA,
        spi1: stm32f401::SPI1,
        width: u32,
        height: u32
    ) -> Self {

        // Enable GPIOA clock
        rcc.ahb1enr.modify(|_, w| w.gpioaen().enabled());

        // Configure output pins
        gpioa.moder.modify(|_, w| {
            w.moder0().output() // CS
             .moder1().output() // RST
             .moder4().output() // RS
        });

        // Enable SPI1 clock
        rcc.apb2enr.modify(|_, w| w.spi1en().enabled());

        // Configure SPI pins
        gpioa.moder.modify(|_, w| {
            w.moder5().alternate() // CLK
             .moder7().alternate() // SDA
        });

        // Set SPI pin alternate functions
        gpioa.afrl.modify(|_, w| {
            w.afrl5().af5() // SPI1_SCK
             .afrl7().af5() // SPI1_MOSI
        });

        // Configure SPI1
        spi1.cr1.modify(|_, w| {
            w.bidimode().clear_bit()
             .bidioe().clear_bit()
             .rxonly().clear_bit()
             .dff().clear_bit()
             .lsbfirst().clear_bit()
             .ssm().set_bit()
             .ssi().set_bit()
             .mstr().set_bit()
             .br().div8()
             .cpol().clear_bit()
             .cpha().clear_bit()
        });

        // Enable SPI1
        spi1.cr1.modify(|_, w| w.spe().set_bit());

        ST7735 { spi: spi1, gpio: gpioa, width, height }
    }

    fn spi1_write(&self, byte: u8) {
        // Wait for TX buffer to be empty
        while self.spi.sr.read().txe().bit_is_clear() {}

        self.spi.dr.write(|w| w.dr().bits(byte.into()));

        // Wait for SPI to be busy (TX started)
        while self.spi.sr.read().bsy().bit_is_set() {}
    }

    fn display_rst_enable(&self) {
        self.gpio.bsrr.write(|w| w.br1().set_bit());
    }

    fn display_rst_disable(&self) {
        self.gpio.bsrr.write(|w| w.bs1().set_bit());
    }

    fn display_cs_enable(&self) {
        self.gpio.bsrr.write(|w| w.br0().set_bit());
    }

    fn display_cs_disable(&self) {
        self.gpio.bsrr.write(|w| w.bs0().set_bit());
    }

    fn display_rs_command_mode(&self) {
        self.gpio.bsrr.write(|w| w.br4().set_bit());
    }

    fn display_rs_data_mode(&self) {
        self.gpio.bsrr.write(|w| w.bs4().set_bit());
    }
}
