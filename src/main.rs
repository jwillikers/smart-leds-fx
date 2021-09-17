#![no_std]
#![no_main]

extern crate cortex_m;
extern crate cortex_m_semihosting;
extern crate nb;

extern crate panic_halt;

extern crate feather_m4 as bsp;

use bsp::hal;
use bsp::Pins;
use bsp::{entry, spi_master};
use core::fmt::Debug;
use core::ops::{AddAssign, Not, SubAssign};
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::CorePeripherals;
use hal::pac::Peripherals;
use hal::prelude::*;
use num_traits::PrimInt;
use smart_leds::{hsv::{hsv2rgb, Hsv}, SmartLedsWrite, White, RGB, RGBW, brightness, RGB8};
use ws2812_spi::prerendered::Ws2812;

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

/// Configuration of the Neopixel and the soft blink effect.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Config<T: PrimInt + Debug + SubAssign + AddAssign> {
    /// The brightness range of the soft blink effect.
    brightness_range: BrightnessRange<T>,
    /// Delay between each individual brightness adjustment in milliseconds.
    delay_ms: T,
    /// The color of the Neopixel.
    neopixel_color: HsColor<T>,
    /// Be careful setting the cycles value too high when fading brightness.
    /// You might end up with a bright white Neopixel.
    spin_timer_cycles: u32,
}

/// A gentle, flame-like orange color.
static RESTFUL_ORANGE: HsColor<u8> = HsColor {
    hue: 5,
    saturation: 255,
};

#[entry]
fn main() -> ! {
    let config = Config {
        brightness_range: BrightnessRange::new(75, 200, 1),
        delay_ms: 8,
        neopixel_color: RESTFUL_ORANGE,
        spin_timer_cycles: 4,
    };

    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );

    let mut delay = Delay::new(core.SYST, &mut clocks);
    let mut pins = Pins::new(peripherals.PORT);

    let spi = spi_master(
        &mut clocks,
        3_000_000u32.hz(),
        peripherals.SERCOM1,
        &mut peripherals.MCLK,
        pins.sck,
        pins.mosi,
        pins.miso,
        &mut pins.port,
    );

    const NUM_LEDS: usize = 8;
    let mut buffer: [u8; 16 * NUM_LEDS + 40] = [0; 16 * NUM_LEDS + 40];
    let mut ws = Ws2812::new_sk6812w(spi, &mut buffer);

    loop {
        for j in config.brightness_range {
            let rgb: RGB<u8> = hsv2rgb(Hsv {
                hue: config.neopixel_color.hue,
                sat: config.neopixel_color.saturation,
                val: j,
            });
            let rgbw: RGBW<u8> = RGBW {
                r: rgb.r,
                g: rgb.g,
                b: rgb.b,
                a: White(j),
            };
            let data = [rgbw; NUM_LEDS];
            ws.write(data.iter().cloned()).unwrap();
            delay.delay_ms(config.delay_ms);
        }
    }
}
