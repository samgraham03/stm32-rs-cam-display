#![no_std]
#![no_main]

use cortex_m::asm;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f401;

const BAUD_RATE: u32 = 115_200;
const CLK_HZ: u32 = 16_000_000;

/*
    Issues:
    * Display only seems to accept pixel data after calling INVON or INVOFF first.
    * Color mode is not being set to 16-bit RGB correcly. Currently 24-bit RGB
    * Disabling CS right after writing a command seems to cause the command to be dropped.
        Shouldn't happen because we wait for busy bit in spi driver
*/

fn spi1_write(spi1: &stm32f401::SPI1, byte: u8) {
    // Wait for TX buffer to be empty
    while spi1.sr.read().txe().bit_is_clear() {}

    spi1.dr.write(|w| w.dr().bits(byte.into()));

    // Wait for SPI to be busy (TX started)
    while spi1.sr.read().bsy().bit_is_set() {}
}

fn debug_print(usart2: &stm32f401::USART2, s: &str) {

    for byte in s.bytes() {
        // Wait for TX buffer to be empty
        while usart2.sr.read().txe().bit_is_clear() {}

        usart2.dr.write(|w| unsafe { w.bits(byte.into()) });
    }
}

fn configure_usart2_debugger(
    rcc : &stm32f401::RCC,
    gpioa : &stm32f401::GPIOA,
    usart2 : &stm32f401::USART2
) {
    /*
        USART over USB

        CON|PIN|NOTE
        ==================
        TX |PA2|USART2_TX
    */

    // Enable GPIOA clock
    rcc.ahb1enr.modify(|_, w| w.gpioaen().enabled());

    // Configure TX pin to use an alternate function
    gpioa.moder.modify(|_, w| w.moder2().alternate());

    // Set PA2 to use USART2_TX
    gpioa.afrl.modify(|_, w| w.afrl2().af7());

    // Enable USART2 clock
    rcc.apb1enr.modify(|_, w| w.usart2en().enabled());

    // Set baud rate and enable USART2 Tx
    usart2.brr.write(|w| unsafe { w.bits(CLK_HZ/BAUD_RATE) });
    usart2.cr1.modify(|_, w| w.ue().enabled().te().enabled());
}

// TODO: Display module
// RST: enable=0, disable=1
fn display_rst_enable(gpioa : &stm32f401::GPIOA) {
    gpioa.bsrr.write(|w| w.br1().set_bit()); //=0
}
fn display_rst_disable(gpioa : &stm32f401::GPIOA) {
    gpioa.bsrr.write(|w| w.bs1().set_bit()); //=1
}
// CS: enable=0, disable=1
fn display_cs_enable(gpioa : &stm32f401::GPIOA) {
    gpioa.bsrr.write(|w| w.br0().set_bit()); //=0
}
fn display_cs_disable(gpioa : &stm32f401::GPIOA) {
    gpioa.bsrr.write(|w| w.bs0().set_bit()); //=1
}
// RS: cmd=1, data=0
fn display_rs_command_mode(gpioa : &stm32f401::GPIOA) {
    // gpioa.bsrr.write(|w| w.bs4().set_bit()); //=1
    gpioa.bsrr.write(|w| w.br4().set_bit()); //=0
}
fn display_rs_data_mode(gpioa : &stm32f401::GPIOA) {
    // gpioa.bsrr.write(|w| w.br4().set_bit()); //=0
    gpioa.bsrr.write(|w| w.bs4().set_bit()); //=1
}

fn clear_display(
    gpioa : &stm32f401::GPIOA,
    spi1 : &stm32f401::SPI1,
    usart2 : &stm32f401::USART2
) {
    const WIDTH: u8 = 128;
    const HEIGHT: u8 = 160;

    const CASET: u8 = 0x2A;
    const RASET: u8 = 0x2B;
    const RAMWR: u8 = 0x2C;

    display_cs_enable(gpioa);

    // TODO: Fails without this... why?
    display_rs_command_mode(gpioa);
    spi1_write(spi1, 0x20); // INVOFF

    // Set column range
    display_rs_command_mode(gpioa);
    spi1_write(spi1, CASET);
    display_rs_data_mode(gpioa);
    // Set x0
    spi1_write(spi1, 0x00); // MSB
    spi1_write(spi1, 0x00); // LSB
    // Set x1
    spi1_write(spi1, 0x00); // MSB
    spi1_write(spi1, (WIDTH - 1) as u8); // LSB

    // Set row range
    display_rs_command_mode(gpioa);
    spi1_write(spi1, RASET);
    display_rs_data_mode(gpioa);
    // Set y0
    spi1_write(spi1, 0x00); // MSB
    spi1_write(spi1, 0x00); // LSB
    // Set y1
    spi1_write(spi1, 0x00); // MSB
    spi1_write(spi1, (HEIGHT - 1) as u8); // LSB

    // Write to the display
    display_rs_command_mode(gpioa);
    spi1_write(spi1, RAMWR);
    display_rs_data_mode(gpioa);

    // Fill the display in with white
    for _ in 0..HEIGHT {
        for _ in 0..WIDTH {
            spi1_write(spi1, 0xFF);
            spi1_write(spi1, 0xFF);
            spi1_write(spi1, 0xFF);
        }
    }

    // TESTING Color lop
    loop {
        // Slowly draw to the display
        for _ in 0..HEIGHT {
            for _ in 0..WIDTH {
                spi1_write(spi1, 0x00); // R
                spi1_write(spi1, 0x00); // G
                spi1_write(spi1, 0xFF); // B
                // asm::delay(CLK_HZ/1000); // ~1ms
            }
            debug_print(usart2, "ROW\r\n");
            // asm::delay(CLK_HZ/10); // ~100ms
        }

        // Slowly draw to the display
        for _ in 0..HEIGHT {
            for _ in 0..WIDTH {
                spi1_write(spi1, 0x00); // R
                spi1_write(spi1, 0xFF); // G
                spi1_write(spi1, 0x00); // B
                // asm::delay(CLK_HZ/1000); // ~1ms
            }
            debug_print(usart2, "ROW\r\n");
            // asm::delay(CLK_HZ/10); // ~100ms
        }

        // Slowly draw to the display
        for _ in 0..HEIGHT {
            for _ in 0..WIDTH {
                spi1_write(spi1, 0xFF); // R
                spi1_write(spi1, 0x00); // G
                spi1_write(spi1, 0x00); // B
                // asm::delay(CLK_HZ/1000); // ~1ms
            }
            debug_print(usart2, "ROW\r\n");
            // asm::delay(CLK_HZ/10); // ~100ms
        }
    }

    display_rs_command_mode(gpioa);
    display_cs_disable(gpioa);
}

