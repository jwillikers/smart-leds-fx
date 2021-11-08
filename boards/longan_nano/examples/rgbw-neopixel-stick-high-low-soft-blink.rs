#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;
use rtt_target::rtt_init_default;

use longan_nano::hal::{delay::McycleDelay, pac, prelude::*, spi::Spi};

use embedded_hal::spi;
use embedded_time::duration::Milliseconds;
use embedded_time::fixed_point::FixedPoint;
use smart_leds::{
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, White, RGB, RGBW,
};
use smart_leds_fx::colors::{HsColor, CHRISTMAS_GREEN};
use smart_leds_fx::colors::{RED, WHITE};
use smart_leds_fx::iterators::BrightnessRange;
use ws2812_spi::Ws2812;

#[entry]
fn main() -> ! {
    rtt_init_default!();

    const DELAY: Milliseconds<u32> = Milliseconds::<u32>(4);
    const FIRST_LED_COLOR: HsColor<u8> = CHRISTMAS_GREEN;
    const SECOND_LED_COLOR: HsColor<u8> = RED;
    const NUM_LEDS: usize = 8;
    debug_assert_ne!(NUM_LEDS, 0);

    let brightness_range = BrightnessRange::new(1, 254, 1);

    let dp = pac::Peripherals::take().unwrap();
    let mut rcu = dp
        .RCU
        .configure()
        .ext_hf_clock(8.mhz())
        .sysclk(108.mhz())
        .freeze();
    let mut afio = dp.AFIO.constrain(&mut rcu);
    let gpioa = dp.GPIOA.split(&mut rcu);

    let mut delay = McycleDelay::new(&rcu.clocks);

    let sck = gpioa.pa5.into_alternate_push_pull();
    let miso = gpioa.pa6.into_floating_input();
    let mosi = gpioa.pa7.into_alternate_push_pull();
    let spi = Spi::spi0(
        dp.SPI0,
        (sck, miso, mosi),
        &mut afio,
        spi::MODE_0,
        3_000_000.hz(),
        &mut rcu,
    );

    let mut ws = Ws2812::new_sk6812w(spi);

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
            let first_rgbw: RGBW<u8> = RGBW {
                r: first_rgb.r,
                g: first_rgb.g,
                b: first_rgb.b,
                a: White(j / 20),
            };
            let second_rgbw: RGBW<u8> = RGBW {
                r: second_rgb.r,
                g: second_rgb.g,
                b: second_rgb.b,
                a: White((254 - j) / 20),
            };
            let mut data = [first_rgbw; NUM_LEDS];
            for led in data.iter_mut().skip(1).step_by(2) {
                *led = second_rgbw.clone();
            }
            ws.write(data.iter().cloned()).unwrap();
            delay.delay_ms(DELAY.integer());
        }
    }
}
