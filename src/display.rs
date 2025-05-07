use cortex_m::asm;
use stm32f4::stm32f401;

use super::constants::CLK_HZ;

#[derive(Copy, Clone)]
pub enum PinState {
    Enable,
    Disable
}

#[derive(Copy, Clone)]
pub enum ControlMode {
    Data,
    Command
}

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

    fn draw_row(&self, row: u32, buf: &[u16]);
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

        // CS not needed for hardware reset
        self.chip_select(PinState::Disable);

        // Reset display
        self.reset(PinState::Enable);
        asm::delay(CLK_HZ / 1000 * 120); // ~120ms
        self.reset(PinState::Disable);
        asm::delay(CLK_HZ / 1000 * 120); // ~120ms

        self.chip_select(PinState::Enable);

        // Software reset
        self.register_select(ControlMode::Command);
        self.spi_write(SWRESET);
        asm::delay(CLK_HZ / 1000 * 120); // ~120ms

        // Wake up display (from reset sleep)
        self.spi_write(SLPOUT);
        asm::delay(CLK_HZ / 1000 * 120); // ~120ms

        // Turn on the display
        self.register_select(ControlMode::Command);
        self.spi_write(DISPON);
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

        self.chip_select(PinState::Enable);

        // Draw sequence fails without this
        self.register_select(ControlMode::Command);
        self.spi_write(NOP);

        // Set column range
        self.register_select(ControlMode::Command);
        self.spi_write(CASET);
        self.register_select(ControlMode::Data);
        // Set x0
        self.spi_write(0x00); // MSB
        self.spi_write(0x00); // LSB
        // Set x1
        self.spi_write(0x00); // MSB
        self.spi_write((self.width - 1) as u8); // LSB

        // Set row range
        self.register_select(ControlMode::Command);
        self.spi_write(RASET);
        self.register_select(ControlMode::Data);
        // Set y0
        self.spi_write(0x00); // MSB
        self.spi_write(0x00); // LSB
        // Set y1
        self.spi_write(0x00); // MSB
        self.spi_write((self.height - 1) as u8); // LSB

        // Write to the display
        self.register_select(ControlMode::Command);
        self.spi_write(RAMWR);
        self.register_select(ControlMode::Data);

        // Fill in display
        for _ in 0..self.height {
            for _ in 0..self.width {
                self.spi_write(((color >> 16) & 0xFF) as u8); // R
                self.spi_write(((color >> 8) & 0xFF) as u8); // G
                self.spi_write((color & 0xFF) as u8); // B
            }
        }

        self.register_select(ControlMode::Command);
        self.chip_select(PinState::Disable);
    }

    // Note: drawing camera "row" here to LCD col since LCD has longer vertical
    fn draw_row(&self, row: u32, buf: &[u16]) {

        const CASET: u8 = 0x2A;
        const RASET: u8 = 0x2B;
        const RAMWR: u8 = 0x2C;
        const NOP: u8 = 0x00;

        let length = self.width.min(buf.len().try_into().unwrap());

        if length == 0 {
            return;
        }

        self.chip_select(PinState::Enable);

        // Draw sequence fails without this
        self.register_select(ControlMode::Command);
        self.spi_write(NOP);

        // Set column range
        self.register_select(ControlMode::Command);
        self.spi_write(CASET);
        self.register_select(ControlMode::Data);
        // Set x0
        self.spi_write(0x00); // MSB
        self.spi_write(row as u8); // LSB
        // Set x1
        self.spi_write(0x00); // MSB
        self.spi_write(row as u8); // LSB

        // Set row range
        self.register_select(ControlMode::Command);
        self.spi_write(RASET);
        self.register_select(ControlMode::Data);
        // Set y0
        self.spi_write(0x00); // MSB
        self.spi_write(0x00 as u8); // LSB
        // Set y1
        self.spi_write(0x00); // MSB
        self.spi_write((length - 1) as u8); // LSB

        // Write to the display
        self.register_select(ControlMode::Command);
        self.spi_write(RAMWR);
        self.register_select(ControlMode::Data);

        // Fill in display
        for i in 0..length {
            let color = buf[i as usize];

            // Convert RGB 565 to RGB 888
            let red = ((color >> 11) & 0x1F) << 3;
            let green = ((color >> 5)  & 0x3F) << 2;
            let blue = (color & 0x1F) << 3;

            self.spi_write(red as u8);
            self.spi_write(green as u8);
            self.spi_write(blue as u8);
        }

        self.register_select(ControlMode::Command);
        self.chip_select(PinState::Disable);
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

    fn spi_write(&self, byte: u8) {
        // Wait for TX buffer to be empty
        while self.spi.sr.read().txe().bit_is_clear() {}

        self.spi.dr.write(|w| w.dr().bits(byte.into()));

        // Wait for SPI to be busy (TX started)
        while self.spi.sr.read().bsy().bit_is_set() {}
    }

    fn reset(&self, state: PinState) {
        match state {
            PinState::Enable => self.gpio.bsrr.write(|w| w.br1().set_bit()),
            PinState::Disable => self.gpio.bsrr.write(|w| w.bs1().set_bit())
        }
    }

    fn chip_select(&self, state: PinState) {
        match state {
            PinState::Enable => self.gpio.bsrr.write(|w| w.br0().set_bit()),
            PinState::Disable => self.gpio.bsrr.write(|w| w.bs0().set_bit())
        }
    }

    fn register_select(&self, mode: ControlMode) {
        match mode {
            ControlMode::Data => self.gpio.bsrr.write(|w| w.bs4().set_bit()),
            ControlMode::Command => self.gpio.bsrr.write(|w| w.br4().set_bit())
        }
    }
}
