#![no_std]
#![no_main]

use circuit_playground_express::entry;
use panic_rtt_target as _;
use rtt_target::rtt_init_default;

use circuit_playground_express as hal;
use circuit_playground_express::hal::clock::GenericClockController;
use circuit_playground_express::hal::delay::Delay;
use circuit_playground_express::hal::hal::blocking::delay::DelayMs;
use circuit_playground_express::hal::hal::timer::CountDown;
use circuit_playground_express::hal::pac::{CorePeripherals, Peripherals};
use circuit_playground_express::hal::time::U32Ext;
use circuit_playground_express::hal::timer::TimerCounter;

use embedded_time::duration::*;
use smart_leds::{
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, RGB,
};
use ws2812_timer_delay as ws2812;

use smart_leds_fx::colors::HsColor;
use smart_leds_fx::colors::RESTFUL_ORANGE;
use smart_leds_fx::iterators::BrightnessRange;

#[entry]
fn main() -> ! {
    rtt_init_default!();

    const DELAY: Milliseconds<u32> = Milliseconds::<u32>(8);
    const LED_COLOR: HsColor<u8> = RESTFUL_ORANGE;
    const NUM_LEDS: usize = 10;
    debug_assert_ne!(NUM_LEDS, 0);

    let brightness_range = BrightnessRange::new(1, 254, 1);

    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );
    let pins = hal::Pins::new(peripherals.PORT);
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let gclk0 = clocks.gclk0();
    let timer_clock = clocks.tcc2_tc3(&gclk0).unwrap();
    let mut timer = TimerCounter::tc3_(&timer_clock, peripherals.TC3, &mut peripherals.PM);
    timer.start(3.mhz());

    let ws_data_pin: hal::NeoPixel = pins.d8.into();
    let mut ws = ws2812::Ws2812::new(timer, ws_data_pin);

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
