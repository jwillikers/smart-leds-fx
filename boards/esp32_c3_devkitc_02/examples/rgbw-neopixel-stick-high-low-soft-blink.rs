#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;

use esp32c3_hal::{gpio::IO, pac::Peripherals, prelude::*, Delay, RtcCntl, Timer};

use embedded_time::duration::Milliseconds;
use embedded_time::fixed_point::FixedPoint;
use smart_leds::{
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, White, RGB, RGBW,
};
use smart_leds_fx::colors::RED;
use smart_leds_fx::colors::{HsColor, CHRISTMAS_GREEN};
use smart_leds_fx::iterators::BrightnessRange;
use ws2812_spi::Ws2812;

#[entry]
fn main() -> ! {
    const DELAY: Milliseconds<u32> = Milliseconds::<u32>(4);
    const FIRST_LED_COLOR: HsColor<u8> = CHRISTMAS_GREEN;
    const SECOND_LED_COLOR: HsColor<u8> = RED;
    const NUM_LEDS: usize = 8;

    let brightness_range = BrightnessRange::new(1, 254, 1);

    let mut peripherals = Peripherals::take().unwrap();

    // Disable the watchdog timers. For the ESP32-C3, this includes the Super WDT,
    // the RTC WDT, and the TIMG WDTs.
    let mut rtc_cntl = RtcCntl::new(peripherals.RTC_CNTL);
    let mut timer0 = Timer::new(peripherals.TIMG0);
    let mut timer1 = Timer::new(peripherals.TIMG1);

    rtc_cntl.set_super_wdt_enable(false);
    rtc_cntl.set_wdt_enable(false);
    timer0.disable();
    timer1.disable();

    peripherals
        .SYSTEM
        .sysclk_conf
        .modify(|_, w| unsafe { w.soc_clk_sel().bits(1) });

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let sclk = io.pins.gpio6;
    let miso = io.pins.gpio2;
    let mosi = io.pins.gpio7;
    let cs = io.pins.gpio10;

    let spi = esp32c3_hal::spi::Spi::new(
        peripherals.SPI2,
        sclk,
        mosi,
        miso,
        cs,
        3_333_333,
        embedded_hal::spi::MODE_0,
        &mut peripherals.SYSTEM,
    );

    let mut delay = Delay::new(peripherals.SYSTIMER);

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
