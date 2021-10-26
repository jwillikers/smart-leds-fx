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
    SmartLedsWrite, RGB, RGB8,
};
use smart_leds_fx::colors::HsColor;
use smart_leds_fx::colors::RESTFUL_ORANGE;
use smart_leds_fx::iterators::BrightnessRange;
use ws2812_spi::Ws2812;

#[entry]
fn main() -> ! {
    rtt_init_default!();

    const DELAY: Milliseconds<u32> = Milliseconds::<u32>(8);
    const LED_COLOR: HsColor<u8> = RESTFUL_ORANGE;
    const NUM_LEDS: usize = 30;
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

    let mut ws = Ws2812::new(spi);

    loop {
        for j in brightness_range {
            let rgb: RGB8 = hsv2rgb(Hsv {
                hue: LED_COLOR.hue,
                sat: LED_COLOR.saturation,
                val: j,
            });
            let data = [rgb; NUM_LEDS];
            // Account for the fact the Triple Ring uses GRB ordering... even though it shouldn't.
            ws.write(data.iter().cloned()).unwrap();
            delay.delay_ms(DELAY.integer());
        }
    }
}
