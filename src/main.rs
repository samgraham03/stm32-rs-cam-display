#![no_std]
#![no_main]

mod constants;
mod usart_debugger;
mod display;
mod camera;

use core::fmt::Write;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4::stm32f401;

use usart_debugger::UsartDebugger;
use display::{Display, ST7735};
use camera::{OV7670};

#[entry]
fn main() -> ! {
    let dp = stm32f401::Peripherals::take().unwrap();

    let rcc = &dp.RCC;
    let gpioa = &dp.GPIOA;
    let gpiob = &dp.GPIOB;
    let gpioc = &dp.GPIOC;

    let mut usart_debugger = UsartDebugger::new(rcc, gpioa, dp.USART2);

    let display = ST7735::new(rcc, gpioa, dp.SPI1, 128, 160);

    let camera = OV7670::new(rcc, gpioa, gpiob, gpioc);

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
