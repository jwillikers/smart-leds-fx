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
    SmartLedsWrite, White, RGBW,
};
use ws2812_spi::Ws2812;

#[entry]
fn main() -> ! {
    const DELAY: u8 = 4;
    const NUM_LEDS: usize = 240;

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
        let first_rgbw: RGBW<u8> = RGBW {
            r: 0,
            g: 0,
            b: 0,
            a: White(175),
        };
        let data = [first_rgbw; NUM_LEDS];
        ws.write(data.iter().cloned()).unwrap();
        delay.delay_ms(DELAY);
    }
}
