#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

use embedded_hal::delay::DelayNs;
use nrf52840_hal::{
    Timer,
    gpio::{Level, p0::Parts},
    pac::Peripherals,
    spi,
};

use smart_leds::{
    RGB, SmartLedsWrite,
    hsv::{Hsv, hsv2rgb},
};
use ws2812_spi::Ws2812;

use smart_leds_fx::colors::HsColor;
use smart_leds_fx::colors::RESTFUL_ORANGE;
use smart_leds_fx::iterators::BrightnessRange;

#[entry]
fn main() -> ! {
    const DELAY_MS: u32 = 8;
    const LED_COLOR: HsColor<u8> = RESTFUL_ORANGE;
    const NUM_LEDS: usize = 10;
    debug_assert_ne!(NUM_LEDS, 0);

    let brightness_range = BrightnessRange::new(1, 254, 1);

    let dp = Peripherals::take().unwrap();
    let mut delay = Timer::new(dp.TIMER0);
    let port0 = Parts::new(dp.P0);

    let _power_switch = port0.p0_06.into_push_pull_output(Level::Low);

    // The onboard NeoPixels are attached to P0.13.
    // Drive them from the MOSI line of the SPI peripheral.
    let mosi = port0.p0_13.into_push_pull_output(Level::Low).degrade();
    let pins = spi::Pins {
        sck: None,
        miso: None,
        mosi: Some(mosi),
    };
    let spi = spi::Spi::new(dp.SPI0, pins, spi::Frequency::M2, embedded_hal::spi::MODE_0);

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
            delay.delay_ms(DELAY_MS);
        }
    }
}
