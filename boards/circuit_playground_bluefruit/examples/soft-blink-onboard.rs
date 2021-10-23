#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
// use panic_rtt_target as _;
// use rtt_target::rtt_init_default;

use nrf52840_hal::{
    pac::Peripherals,
    gpio::{p0::Parts, Level},
    prelude::*,
    Timer,
};
use embedded_hal::timer::{CountDown, Periodic};

use embedded_time::duration::*;
use smart_leds::{
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, RGB,
};
use ws2812_timer_delay as ws2812;

use smart_leds_fx::colors::HsColor;
use smart_leds_fx::colors::RESTFUL_ORANGE;
use smart_leds_fx::iterators::BrightnessRange;
use cortex_m::asm::delay as cycle_delay;

#[derive(Clone, Copy)]
pub struct SpinTimer {
    cycles: u32,
}

// From the atsamd-rs HAL project: https://github.com/atsamd-rs/atsamd
// todo Pull into the library, but only for Arm Cortex devices.
impl SpinTimer {
    pub fn new(cycles: u32) -> SpinTimer {
        SpinTimer { cycles }
    }
}

impl Periodic for SpinTimer {}

impl CountDown for SpinTimer {
    type Time = u32;

    fn start<T>(&mut self, cycles: T)
        where
            T: Into<Self::Time>,
    {
        self.cycles = cycles.into();
    }

    fn wait(&mut self) -> nb::Result<(), void::Void> {
        cycle_delay(self.cycles);
        Ok(())
    }
}

#[entry]
fn main() -> ! {
    // rtt_init_default!();

    const DELAY: Milliseconds<u32> = Milliseconds::<u32>(8);
    const LED_COLOR: HsColor<u8> = RESTFUL_ORANGE;
    const NUM_LEDS: usize = 10;
    debug_assert_ne!(NUM_LEDS, 0);

    let brightness_range = BrightnessRange::new(1, 254, 1);

    let dp = Peripherals::take().unwrap();
    let mut delay = Timer::new(dp.TIMER0);
    let port0 = Parts::new(dp.P0);

    let _power_switch = port0.p0_06.into_push_pull_output(Level::Low);
    let ws_data_pin = port0.p0_13.into_push_pull_output(Level::Low);
    let timer = SpinTimer::new(8);
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
