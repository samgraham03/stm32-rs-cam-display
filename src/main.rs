#![no_std]
#![no_main]

use cortex_m::asm;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f401;

const BAUD_RATE : u32 = 115_200;
const CLK_HZ : u32 = 16_000_000;

fn debug_print(usart2 : &stm32f401::USART2, s : &str) {

    for byte in s.bytes() {
        // Wait for Tx to be ready
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

fn configure_st7735_display(
    rcc : &stm32f401::RCC,
    gpioa : &stm32f401::GPIOA
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
         .moder5().alternate() // CLK
    });

    // Disable Chip Select (CS)
    gpioa.bsrr.write(|w| w.bs0().set_bit());

    // Enable Reset (RST)
    gpioa.bsrr.write(|w| w.br1().set_bit());

    // ~120ms delay
    asm::delay(CLK_HZ / 1000 * 120);

    // Disable Reset (RST)
    gpioa.bsrr.write(|w| w.bs1().set_bit());

    // Put Register Select (RS) into command mode
    gpioa.bsrr.write(|w| w.br4().set_bit());

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
        w.afrl7().af5() // SPI1_SCK
         .afrl2().af7() // SPI1_MOSI
    });
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

fn configure_peripherals(
    rcc : &stm32f401::RCC,
    gpioa : &stm32f401::GPIOA,
    usart2 : &stm32f401::USART2
) {
    configure_usart2_debugger(rcc, gpioa, usart2);

    configure_st7735_display(rcc, gpioa);

    configure_microsd_interface();

    configure_ov7670_camera();
}

#[entry]
fn main() -> ! {
    let dp = stm32f401::Peripherals::take().unwrap();

    let rcc = &dp.RCC;
    let gpioa = &dp.GPIOA;
    let usart2 = &dp.USART2;

    configure_peripherals(rcc, gpioa, usart2);

    display_hello_world();

    loop {
        debug_print(usart2, "Hello World\r\n");

        // ~1s delay
        asm::delay(CLK_HZ);
    }
}
