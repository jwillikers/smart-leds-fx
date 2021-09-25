#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::*;
use defmt::debug_assert_ne;
use defmt_rtt as _;
use embedded_hal::spi::MODE_0;
use embedded_time::duration::Milliseconds;
use embedded_time::fixed_point::FixedPoint;
use embedded_time::rate::Extensions;
use panic_probe as _;
use pico::hal;
use pico::hal::pac;
use pico::hal::prelude::*;
use pico::hal::{gpio::FunctionSpi, sio::Sio, spi::Spi};
use smart_leds::{
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, White, RGB, RGBW,
};
use smart_leds_fx::colors::HsColor;
use smart_leds_fx::colors::RESTFUL_ORANGE;
use smart_leds_fx::iterators::BrightnessRange;
use ws2812_spi::Ws2812;

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER;

const SYS_HZ: u32 = 125_000_000_u32;

#[entry]
fn main() -> ! {
    info!("Program start");

    const DELAY: Milliseconds<u32> = Milliseconds::<u32>(8);
    const LED_COLOR: HsColor<u8> = RESTFUL_ORANGE;
    const NUM_LEDS: usize = 8;
    debug_assert_ne!(NUM_LEDS, 0);

    let brightness_range = BrightnessRange::new(1, 254, 1);

    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::watchdog::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // Our default is 12 MHz crystal input, 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
        pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

    // The single-cycle I/O block controls our GPIO pins
    let sio = Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // These are implicitly used by the spi driver if they are in the correct mode
    let _spi_sclk = pins.gpio6.into_mode::<FunctionSpi>();
    let _spi_mosi = pins.gpio7.into_mode::<FunctionSpi>();
    let _spi_miso = pins.gpio4.into_mode::<FunctionSpi>();
    let spi = Spi::<_, _, 8>::new(pac.SPI0).init(
        &mut pac.RESETS,
        SYS_HZ.Hz(),
        3_000_000u32.Hz(),
        &MODE_0,
    );

    let mut ws = Ws2812::new_sk6812w(spi);

    loop {
        for j in brightness_range {
            let rgb: RGB<u8> = hsv2rgb(Hsv {
                hue: LED_COLOR.hue,
                sat: LED_COLOR.saturation,
                val: j,
            });
            let rgbw: RGBW<u8> = RGBW {
                r: rgb.r,
                g: rgb.g,
                b: rgb.b,
                a: White(j / 20),
            };
            let data = [rgbw; NUM_LEDS];
            ws.write(data.iter().cloned()).unwrap();
            delay.delay_ms(DELAY.integer());
        }
    }
}
