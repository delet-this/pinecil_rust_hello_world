#![no_std]
#![no_main]

use panic_halt as _;

use gd32vf103xx_hal::{self as hal, prelude::*};
use hal::{delay::McycleDelay, time::Bps};
use embedded_hal::digital::v2::{InputPin, OutputPin};

// use ssd1306::{prelude::*, Builder, I2CDIBuilder};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use embedded_graphics::{
    // image::{Image, ImageRaw},
    text::{Alignment, Baseline, Text},
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
};

#[riscv_rt::entry]
fn main() -> ! {
    let p = gd32vf103_pac::Peripherals::take().unwrap();

    // Use external 8MHz HXTAL and set PLL to get 96MHz system clock.
    let mut rcu = p
        .RCU
        .configure()
        .ext_hf_clock(8.mhz())
        .sysclk(96.mhz())
        .freeze();
    let mut afio = p.AFIO.constrain(&mut rcu);

    let mut delay = McycleDelay::new(&rcu.clocks);

    let gpioa = p.GPIOA.split(&mut rcu);
    let gpiob = p.GPIOB.split(&mut rcu);
    // let pa2_tx = gpioa.pa2.into_alternate_push_pull();
    // let pa3_rx = gpioa.pa3.into_pull_up_input();

    // left button
    let btn_b = gpiob.pb0.into_pull_down_input();
    // right button
    // Note that this pin is already pulled low externally via a 10K resistor
    // since it also operates the BOOT0 pin, so we don't need the internal
    // pull-down.
    let btn_a = gpiob.pb1.into_floating_input();

    // OLED reset: Pull low to reset.
    let mut oled_reset = gpioa
        .pa9
        .into_push_pull_output_with_state(hal::gpio::State::Low);

    let pb6_scl = gpiob.pb6.into_alternate_open_drain();
    let pb7_sda = gpiob.pb7.into_alternate_open_drain();

    // Set up i2c.
    let i2c0 = hal::i2c::BlockingI2c::i2c0(
        p.I2C0,
        (pb6_scl, pb7_sda),
        &mut afio,
        hal::i2c::Mode::Fast {
            frequency: 400_000.hz(),
            duty_cycle: hal::i2c::DutyCycle::Ratio2to1,
        },
        &mut rcu,
        1000,
        10,
        1000,
        1000,
    );   

    // OLED datasheet recommends 100 ms delay on power up.
    delay.delay_ms(100);

    // Init OLED.
    oled_reset.set_high().unwrap();

    // OLED datasheet recommends 3 us delay to wait for init.
    delay.delay_us(3);

    let interface = I2CDisplayInterface::new(i2c0);
    let mut disp = Ssd1306::new(interface, DisplaySize96x16, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();
    disp.init().unwrap();

    // disp.set_brightness(Brightness::custom(0xF1, 0x0F_u8));
    
    // Text::with_alignment(
    //     text,
    //     disp.bounding_box().center() + Point::new(0, 15),
    //     character_style,
    //     Alignment::Center,
    // )
    // .draw(&mut disp);
    
    let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let text = "Hello Flashing!";
    let mut x = 0;

    loop {
        disp.clear();
        Text::with_baseline(
            text,
            Point::new(x, 0),
            character_style,
            Baseline::Top,
        )
        .draw(&mut disp).unwrap();

        disp.flush().unwrap();

        x += 1;
        if x > 100 {
            x = -100;
        }
        delay.delay_ms(10);
    }
}
