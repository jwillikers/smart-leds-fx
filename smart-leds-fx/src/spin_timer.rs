// From the atsamd-rs HAL project: https://github.com/atsamd-rs/atsamd

use embedded_hal::timer::{CountDown, Periodic};
use cortex_m::asm::delay as cycle_delay;

#[derive(Clone, Copy)]
pub struct SpinTimer {
    cycles: u32,
}

impl SpinTimer {
    pub fn new(cycles: u32) -> SpinTimer {
        SpinTimer { cycles }
    }
}

impl Periodic for SpinTimer {}

impl CountDown for SpinTimer {
    type Time = u32;

    fn start<T>(&mut self, cycles: T)
        where
            T: Into<Self::Time>,
    {
        self.cycles = cycles.into();
    }

    fn wait(&mut self) -> nb::Result<(), void::Void> {
        cycle_delay(self.cycles);
        Ok(())
    }
}
