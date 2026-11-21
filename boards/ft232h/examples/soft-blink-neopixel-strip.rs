use ftdi_embedded_hal as hal;
use hal::Delay;
use hal::eh1::delay::DelayNs;
use hal::eh1::spi::Polarity;
use smart_leds::{
    RGB8, SmartLedsWrite,
    hsv::{Hsv, hsv2rgb},
};
use smart_leds_fx::colors::HsColor;
use smart_leds_fx::colors::RESTFUL_ORANGE;
use smart_leds_fx::iterators::BrightnessRange;
use ws2812_spi::Ws2812;

fn main() -> ! {
    const DELAY_MS: u32 = 8;
    const LED_COLOR: HsColor<u8> = RESTFUL_ORANGE;
    const NUM_LEDS: usize = 30;
    debug_assert_ne!(NUM_LEDS, 0);

    let brightness_range = BrightnessRange::new(1, 254, 1);

    let mut delay = Delay::new();

    // AD0 => SCK
    // AD1 => MOSI
    // AD2 => MISO
    let device = hal::ftdi::find_by_vid_pid(0x0403, 0x6014)
        .interface(hal::ftdi::Interface::A)
        .open()
        .unwrap();
    let ft = hal::FtHal::init_freq(device, 3_000_000).unwrap();
    let mut spi = ft.spi().unwrap();
    spi.set_clock_polarity(Polarity::IdleLow).unwrap();

    let mut neopixel = Ws2812::new(spi);

    loop {
        for j in brightness_range {
            let rgb: RGB8 = hsv2rgb(Hsv {
                hue: LED_COLOR.hue,
                sat: LED_COLOR.saturation,
                val: j,
            });
            let data = [rgb; NUM_LEDS];
            neopixel.write(data.iter().cloned()).unwrap();
            delay.delay_ms(DELAY_MS);
        }
    }
}
