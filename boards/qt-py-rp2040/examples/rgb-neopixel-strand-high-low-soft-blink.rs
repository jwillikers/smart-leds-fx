#![no_std]
#![no_main]

use embedded_hal_0_2::timer::CountDown;
use fugit::MicrosDurationU32;
use panic_halt as _;
use smart_leds::{
    RGB, SmartLedsWrite,
    hsv::{Hsv, hsv2rgb},
};
use smart_leds_fx::iterators::BrightnessRange;
use smart_leds_fx_qt_py_rp2040::Pins;
use smart_leds_fx_qt_py_rp2040::XOSC_CRYSTAL_FREQ;
use smart_leds_fx_qt_py_rp2040::entry;
use smart_leds_fx_qt_py_rp2040::hal::clocks::init_clocks_and_plls;
use smart_leds_fx_qt_py_rp2040::hal::prelude::*;
use smart_leds_fx_qt_py_rp2040::hal::sio::Sio;
use smart_leds_fx_qt_py_rp2040::hal::{Timer, Watchdog, pac};
use ws2812_pio::Ws2812;

use smart_leds_fx::colors::RED;
use smart_leds_fx::colors::{CHRISTMAS_GREEN, HsColor};

#[entry]
fn main() -> ! {
    const DELAY: MicrosDurationU32 = MicrosDurationU32::millis(15);
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

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut delay = timer.count_down();

    // GPIO 26
    let data_pin = pins.a3.into_function();
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
                *led = second_rgb;
            }
            ws.write(data.iter().cloned()).unwrap();
            delay.start(DELAY);
            let _ = nb::block!(delay.wait());
        }
    }
}
