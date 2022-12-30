#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt::debug_assert_ne;
use defmt_rtt as _;
use embedded_hal::spi::MODE_0;
use embedded_hal::adc::OneShot;
use fugit::RateExtU32;
use panic_probe as _;
use rp_pico as bsp;

use bsp::hal::{
    adc::Adc,
    clocks,
    prelude::*,
    gpio::FunctionSpi,
    pac,
    sio::Sio,
    spi::Spi,
    watchdog,
};
use movavg::MovAvg;
use smart_leds::{
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, White, RGB, RGBW,
};
use smart_leds_fx::colors::{ HsColor, RED };
use ws2812_spi::Ws2812;

const SYS_HZ: u32 = 125_000_000_u32;

const DARKNESS_THRESHOLD: u16 = 1400u16;

#[entry]
fn main() -> ! {
    info!("Program start");

    const DELAY: u32 = 8u32;
    const SECOND_LED_COLOR: HsColor<u8> = RED;
    const NUM_LEDS: usize = 450;
    debug_assert_ne!(NUM_LEDS, 0);

    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = watchdog::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // Our default is 12 MHz crystal input, 125 MHz system clock
    let clocks = clocks::init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // The single-cycle I/O block controls our GPIO pins
    let sio = Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Setup a delay for the LED blink signals:
    let mut delay =
        cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // These are implicitly used by the spi driver if they are in the correct mode
    let _spi_sclk = pins.gpio6.into_mode::<FunctionSpi>();
    let _spi_mosi = pins.gpio7.into_mode::<FunctionSpi>();
    let _spi_miso = pins.gpio4.into_mode::<FunctionSpi>();
    let spi = Spi::<_, _, 8>::new(pac.SPI0).init(
        &mut pac.RESETS,
        SYS_HZ.Hz(),
        3_000_000u32.Hz(),
        &MODE_0,
    );

    let mut ws = Ws2812::new_sk6812w(spi);

    let first_rgbw: RGBW<u8> = RGBW {
        r: 0,
        g: 0,
        b: 0,
        a: White(20),
    };
    let second_rgb: RGB<u8> = hsv2rgb(Hsv {
        hue: SECOND_LED_COLOR.hue,
        sat: SECOND_LED_COLOR.saturation,
        val: 200,
    });
    let second_rgbw: RGBW<u8> = RGBW {
        r: second_rgb.r,
        g: second_rgb.g,
        b: second_rgb.b,
        a: White(0),
    };

    let mut data = [first_rgbw; NUM_LEDS];
    for led in data.iter_mut().skip(1).step_by(2) {
        *led = second_rgbw.clone();
    }

    let blank_rgbw: RGBW<u8> = RGBW {
        r: 0,
        g: 0,
        b: 0,
        a: White(0),
    };
    let blank_data = [blank_rgbw; NUM_LEDS];

    let mut adc = Adc::new(pac.ADC, &mut pac.RESETS);
    let mut adc_pin_2 = pins.gpio28.into_floating_input();
    let mut ma: MovAvg<u16, u32, 200> = MovAvg::new();

    loop {
        let light_reading: u16 = adc.read(&mut adc_pin_2).unwrap();
        debug!("Light reading: {}", light_reading);
        ma.feed(light_reading);
        debug!("Moving Average: {}", ma.get());
        if ma.get() <= DARKNESS_THRESHOLD {
            ws.write(data.iter().cloned()).unwrap();
        } else {
            ws.write(blank_data.iter().cloned()).unwrap();
        }
        delay.delay_ms(DELAY);
    }
}
