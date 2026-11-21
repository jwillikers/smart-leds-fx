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

use smart_leds::{RGBW, SmartLedsWrite, White};
use ws2812_spi::Ws2812;

// Required by the ESP-IDF bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    const DELAY_MS: u32 = 4;
    const NUM_LEDS: usize = 240;

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
        let first_rgbw: RGBW<u8> = RGBW {
            r: 0,
            g: 0,
            b: 0,
            a: White(175),
        };
        let data = [first_rgbw; NUM_LEDS];
        ws.write(data.iter().cloned()).unwrap();
        delay.delay_millis(DELAY_MS);
    }
}
