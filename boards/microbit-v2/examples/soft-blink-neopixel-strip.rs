#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_probe as _;

use microbit::{
    hal::{
        gpio::{p0::Parts, Level},
        prelude::*,
        spi, Timer,
    },
    Peripherals,
};

use embedded_time::duration::*;
use smart_leds::{
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, RGB,
};
use ws2812_spi::Ws2812;

use smart_leds_fx::colors::HsColor;
use smart_leds_fx::colors::RESTFUL_ORANGE;
use smart_leds_fx::iterators::BrightnessRange;

#[entry]
fn main() -> ! {
    const DELAY: Milliseconds<u32> = Milliseconds::<u32>(8);
    const LED_COLOR: HsColor<u8> = RESTFUL_ORANGE;
    const NUM_LEDS: usize = 30;

    let brightness_range = BrightnessRange::new(1, 254, 1);

    let dp = Peripherals::take().unwrap();
    let mut delay = Timer::new(dp.TIMER0);
    let port0 = Parts::new(dp.P0);
    let sck = port0.p0_17.into_push_pull_output(Level::Low).degrade();
    let mosi = port0.p0_13.into_push_pull_output(Level::Low).degrade();
    let miso = port0.p0_01.into_floating_input().degrade();
    let pins = spi::Pins {
        sck,
        miso: Some(miso),
        mosi: Some(mosi),
    };
    let spi = spi::Spi::new(dp.SPI0, pins, spi::Frequency::M2, spi::MODE_0);

    let mut ws = Ws2812::new(spi);

    loop {
        for j in brightness_range {
            let rgb: RGB<u8> = hsv2rgb(Hsv {
                hue: LED_COLOR.hue,
                sat: LED_COLOR.saturation,
                val: j,
            });
            let data = [rgb; NUM_LEDS];
            ws.write(data.iter().cloned()).unwrap();
            delay.delay_ms(DELAY.integer());
        }
    }
}
