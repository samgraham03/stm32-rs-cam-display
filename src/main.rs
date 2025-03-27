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

fn setup_usart2(
    rcc : &stm32f401::RCC,
    gpioa : &stm32f401::GPIOA,
    usart2 : &stm32f401::USART2
) {
    // Enable GPIOA and USART2 clocks
    rcc.ahb1enr.modify(|_, w| w.gpioaen().enabled());
    rcc.apb1enr.modify(|_, w| w.usart2en().enabled());

    // Set PA2 to USART2_TX alt function
    gpioa.moder.modify(|_, w| w.moder2().alternate());
    gpioa.afrl.modify(|_, w| w.afrl2().af7());

    // Set baud rate and enable USART2 Tx
    usart2.brr.write(|w| unsafe { w.bits(CLK_HZ/BAUD_RATE) });
    usart2.cr1.modify(|_, w| w.ue().enabled().te().enabled());
}

#[entry]
fn main() -> ! {
    let dp = stm32f401::Peripherals::take().unwrap();

    let rcc = &dp.RCC;
    let gpioa = &dp.GPIOA;
    let usart2 = &dp.USART2;

    // Setup USART2 for dubug logging
    setup_usart2(rcc, gpioa, usart2);

    loop {
        debug_print(usart2, "Hello World\r\n");

        for _ in 0..1_000_000 {
            asm::nop();
        }
    }
}
