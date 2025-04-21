use stm32f4::stm32f401;

/*
    OV7670 Camera

    CON |PIN|NOTE
    ==============
    3.3V|3.3|
    SCL |PB8|SCCB clock (I2C1_SCL)
    VS  |PA6|Vsync (GPIO)
    PCLK|PA9|Pixel Clock (GPIO)
    D7  |PC7|Data[7] (GPIO)
    D5  |PC5|Data[5] (GPIO)
    D3  |PC3|Data[3] (GPIO)
    D1  |PC1|Data[1] (GPIO)
    RET |3.3|Reset (unused)
    DGND|GND|
    SDA |PB9|SCCB data (I2C1_SDA)
    HS  |PA3|HSync (GPIO)
    XCLK|PA8|External clock (MCO_1)
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

        // Enable GPIOA, GPIOB, GPIOC clocks
        rcc.ahb1enr.modify(|_, w| {
            w.gpioaen().enabled()
             .gpioben().enabled()
             .gpiocen().enabled()
        });

        // Configure I2C bus to use open-drain
        gpiob.otyper.modify(|_, w| {
            w.ot8().open_drain()
             .ot9().open_drain()
        });

        // Add pull-up resistors to I2C control pins
        gpiob.pupdr.modify(|_, w| {
            w.pupdr8().pull_up()
             .pupdr9().pull_up()
        });

        // Configure SCL (I2C1_SCL)
        gpiob.moder.modify(|_, w| w.moder8().alternate());
        gpiob.afrh.modify(|_, w| w.afrh8().af4());

        // Configure SDA (I2C1_SDA)
        gpiob.moder.modify(|_, w| w.moder9().alternate());
        gpiob.afrh.modify(|_, w| w.afrh9().af4());

        // Configure data pins (GPIO)
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

        // Configure VSYNC (GPIO)
        gpioa.moder.modify(|_, w| w.moder6().input());

        // Configure HSYNC (GPIO)
        gpioa.moder.modify(|_, w| w.moder3().input());

        // Configure PCLK (GPIO)
        gpioa.moder.modify(|_, w| w.moder9().input());

        // Configure XCLK (MSO_1)
        gpioa.moder.modify(|_, w| w.moder8().alternate());
        gpioa.afrh.modify(|_, w| w.afrh8().af0());

        // Enable HSI (16 MHz clock)
        rcc.cr.modify(|_, w| w.hsion().on());
        while rcc.cr.read().hsirdy().is_not_ready() {}

        // Select HSI as XCLK source
        rcc.cfgr.modify(|_, w| {
            w.mco1().hsi()
             .mco1pre().div1()
        });

        OV7670 { gpioa, gpiob, gpioc }
    }
}
