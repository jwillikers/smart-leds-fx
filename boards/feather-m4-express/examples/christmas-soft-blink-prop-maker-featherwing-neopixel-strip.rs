#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::rtt_init_default;

use bsp::entry;
use bsp::hal;
use feather_m4 as bsp;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::ehal::delay::DelayNs;
use hal::ehal::digital::OutputPin;
use hal::pac::{CorePeripherals, Peripherals};
use hal::time::Hertz;
use hal::timer::TimerCounter;
use hal::timer_traits::InterruptDrivenTimer;

use smart_leds::{
    RGB, SmartLedsWrite,
    hsv::{Hsv, hsv2rgb},
};
use ws2812_timer_delay::Ws2812;

use smart_leds_fx::colors::HsColor;
use smart_leds_fx::colors::{CHRISTMAS_GREEN, RED};
use smart_leds_fx::iterators::BrightnessRange;

#[entry]
fn main() -> ! {
    rtt_init_default!();

    const DELAY_MS: u32 = 8;
    const FIRST_LED_COLOR: HsColor<u8> = CHRISTMAS_GREEN;
    const SECOND_LED_COLOR: HsColor<u8> = RED;
    const NUM_LEDS: usize = 30;
    debug_assert_ne!(NUM_LEDS, 0);

    let brightness_range = BrightnessRange::new(1, 254, 1);

    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.gclk,
        &mut peripherals.mclk,
        &mut peripherals.osc32kctrl,
        &mut peripherals.oscctrl,
        &mut peripherals.nvmctrl,
    );
    let pins = bsp::Pins::new(peripherals.port);
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let gclk0 = clocks.gclk0();
    let timer_clock = clocks.tc2_tc3(&gclk0).unwrap();
    let mut timer = TimerCounter::tc3_(&timer_clock, peripherals.tc3, &mut peripherals.mclk);
    timer.start(Hertz::MHz(3).into_duration());

    // The power pin, D10, must be pulled high in order to enable the NeoPixel data line.
    let mut power = pins.d10.into_push_pull_output();
    power.set_high().unwrap();

    // The Prop-Maker FeatherWing uses pin D5 on the Feather M4 Express for the NeoPixel data.
    let neopixel = pins.d5.into_push_pull_output();
    let mut ws = Ws2812::new(timer, neopixel);

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
            let mut data = [first_rgb; NUM_LEDS];
            for led in data.iter_mut().skip(1).step_by(2) {
                *led = second_rgb;
            }
            ws.write(data.iter().cloned()).unwrap();
            delay.delay_ms(DELAY_MS);
        }
    }
}