fn calibrate_display(
    gpioa: &stm32f401::GPIOA,
    spi1: &stm32f401::SPI1
) {
    const SWRESET: u8 = 0x01;
    const SLPOUT: u8 = 0x11;
    const COLMOD: u8 = 0x3A;
    const DISPON: u8 = 0x29;

    const COLOUR_16_BIT_MODE: u8 = 0x05;

    display_cs_enable(gpioa);

    // Software reset
    display_rs_command_mode(gpioa);
    spi1_write(spi1, SWRESET);
    asm::delay(CLK_HZ / 1000 * 120); // ~120ms

    // Wake up display (from reset sleep)
    spi1_write(spi1, SLPOUT);
    asm::delay(CLK_HZ / 1000 * 120); // ~120ms

    // Set color mode
    spi1_write(spi1, COLMOD);
    display_rs_data_mode(gpioa);
    spi1_write(spi1, COLOUR_16_BIT_MODE);

    // Turn on the display
    display_rs_command_mode(gpioa);
    spi1_write(spi1, DISPON);
    asm::delay(CLK_HZ / 1000 * 120); // ~120ms

    display_cs_disable(gpioa);
}

fn configure_st7735_display(
    rcc: &stm32f401::RCC,
    gpioa: &stm32f401::GPIOA,
    spi1: &stm32f401::SPI1,
    usart2 : &stm32f401::USART2
) {
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

    // Enable GPIOA clock
    rcc.ahb1enr.modify(|_, w| w.gpioaen().enabled());

    // Configure output pins
    gpioa.moder.modify(|_, w| {
        w.moder0().output() // CS
         .moder1().output() // RST
         .moder4().output() // RS
    });

    // Disable Chip Select (CS)
    display_cs_disable(gpioa);

    // Enable Reset (RST)
    display_rst_enable(gpioa);

    // ~120ms delay
    asm::delay(CLK_HZ / 1000 * 120);

    // Disable Reset (RST)
    display_rst_disable(gpioa);

    // Put Register Select (RS) into command mode
    display_rs_command_mode(gpioa);

    // ~120ms delay
    asm::delay(CLK_HZ / 1000 * 120);

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

    calibrate_display(gpioa, spi1);

    asm::delay(CLK_HZ / 2); // 500ms

    clear_display(gpioa, spi1, usart2); // TODO: move this
    asm::delay(CLK_HZ * 3); // TESTING
}

fn configure_microsd_interface() {
    /*
        MicroSD card reader/writer

        CON |PIN|NOTE
        =============
        CS  |
        MOSI|
        SCK |
        MISO|
    */

    // TODO
}

fn configure_ov7670_camera() {
    /*
        OV7670 Camera

        CON |PIN|NOTE
        =============
        3.3V|
        SCL |
        VS  |
        PLK |
        D7  |
        D5  |
        D3  |
        D1  |
        RET |
        DGND|
        SDA |
        HS  |
        XLK |
        D6  |
        D4  |
        D2  |
        D0  |
        PWDN|
    */

    // TODO
}

#[entry]
fn main() -> ! {
    let dp = stm32f401::Peripherals::take().unwrap();

    let rcc = &dp.RCC;
    let gpioa = &dp.GPIOA;
    let usart2 = &dp.USART2;
    let spi1 = &dp.SPI1;

    configure_usart2_debugger(rcc, gpioa, usart2);

    debug_print(usart2, "\r\n"); // DEBUG

    configure_st7735_display(rcc, gpioa, spi1, usart2);

    configure_microsd_interface();

    configure_ov7670_camera();

    loop {
        debug_print(usart2, "Hello World\r\n");

        // ~1s delay
        asm::delay(CLK_HZ);
    }
}
