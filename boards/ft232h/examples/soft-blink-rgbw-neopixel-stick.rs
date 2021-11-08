use ftdi_embedded_hal as hal;
use ftdi_embedded_hal::ftdi_mpsse::MpsseSettings;
use hal::embedded_hal::blocking::delay::DelayMs;
use hal::embedded_hal::spi::Polarity;
use hal::Delay;
use smart_leds::{
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite, White, RGB8, RGBW,
};
use smart_leds_fx::colors::HsColor;
use smart_leds_fx::colors::RESTFUL_ORANGE;
use smart_leds_fx::iterators::BrightnessRange;
use std::time::Duration;
use ws2812_spi::Ws2812;

fn main() -> ! {
    const DELAY: u32 = 4;
    const LED_COLOR: HsColor<u8> = RESTFUL_ORANGE;
    const NUM_LEDS: usize = 8;
    debug_assert_ne!(NUM_LEDS, 0);

    let brightness_range = BrightnessRange::new(15, 240, 1);

    let mut delay: Delay = Delay::new();

    // let num_devices = hal::num_devices().unwrap();
    // println!("Number of devices: {}", num_devices);

    // AD0 => SCK
    // AD1 => MOSI
    // AD2 => MISO
    // let ft = hal::FtHal::new().unwrap();
    // let mut ft = hal::Ftdi::new().unwrap();

    // let dev_type = ft.device_type().unwrap();
    // println!("Device type: {:?}", dev_type);

    // let info = ft.device_info().unwrap();
    // println!("Device information: {:?}", info);

    // ft.close().unwrap();

    // let device = hal::FtHal::new().unwrap();

    let mut device: ftdi::Device = ftdi::find_by_vid_pid(0x0403, 0x6014)
        .interface(ftdi::Interface::A)
        .open()
        .unwrap();

    // device.libftdi_context().

    // device.usb_reset().unwrap();
    // device.usb_purge_buffers().unwrap();
    // device.set_latency_timer(2).unwrap();
    // let config: MpsseSettings = MpsseSettings {
    //     reset: true,
    //     // in_transfer_size: 128,
    //     // in_transfer_size: 16384,
    //     // in_transfer_size: 32768,
    //     // read_timeout: Duration::from_millis(100),
    //     // write_timeout: Duration::from_millis(100),
    //     // latency_timer: Duration::from_millis(8),
    //     // clock_frequency: Some(6_400_000u32),
    //     clock_frequency: Some(3_000_000u32),
    //     // clock_frequency: Some(100_000u32),
    //     // mask: 0x00,
    //     ..Default::default()
    // };
    // let ft = device.init(&config).unwrap();
    // let ftdi = ft.init_default().unwrap();
    // let ft = hal::FtHal::init(device, &config).unwrap();
    let ft = hal::FtHal::init_freq(device, 3_000_000).unwrap();
    let mut spi = ft.spi().unwrap();
    spi.set_clock_polarity(Polarity::IdleLow);

    let mut neopixel = Ws2812::new_sk6812w(spi);

    loop {
        for j in brightness_range {
            let rgb: RGB8 = hsv2rgb(Hsv {
                hue: LED_COLOR.hue,
                sat: LED_COLOR.saturation,
                val: j,
            });
            let rgbw: RGBW<u8> = RGBW {
                r: rgb.r,
                g: rgb.g,
                b: rgb.b,
                a: White(j / 20),
            };
            let data = [rgbw; NUM_LEDS];
            neopixel.write(data.iter().cloned()).unwrap();
            delay.delay_ms(DELAY);
        }
    }
}
