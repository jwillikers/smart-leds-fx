//! Board support for the Adafruit QT Py RP2040.
//!
//! This module provides the same pin names, boot loader, and crystal frequency as the
//! [adafruit-qt-py-rp2040](https://crates.io/crates/adafruit-qt-py-rp2040) board support package.
//! It's adapted from version 0.8.0 of that crate, which is licensed under MIT OR Apache-2.0.
//! The board support package depends on rp2040-hal 0.10.
//! That prevents using ws2812-pio 0.9, which requires rp2040-hal 0.11 and supports smart-leds 0.4.
//! Defining the pins here allows using the newer rp2040-hal directly.
//!
//! The pin names are identical to those of the board support package.
//! The following table lists each pin name, the RP2040 GPIO it refers to, and the type aliases available for it.
//!
//! | Name             | GPIO   | Aliases                           |
//! |------------------|--------|-----------------------------------|
//! | `mosi`           | GPIO3  | `Mosi` (SPI0 TX, no pull)         |
//! | `miso`           | GPIO4  | `Miso` (SPI0 RX, no pull)         |
//! | `rx`             | GPIO5  | `UartRx` (UART1 RX, no pull)      |
//! | `sclk`           | GPIO6  | `Sclk` (SPI0 SCK, no pull)        |
//! | `neopixel_power` | GPIO11 |                                   |
//! | `neopixel_data`  | GPIO12 |                                   |
//! | `tx`             | GPIO20 | `UartTx` (UART1 TX, no pull)      |
//! | `button`         | GPIO21 |                                   |
//! | `sda1`           | GPIO22 | `Sda1` (I2C1 SDA, pull-up)        |
//! | `scl1`           | GPIO23 | `Scl1` (I2C1 SCL, pull-up)        |
//! | `sda`            | GPIO24 | `Sda` (I2C0 SDA, pull-up)         |
//! | `scl`            | GPIO25 | `Scl` (I2C0 SCL, pull-up)         |
//! | `a3`             | GPIO26 |                                   |
//! | `a2`             | GPIO27 |                                   |
//! | `a1`             | GPIO28 |                                   |
//! | `a0`             | GPIO29 |                                   |
#![no_std]

pub extern crate rp2040_hal as hal;

pub use hal::entry;
pub use hal::pac;

/// The linker will place this boot block at the start of our program image.
/// The ROM bootloader needs this to get our code up and running.
/// The QT Py RP2040 uses a GD25Q64CS flash chip.
#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_GD25Q64CS;

hal::bsp_pins!(
    Gpio3 {
        name: mosi,
        aliases: { FunctionSpi, PullNone: Mosi }
    },
    Gpio4 {
        name: miso,
        aliases: { FunctionSpi, PullNone: Miso }
    },
    Gpio5 {
        name: rx,
        aliases: { FunctionUart, PullNone: UartRx }
    },
    Gpio6 {
        name: sclk,
        aliases: { FunctionSpi, PullNone: Sclk }
    },
    Gpio11 { name: neopixel_power },
    Gpio12 { name: neopixel_data },
    Gpio20 {
        name: tx,
        aliases: { FunctionUart, PullNone: UartTx }
    },
    Gpio21 { name: button },
    Gpio22 {
        name: sda1,
        aliases: { FunctionI2C, PullUp: Sda1 }
    },
    Gpio23 {
        name: scl1,
        aliases: { FunctionI2C, PullUp: Scl1 }
    },
    Gpio24 {
        name: sda,
        aliases: { FunctionI2C, PullUp: Sda }
    },
    Gpio25 {
        name: scl,
        aliases: { FunctionI2C, PullUp: Scl }
    },
    Gpio26 { name: a3 },
    Gpio27 { name: a2 },
    Gpio28 { name: a1 },
    Gpio29 { name: a0 },
);

/// The frequency of the external crystal oscillator on the QT Py RP2040.
pub const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;
