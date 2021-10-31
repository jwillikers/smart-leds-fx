use ftdi_embedded_hal::Delay;
use ftdi_embedded_hal::Ft232hHal;
use std::time::Duration;

use ftdi_embedded_hal::embedded_hal::blocking::delay::DelayMs;
use ftdi_embedded_hal::embedded_hal::spi::Polarity;
use ftdi_embedded_hal::libftd2xx::{
    list_devices, num_devices, Ft232h, FtStatus, Ftdi, FtdiCommon, FtdiMpsse, MpsseSettings,
};
use smart_leds::{
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, RGB8,
};
use smart_leds_fx::colors::HsColor;
use smart_leds_fx::colors::RESTFUL_ORANGE;
use smart_leds_fx::iterators::BrightnessRange;
use ws2812_spi::Ws2812;
// use ws2812_spi::prerendered::Ws2812;

fn main() -> Result<(), FtStatus> {
    const DELAY: u32 = 8;
    const LED_COLOR: HsColor<u8> = RESTFUL_ORANGE;
    const NUM_LEDS: usize = 30;
    debug_assert_ne!(NUM_LEDS, 0);

    let brightness_range = BrightnessRange::new(1, 254, 1);

    let mut delay: Delay = Delay::new();

    // let num_devices = num_devices().unwrap();
    // println!("Number of devices: {}", num_devices);

    // AD0 => SCK
    // AD1 => MOSI
    // AD2 => MISO
    // let ft = Ft232hHal::new().unwrap();
    // let mut ft = Ftdi::new().unwrap();
    //
    // let dev_type = ft.device_type().unwrap();
    // println!("Device type: {:?}", dev_type);
    //
    // let info = ft.device_info().unwrap();
    // println!("Device information: {:?}", info);
    //
    // ft.close().unwrap();
    //
    // let mut ft = Ft232h::with_serial_number("FT000001").unwrap();
    // ft.initialize_mpsse_default().unwrap();
    // ft.set_clock(3_000_000).unwrap();

    let ft = Ft232hHal::new().unwrap();
    // ft.initialize_mpsse_default().unwrap();
    // ft.set_clock(100_000).unwrap();
    let config: MpsseSettings = MpsseSettings {
        //     reset: true,
        in_transfer_size: 128,
        // in_transfer_size: 16384,
        // in_transfer_size: 32768,
        // read_timeout: Duration::from_millis(100),
        // write_timeout: Duration::from_millis(100),
        // latency_timer: Duration::from_millis(8),
        // clock_frequency: Some(6_400_000u32),
        clock_frequency: Some(3_000_000u32),
        //     //     clock_frequency: Some(100_000u32),
        //     mask: 0x00,
        ..Default::default()
    };
    let ftdi = ft.init(&config).unwrap();
    // let ftdi = ft.init_default().unwrap();
    let mut spi = ftdi.spi().unwrap();
    spi.set_clock_polarity(Polarity::IdleLow);
    // spi.set_clock_polarity(Polarity::IdleHigh);
    // let mut output_buffer = [0; 20 + (NUM_LEDS * 12)];
    // let mut neopixel = Ws2812::new(spi, &mut output_buffer);
    let mut neopixel = Ws2812::new(spi);

    loop {
        for j in brightness_range {
            let rgb: RGB8 = hsv2rgb(Hsv {
                hue: LED_COLOR.hue,
                sat: LED_COLOR.saturation,
                val: j,
            });
            let data = [rgb; NUM_LEDS];
            // Account for the fact the Triple Ring uses GRB ordering... even though it shouldn't.
            neopixel.write(data.iter().cloned()).unwrap();
            delay.delay_ms(DELAY);
        }
    }
}
