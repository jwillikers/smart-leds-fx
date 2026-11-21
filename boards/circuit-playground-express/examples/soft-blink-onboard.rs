#![no_main]
#![no_std]

use panic_halt as _;

use bsp::entry;
use bsp::hal;
use circuit_playground_express as bsp;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::ehal::delay::DelayNs;
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
use smart_leds_fx::colors::RESTFUL_ORANGE;
use smart_leds_fx::iterators::BrightnessRange;

#[entry]
fn main() -> ! {
    const DELAY_MS: u32 = 8;
    const LED_COLOR: HsColor<u8> = RESTFUL_ORANGE;
    const NUM_LEDS: usize = 10;
    debug_assert_ne!(NUM_LEDS, 0);

    let brightness_range = BrightnessRange::new(1, 254, 1);

    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.gclk,
        &mut peripherals.pm,
        &mut peripherals.sysctrl,
        &mut peripherals.nvmctrl,
    );
    let pins = bsp::Pins::new(peripherals.port);
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let gclk0 = clocks.gclk0();
    let timer_clock = clocks.tcc2_tc3(&gclk0).unwrap();
    let mut timer = TimerCounter::tc3_(&timer_clock, peripherals.tc3, &mut peripherals.pm);
    timer.start(Hertz::MHz(3).into_duration());

    let neopixel_pin: bsp::NeoPixel = pins.d8.into();
    let mut ws = Ws2812::new(timer, neopixel_pin);

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
