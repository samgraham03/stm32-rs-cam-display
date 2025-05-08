use stm32f4::stm32f401;

use cortex_m::asm;

use crate::{constants::CLK_HZ, display::{ST7735, Display}};

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
    HS  |PB3|HSync (GPIO)
    XCLK|PA8|External clock (MCO_1)
    D6  |PC6|Data[6] (GPIO)
    D4  |PC4|Data[4] (GPIO)
    D2  |PC2|Data[2] (GPIO)
    D0  |PC0|Data[0] (GPIO)
    PWDN|GND|Power down (unused)
*/

pub trait Camera {

    /// Setup and turn on the camera
    fn calibrate(&self);

    fn draw_frame(&self, display: &ST7735);
}

pub struct OV7670<'a> {
    gpioa: &'a stm32f401::GPIOA,
    gpiob: &'a stm32f401::GPIOB,
    gpioc: &'a stm32f401::GPIOC,
    i2c1: stm32f401::I2C1
}

impl<'a> Camera for OV7670<'a> {

    fn calibrate(&self) {

        const COM7_ADDR: u8 = 0x12;
        const COM7_RGB_SELECT: u8 = 0x04;
        const COM7_QVGA_SELECT: u8 = 0x10;
        const COM7_RESET: u8 = 0x80;

        const CLKRC_ADDR: u8 = 0x11;
        const CLKRC_PRESCALER: u8 = 0x01; // CLK = CLK_IN/(PRESCALER+1)

        const COM3_ADDR: u8 = 0x0C;
        const COM3_DCW_EN: u8 = 0x04;

        const COM14_ADDR: u8 = 0x3E;
        const COM14_MANUAL_SCALE_EN: u8 = 0x08;
        const COM14_DCW_AND_PCLK_SCALE_EN: u8 = 0x10;
        const COM14_PCLK_DIVIDER: u8 = 0x01; // Divide by 2

        const SCALING_XSC_ADDR: u8 = 0x70;
        const SCALING_XSC_HORZ_SCALE_FACTOR: u8 = 0x3A; // Default

        const SCALING_YSC_ADDR: u8 = 0x71;
        const SCALING_YSC_VERT_SCALE_FACTOR: u8 = 0x35; // Default

        const SCALING_DCWCTR_ADDR: u8 = 0x72;
        const SCALING_DCWCTR_HORZ_DOWNSAMPLE: u8 = 0x01; // Horizontal downsample by 2
        const SCALING_DCWCTR_VERT_DOWNSAMPLE: u8 = 0x10; // Vertical downsample by 2

        const SCALING_PCLK_DIV_ADDR: u8 = 0x73;
        const SCALING_PCLK_DIV_CLOCK_DIVIDER: u8 = 0x01; // Divide by 2

        const SCALING_PCLK_DELAY_ADDR: u8 = 0xA2;
        const SCALING_PCLK_DELAY_SCALING_OUTPUT_DELAY: u8 = 0x02; // Default

        const COM15_ADDR: u8 = 0x40;
        const COM15_DATA_FORMAT: u8 = 0xC0; // Full ([00] to [FF])
        const COM15_RGB_OPTION: u8 = 0x10; // RGB 565

        const COM8_ADDR: u8 = 0x13;
        const COM8_AWB_ENABLE: u8 = 0x02; // Auto white balance
        const COM8_AEC_ENABLE: u8 = 0x01; // Auto exposure control

        const GAIN_ADDR: u8 = 0x00;
        const GAIN_AGC: u8 = 0xA0; // [00,FF]

        // Reset all registers to default values
        self.sccb_write(COM7_ADDR, COM7_RESET); // COM7: reset
        asm::delay(CLK_HZ / 1000 * 120); // ~120ms

        // Configure OV7670 to use QVGA with downsampling to get 160x120 resolution
        self.sccb_write(COM7_ADDR, COM7_RGB_SELECT | COM7_QVGA_SELECT);
        self.sccb_write(CLKRC_ADDR, CLKRC_PRESCALER);
        self.sccb_write(COM3_ADDR, COM3_DCW_EN);
        self.sccb_write(COM14_ADDR, COM14_MANUAL_SCALE_EN | COM14_DCW_AND_PCLK_SCALE_EN | COM14_PCLK_DIVIDER);
        self.sccb_write(SCALING_XSC_ADDR, SCALING_XSC_HORZ_SCALE_FACTOR);
        self.sccb_write(SCALING_YSC_ADDR, SCALING_YSC_VERT_SCALE_FACTOR);
        self.sccb_write(SCALING_DCWCTR_ADDR, SCALING_DCWCTR_HORZ_DOWNSAMPLE | SCALING_DCWCTR_VERT_DOWNSAMPLE);
        self.sccb_write(SCALING_PCLK_DIV_ADDR, SCALING_PCLK_DIV_CLOCK_DIVIDER);
        self.sccb_write(SCALING_PCLK_DELAY_ADDR, SCALING_PCLK_DELAY_SCALING_OUTPUT_DELAY);
        self.sccb_write(COM15_ADDR, COM15_DATA_FORMAT | COM15_RGB_OPTION);

        // Apply additionaly tuning to improve image quality
        self.sccb_write(COM8_ADDR, COM8_AWB_ENABLE | COM8_AEC_ENABLE);
        self.sccb_write(GAIN_ADDR, GAIN_AGC);
    }

