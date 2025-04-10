#![no_std]
#![no_main]

mod constants;
mod usart_debugger;
mod display;

use core::fmt::Write;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f401;

use usart_debugger::UsartDebugger;
use display::{Display, ST7735};

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

    let mut usart_debugger = UsartDebugger::new(rcc, gpioa, dp.USART2);
    let display = ST7735::new(rcc, gpioa, dp.SPI1, 128, 160);

    configure_microsd_interface();

    configure_ov7670_camera();

    write!(usart_debugger, "Calibrating display\r\n").unwrap();

    display.calibrate();

    write!(usart_debugger, "Entering color loop\r\n").unwrap();

    loop {
        const COLOR_RED: u32 = 0xFF0000;
        const COLOR_GREEN: u32 = 0x00FF00;
        const COLOR_BLUE: u32 = 0x0000FF;

        display.fill(Some(COLOR_RED));
        display.fill(Some(COLOR_GREEN));
        display.fill(Some(COLOR_BLUE));
    }
}
