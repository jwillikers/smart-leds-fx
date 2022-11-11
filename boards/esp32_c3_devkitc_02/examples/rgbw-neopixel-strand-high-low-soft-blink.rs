#![no_std]
#![no_main]

use esp_backtrace as _;
use riscv_rt::entry;

use esp32c3_hal::{
    clock::ClockControl,
    gpio::IO,
    pac::Peripherals,
    prelude::*,
    spi::{Spi, SpiMode},
    timer::TimerGroup,
    Delay,
    Rtc,
};

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
    const DELAY: u8 = 4;
    const FIRST_LED_COLOR: HsColor<u8> = CHRISTMAS_GREEN;
    const SECOND_LED_COLOR: HsColor<u8> = RED;
    const NUM_LEDS: usize = 240;

    let brightness_range = BrightnessRange::new(1, 75, 1);

    let peripherals = Peripherals::take().unwrap();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the watchdog timers. For the ESP32-C3, this includes the Super WDT,
    // the RTC WDT, and the TIMG WDTs.
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let sclk = io.pins.gpio6;
    let miso = io.pins.gpio2;
    let mosi = io.pins.gpio7;
    let cs = io.pins.gpio10;

    let spi = Spi::new(
        peripherals.SPI2,
        sclk,
        mosi,
        miso,
        cs,
        3_333_333u32.Hz(),
        // 100u32.kHz(),
        SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    );

    let mut delay = Delay::new(&clocks);

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
            delay.delay_ms(DELAY);
        }
    }
}