    fn draw_frame(&self, display: &ST7735) {

        // vsync pulses high before a new frame starts
        while !self.read_vsync() {} // wait for vsync rising edge
        while self.read_vsync() {} // wait for vsync falling edge

        // RGB 565 buffer
        let mut buf: [u16; 160] = [0; 160];

        // TODO: dynamically parse rows
        for y in 0..80 {
            let mut x = 0;

            // wait for an hsync rising edge - start of row
            while !self.read_hsync() {};

            while self.read_hsync() {

                // wait for pclk rising edge
                while !self.read_pclk() {}

                let data_msb: u8 = self.read_data();

                // wait for pclk falling edge
                while self.read_pclk() {}

                // wait for pclk rising edge
                while !self.read_pclk() {}

                let data_lsb: u8 = self.read_data();

                // Concat data MSB and LSB
                let data: u16 = ((data_msb as u16) << 8) | (data_lsb as u16);

                if x < 160 {
                    buf[x] = data;
                }

                x += 1;

                while self.read_pclk() {} // wait for pclk falling edge
            }

            display.draw_row(y, &buf);
        }
    }
}

impl<'a> OV7670<'a> {

    const HSI_HZ: usize = 16_000_000;
    const SCL_HZ: usize = 100_000;

    const I2C_ADDR: u8 = 0x21;

    pub fn new(
        rcc: &stm32f401::RCC,
        gpioa: &'a stm32f401::GPIOA,
        gpiob: &'a stm32f401::GPIOB,
        gpioc: &'a stm32f401::GPIOC,
        i2c1: stm32f401::I2C1
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
        gpiob.moder.modify(|_, w| w.moder3().input());

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

        // Enable I2C1 clock
        rcc.apb1enr.modify(|_, w| w.i2c1en().enabled());

        // Specify I2C1 input clock frequency for timing
        i2c1.cr2.modify(|_, w| unsafe { w.freq().bits((OV7670::HSI_HZ / 1_000_000) as u8) });

        // CCR = CLK / (2 × SCL)
        const CCR: usize = OV7670::HSI_HZ / (2 * OV7670::SCL_HZ);

        // Configure I2C1_SCL in standard mode (100KHz)
        i2c1.ccr.modify(|_, w| unsafe {
            w.f_s().clear_bit();
            w.ccr().bits(CCR as u16)
        });

        // trise = CLK[MHz] + 1 (standard mode)
        const TRISE: usize = OV7670::HSI_HZ / 1_000_000 + 1;

        // Configure I2C rise time
        i2c1.trise.modify(|_, w|
            w.trise().bits(TRISE as u8)
        );

        // Enable I2C1
        i2c1.cr1.modify(|_, w| w.pe().enabled());

        OV7670 { gpioa, gpiob, gpioc, i2c1 }
    }

