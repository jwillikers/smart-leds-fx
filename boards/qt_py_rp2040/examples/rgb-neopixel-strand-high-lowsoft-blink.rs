#![no_std]
#![no_main]

use adafruit_qt_py_rp2040::entry;
use embedded_hal::timer::CountDown;
use fugit::Duration;
use panic_halt as _;
use adafruit_qt_py_rp2040::hal::{pac, Timer, Watchdog};
use adafruit_qt_py_rp2040::hal::prelude::*;
use adafruit_qt_py_rp2040::hal::{sio::Sio};
use adafruit_qt_py_rp2040::hal::clocks::init_clocks_and_plls;
use adafruit_qt_py_rp2040::Pins;
use adafruit_qt_py_rp2040::XOSC_CRYSTAL_FREQ;
use smart_leds::{
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, RGB,
};
use smart_leds_fx::iterators::BrightnessRange;
use ws2812_pio::Ws2812;

use smart_leds_fx::colors::RED;
use smart_leds_fx::colors::{HsColor, CHRISTMAS_GREEN};


#[entry]
fn main() -> ! {
    const DELAY: Duration<u32, 1, 1_000_000> = Duration::<u32, 1, 1_000_000>::from_ticks(15_000u32);
    const FIRST_LED_COLOR: HsColor<u8> = CHRISTMAS_GREEN;
    const SECOND_LED_COLOR: HsColor<u8> = RED;
    const NUM_LEDS: usize = 30;
    debug_assert_ne!(NUM_LEDS, 0);

    let brightness_range = BrightnessRange::new(1, 75, 1);

    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
        .ok()
        .unwrap();

    let sio = Sio::new(pac.SIO);

    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut delay = timer.count_down();

    // GPIO 26
    let data_pin = pins.a3.into_mode();
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let mut ws = Ws2812::new(
        data_pin,
        &mut pio,
        sm0,
        clocks.peripheral_clock.freq(),
        timer.count_down(),
    );

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
                val: 75 - j,
            });
            let mut data = [first_rgb; NUM_LEDS];
            for led in data.iter_mut().skip(1).step_by(2) {
                *led = second_rgb.clone();
            }
            ws.write(data.iter().cloned()).unwrap();
            delay.start(DELAY);
            let _ = nb::block!(delay.wait());
        }
    }
}
