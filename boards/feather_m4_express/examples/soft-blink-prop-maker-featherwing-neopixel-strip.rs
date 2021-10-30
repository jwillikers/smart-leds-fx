#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::rtt_init_default;

use embedded_time::duration::*;
use feather_m4 as bsp;
use bsp::entry;
use bsp::hal;
use hal::clock::GenericClockController;
use hal::prelude::*;
use hal::delay::Delay;
use hal::pac::{CorePeripherals, Peripherals};
use hal::thumbv7em::timer::TimerCounter;
use hal::time::U32Ext;

use smart_leds::{
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, RGB,
};
use ws2812_timer_delay::Ws2812;

use smart_leds_fx::colors::HsColor;
use smart_leds_fx::colors::RESTFUL_ORANGE;
use smart_leds_fx::iterators::BrightnessRange;

#[entry]
fn main() -> ! {
    rtt_init_default!();

    const DELAY: Milliseconds<u32> = Milliseconds::<u32>(8);
    const LED_COLOR: HsColor<u8> = RESTFUL_ORANGE;
    const NUM_LEDS: usize = 30;
    debug_assert_ne!(NUM_LEDS, 0);

    let brightness_range = BrightnessRange::new(1, 254, 1);

    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let pins = bsp::Pins::new(peripherals.PORT);
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let gclk0 = clocks.gclk0();
    let timer_clock = clocks.tc2_tc3(&gclk0).unwrap();
    let mut timer = TimerCounter::tc3_(&timer_clock, peripherals.TC3, &mut peripherals.MCLK);
    timer.start(3.mhz());

    // The power pin, D10, must be pulled high in order to enable the NeoPixel data line.
    let mut power = pins.d10.into_push_pull_output();
    power.set_high().unwrap();

    // The Prop-Maker FeatherWing uses pin D5 on the Feather M4 Express for the NeoPixel data.
    let neopixel = pins.d5.into_push_pull_output();
    let mut ws = Ws2812::new(timer, neopixel);

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
