#![no_std]
#![no_main]

extern crate nb;

extern crate panic_halt;

use riscv_rt::entry;
use longan_nano::hal::{pac, prelude::*};
use longan_nano::led::{Led, rgb};
use longan_nano::hal::delay::McycleDelay;
use longan_nano::hal::spi::Pins;

use core::fmt::Debug;
use core::ops::{AddAssign, Not, SubAssign};
use num_traits::PrimInt;
use smart_leds::{hsv::{hsv2rgb, Hsv}, SmartLedsWrite, White, RGB, RGBW, brightness, RGB8};
// use ws2812_spi::prerendered::Ws2812;
use ws2812_spi::Ws2812;
use longan_nano::hal::spi::Spi;
use embedded_hal::spi::MODE_0;

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
        brightness_range: BrightnessRange::new(1, 254, 1),
        delay_ms: 8,
        neopixel_color: RESTFUL_ORANGE,
        spin_timer_cycles: 4,
    };
    let dp = pac::Peripherals::take().unwrap();
    let mut rcu = dp
        .RCU
        .configure()
        .ext_hf_clock(8.mhz())
        .sysclk(108.mhz())
        .freeze();
    let mut afio = dp.AFIO.constrain(&mut rcu);
    // let mut rcu = dp.RCU.configure().freeze();

    let gpioa = dp.GPIOA.split(&mut rcu);
    // let gpioc = dp.GPIOC.split(&mut rcu);

    let mut delay = McycleDelay::new(&rcu.clocks);

    let sck = gpioa.pa5.into_alternate_push_pull();
    let miso = gpioa.pa6.into_floating_input();
    let mosi = gpioa.pa7.into_alternate_push_pull();

    let spi = Spi::spi0(dp.SPI0, (sck, miso, mosi), &mut afio, MODE_0, 3_000_000.hz(), &mut rcu);

    // let mut delay = Delay::new(core.SYST, &mut clocks);
    // let mut pins = Pins::new(peripherals.PORT);
    //
    // let spi = spi_master(
    //     &mut clocks,
    //     3_000_000u32.hz(),
    //     peripherals.SERCOM1,
    //     &mut peripherals.MCLK,
    //     pins.sck,
    //     pins.mosi,
    //     pins.miso,
    //     &mut pins.port,
    // );

    const NUM_LEDS: usize = 8;
    // let mut buffer: [u8; 16 * NUM_LEDS + 40] = [0; 16 * NUM_LEDS + 40];
    // let mut ws = Ws2812::new_sk6812w(spi, &mut buffer);
    let mut ws = Ws2812::new_sk6812w(spi);

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
                a: White(j / 20),
            };
            let data = [rgbw; NUM_LEDS];
            ws.write(data.iter().cloned()).unwrap();
            delay.delay_ms(config.delay_ms);
        }
    }
}
