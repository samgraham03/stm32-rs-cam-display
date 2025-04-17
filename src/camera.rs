use stm32f4::stm32f401;

/*
    OV7670 Camera

    CON |PIN|NOTE
    ==============
    3.3V|3.3|
    SCL |PB8|SCCB clock (I2C_SCL)
    VS  |PA6|Vsync (GPIO)
    PCLK|PA9|Pixel Clock (GPIO)
    D7  |PC7|Data[7] (GPIO)
    D5  |PC5|Data[5] (GPIO)
    D3  |PC3|Data[3] (GPIO)
    D1  |PC1|Data[1] (GPIO)
    RET |3.3|Reset (unused)
    DGND|GND|
    SDA |PB9|SCCB data (I2C_SDA)
    HS  |PA3|HSync (GPIO)
    XCLK|PA8|External clock ()
    D6  |PC6|Data[6] (GPIO)
    D4  |PC4|Data[4] (GPIO)
    D2  |PC2|Data[2] (GPIO)
    D0  |PC0|Data[0] (GPIO)
    PWDN|GND|Power down (unused)
*/

pub struct OV7670<'a> {
    gpioa: &'a stm32f401::GPIOA,
    gpiob: &'a stm32f401::GPIOB,
    gpioc: &'a stm32f401::GPIOC
}

impl<'a> OV7670<'a> {

    pub fn new(
        rcc: &stm32f401::RCC,
        gpioa: &'a stm32f401::GPIOA,
        gpiob: &'a stm32f401::GPIOB,
        gpioc: &'a stm32f401::GPIOC
    ) -> Self {

        // Enable GPIOC clock
        rcc.ahb1enr.modify(|_, w| w.gpiocen().enabled());

        // Configure data pins as inputs
        gpioc.moder.modify(|_, w| {
            w.moder0().input()
             .moder1().input()
             .moder2().input()
             .moder3().input()
             .moder4().input()
             .moder5().input()
             .moder6().input()
             .moder7().input()
        });

        OV7670 { gpioa, gpiob, gpioc }
    }
}
