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

fn configure_peripherals(
    rcc : &stm32f401::RCC,
    gpioa : &stm32f401::GPIOA,
    usart2 : &stm32f401::USART2
) {

    // Enable GPIOA and USART2 clocks
    rcc.ahb1enr.modify(|_, w| w.gpioaen().enabled());
    rcc.apb1enr.modify(|_, w| w.usart2en().enabled());

    // Configure pin modes
    gpioa.moder.modify(|_, w| {
        w.moder0().output()
         .moder1().output()
         .moder2().alternate()
         .moder4().output()
    });

    // Set PA2 to use USART2_TX
    gpioa.afrl.modify(|_, w| w.afrl2().af7());

    /*
        USART over USB

        CON|PIN|NOTE
        ==================
        TX |PA2|USART2_TX
    */

    // Set baud rate and enable USART2 Tx
    usart2.brr.write(|w| unsafe { w.bits(CLK_HZ/BAUD_RATE) });
    usart2.cr1.modify(|_, w| w.ue().enabled().te().enabled());

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

    configure_peripherals(rcc, gpioa, usart2);

    loop {
        debug_print(usart2, "Hello World\r\n");

        // ~1s delay
        asm::delay(CLK_HZ);
    }
}
