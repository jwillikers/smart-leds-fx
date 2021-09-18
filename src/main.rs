#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

use microbit::{
    hal::{
        gpio::{p0::Parts, Level},
        prelude::*,
        spi, Timer,
    },
    Peripherals,
};

use core::fmt::Debug;
use core::ops::{AddAssign, Not, SubAssign};
use embedded_time::duration::*;
use num_traits::PrimInt;
use smart_leds::{
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, RGB,
};
use ws2812_spi::Ws2812;

/// The direction of the changing brightness.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    Increasing,
    Decreasing,
}

impl Not for Direction {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Direction::Increasing => Direction::Decreasing,
            Direction::Decreasing => Direction::Increasing,
        }
    }
}

/// A range which forever oscillates between a minimum and maximum using the given step.
#[derive(Clone, Copy, Debug, PartialEq)]
struct BrightnessRange<T: PrimInt + Debug + SubAssign + AddAssign> {
    min: T,
    max: T,
    step: T,
    direction: Direction,
    value: T,
}

impl<T: PrimInt + Debug + SubAssign + AddAssign> BrightnessRange<T> {
    fn new(min: T, max: T, step: T) -> BrightnessRange<T> {
        debug_assert_ne!(step, T::zero());
        BrightnessRange {
            min,
            max,
            step,
            direction: {
                if (min < max && step > T::zero()) || (max < min && step < T::zero()) {
                    Direction::Increasing
                } else {
                    Direction::Decreasing
                }
            },
            value: min,
        }
    }
}

impl<T: PrimInt + Debug + SubAssign + AddAssign> Iterator for BrightnessRange<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.direction == Direction::Increasing {
            self.value -= self.step;
            if self.value <= self.min {
                self.direction = !self.direction
            }
        } else {
            self.value += self.step;
            if self.value >= self.max {
                self.direction = !self.direction
            }
        }

        return Some(self.value);
    }
}

/// Representation of a color with only the hue and saturation.
#[derive(Clone, Copy, Debug, PartialEq)]
struct HsColor<T> {
    hue: T,
    saturation: T,
}

impl<T> HsColor<T> {
    #[allow(dead_code)]
    fn new(hue: T, saturation: T) -> HsColor<T> {
        HsColor { hue, saturation }
    }
}

/// A gentle, flame-like orange color.
const RESTFUL_ORANGE: HsColor<u8> = HsColor {
    hue: 5,
    saturation: 255,
};

#[entry]
fn main() -> ! {
    const DELAY: Milliseconds<u32> = Milliseconds::<u32>(8);
    const LED_COLOR: HsColor<u8> = RESTFUL_ORANGE;
    const NUM_LEDS: usize = 30;

    let brightness_range = BrightnessRange::new(1, 254, 1);

    let dp = Peripherals::take().unwrap();
    let mut delay = Timer::new(dp.TIMER0);
    let port0 = Parts::new(dp.P0);
    let sck = port0.p0_17.into_push_pull_output(Level::Low).degrade();
    let mosi = port0.p0_13.into_push_pull_output(Level::Low).degrade();
    let miso = port0.p0_01.into_floating_input().degrade();
    let pins = spi::Pins {
        sck,
        miso: Some(miso),
        mosi: Some(mosi),
    };
    let spi = spi::Spi::new(dp.SPI0, pins, spi::Frequency::M2, spi::MODE_0);

    let mut ws = Ws2812::new(spi);

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
