#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    main,
    spi::{
        Mode,
        master::{Config, Spi},
    },
    time::Rate,
};

use smart_leds::{
    RGB, RGBW, SmartLedsWrite, White,
    hsv::{Hsv, hsv2rgb},
};
use smart_leds_fx::colors::RED;
use smart_leds_fx::colors::{CHRISTMAS_GREEN, HsColor};
use smart_leds_fx::iterators::BrightnessRange;
use ws2812_spi::Ws2812;

// Required by the ESP-IDF bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    const DELAY_MS: u32 = 4;
    const FIRST_LED_COLOR: HsColor<u8> = CHRISTMAS_GREEN;
    const SECOND_LED_COLOR: HsColor<u8> = RED;
    const NUM_LEDS: usize = 240;

    let brightness_range = BrightnessRange::new(1, 75, 1);

    let peripherals = esp_hal::init(esp_hal::Config::default());

    let spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(Rate::from_khz(3_333))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO6)
    .with_mosi(peripherals.GPIO7)
    .with_miso(peripherals.GPIO2)
    .with_cs(peripherals.GPIO10);

    let delay = Delay::new();

    let mut ws = Ws2812::new_sk6812w(spi);

    loop {
        for j in brightness_range {
            let first_rgb: RGB<u8> = hsv2rgb(Hsv {
                hue: FIRST_LED_COLOR.hue,
                sat: FIRST_LED_COLOR.saturation,
                val: j,
            });
            let second_rgb: RGB<u8> = hsv2rgb(Hsv {
                hue: SECOND_LED_COLOR.hue,
                sat: SECOND_LED_COLOR.saturation,
                val: 254 - j,
            });
            let first_rgbw: RGBW<u8> = RGBW {
                r: first_rgb.r,
                g: first_rgb.g,
                b: first_rgb.b,
                a: White(j / 20),
            };
            let second_rgbw: RGBW<u8> = RGBW {
                r: second_rgb.r,
                g: second_rgb.g,
                b: second_rgb.b,
                a: White((254 - j) / 20),
            };
            let mut data = [first_rgbw; NUM_LEDS];
            for led in data.iter_mut().skip(1).step_by(2) {
                *led = second_rgbw;
            }
            ws.write(data.iter().cloned()).unwrap();
            delay.delay_millis(DELAY_MS);
        }
    }
}