    // Restore I2C bus to IDLE state
    #[allow(dead_code)]
    fn flush_i2c_bus(&self) {

        // Re-configure SCL and SDA as outputs
        self.gpiob.moder.modify(|_, w| {
            w.moder8().output()
             .moder9().output()
        });

        // Attempt to put the bus into the IDLE state (SCL & SDA high)
        self.gpiob.bsrr.write(|w| {
            w.bs8().set_bit()
             .bs9().set_bit()
        });

        // Manually flush the bus if OV7670 is still driving SDA low
        for _ in 0..9 {

            // If SDA is high, the bus is flushed
            if self.gpiob.idr.read().idr9().bit_is_set() {
                break;
            }

            self.gpiob.bsrr.write(|w| w.br8().set_bit()); // SCL low
            asm::delay(1000);

            self.gpiob.bsrr.write(|w| w.bs8().set_bit()); // SCL high
            asm::delay(1000);
        }

        // Generate a manual stop signal (SDA rises while SCL is high)
        self.gpiob.bsrr.write(|w| w.br9().set_bit()); // SDA low
        asm::delay(1000);
        self.gpiob.bsrr.write(|w| w.bs8().set_bit()); // SCL high
        asm::delay(1000);
        self.gpiob.bsrr.write(|w| w.bs9().set_bit()); // SDA high
        asm::delay(1000);

        // Restore SCL as I2C1_SCL
        self.gpiob.moder.modify(|_, w| w.moder8().alternate());
        self.gpiob.afrh.modify(|_, w| w.afrh8().af4());

        // Restore SDA as I2C1_SDA
        self.gpiob.moder.modify(|_, w| w.moder9().alternate());
        self.gpiob.afrh.modify(|_, w| w.afrh9().af4());
    }

    // Issue a register read on the OV7670
    fn sccb_read(&self, addr: u8) -> u8 {

        const READ: u8 = 0x1;
        const WRITE: u8 = 0x0;

        // Send start signal
        self.i2c1.cr1.modify(|_, w| w.start().set_bit());
        while self.i2c1.sr1.read().sb().bit_is_clear() {}

        // Enable OV7670 write mode
        self.i2c1.dr.write(|w| w.dr().bits((OV7670::I2C_ADDR << 1) | WRITE));
        while self.i2c1.sr1.read().addr().bit_is_clear() {}
        self.i2c1.sr2.read().bits(); // Read to clear addr sent flag

        // Write the register address to the bus
        self.i2c1.dr.write(|w| w.dr().bits(addr));
        while self.i2c1.sr1.read().btf().bit_is_clear() {}

        // Send stop signal
        self.i2c1.cr1.modify(|_, w| w.stop().set_bit());

        // Send start signal
        self.i2c1.cr1.modify(|_, w| w.start().set_bit());
        while self.i2c1.sr1.read().sb().bit_is_clear() {}

        // Enable OV7670 read mode
        self.i2c1.dr.write(|w| w.dr().bits((OV7670::I2C_ADDR << 1) | READ));
        while self.i2c1.sr1.read().addr().bit_is_clear() {}
        self.i2c1.sr2.read().bits(); // Read to clear addr sent flag

        // NACK next byte, send stop signal
        self.i2c1.cr1.modify(|_, w| {
            w.ack().clear_bit()
             .stop().set_bit()
        });

        // Wait for data to be ready
        while self.i2c1.sr1.read().rx_ne().bit_is_clear() {}

        // Read data
        self.i2c1.dr.read().dr().bits()
    }

    // Issue a register write on the OV7670
    fn sccb_write(&self, addr: u8, data: u8) {

        const WRITE: u8 = 0x0;

        // Send start signal
        self.i2c1.cr1.modify(|_, w| w.start().set_bit());
        while self.i2c1.sr1.read().sb().bit_is_clear() {}

        // Enable OV7670 write mode
        self.i2c1.dr.write(|w| w.dr().bits((OV7670::I2C_ADDR << 1) | WRITE));
        while self.i2c1.sr1.read().addr().bit_is_clear() {}
        self.i2c1.sr2.read().bits(); // Read to clear addr sent flag

        // Write the register address to the bus
        self.i2c1.dr.write(|w| w.dr().bits(addr));
        while self.i2c1.sr1.read().btf().bit_is_clear() {}

        // Write the data to the bus
        self.i2c1.dr.write(|w| w.dr().bits(data));
        while self.i2c1.sr1.read().btf().bit_is_clear() {}

        // Send stop signal
        self.i2c1.cr1.modify(|_, w| w.stop().set_bit());
    }

    fn read_vsync(&self) -> bool {
        self.gpioa.idr.read().idr6().bit()
    }

    fn read_hsync(&self) -> bool {
        self.gpiob.idr.read().idr3().bit()
    }

    fn read_pclk(&self) -> bool {
        self.gpioa.idr.read().idr9().bit()
    }

    fn read_data(&self) -> u8 {
        self.gpioc.idr.read().bits() as u8
    }
}
