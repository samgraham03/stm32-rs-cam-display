use stm32f4::stm32f401;
use core::fmt;

use super::constants::{BAUD_RATE, CLK_HZ};

/*
    USART over USB

    CON|PIN|NOTE
    ==================
    TX |PA2|USART2_TX
*/

pub struct UsartDebugger {
    usart: stm32f401::USART2
}

impl UsartDebugger {

    pub fn new(
        rcc: &stm32f401::RCC,
        gpioa: &stm32f401::GPIOA,
        usart2: stm32f401::USART2,
    ) -> Self {

        // Enable GPIOA clock
        rcc.ahb1enr.modify(|_, w| w.gpioaen().enabled());

        // Configure TX pin to use an alternate function
        gpioa.moder.modify(|_, w| w.moder2().alternate());

        // Set PA2 to use USART2_TX
        gpioa.afrl.modify(|_, w| w.afrl2().af7());

        // Enable USART2 clock
        rcc.apb1enr.modify(|_, w| w.usart2en().enabled());

        // Set baud rate
        usart2.brr.write(|w| unsafe { w.bits(CLK_HZ/BAUD_RATE) });

        // Enable USART2 TX
        usart2.cr1.modify(|_, w| w.ue().enabled().te().enabled());

        UsartDebugger { usart: usart2 }
    }
}

impl fmt::Write for UsartDebugger {

    fn write_str(&mut self, s: &str) -> fmt::Result {

        for byte in s.bytes() {

            // Wait for TX buffer to be empty
            while self.usart.sr.read().txe().bit_is_clear() {}

            // Write to data register
            self.usart.dr.write(|w| unsafe { w.bits(byte.into()) });
        }

        Ok(())
    }
}
