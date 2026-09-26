//! System configuration (SYSCFG)

use stm32n6::stm32n657::SYSCFG_S;

use crate::rcc::{Peripheral, Rcc};

/// ES0620 2.2.20: CS = 1, RAPSRC = 0x8, RANSRC = 0x7, cell disabled
const IO_COMPENSATION_CODE: u32 = 0x0000_0287;

pub struct Syscfg(SYSCFG_S);

impl Syscfg {
    pub fn new(syscfg: SYSCFG_S, rcc: &Rcc) -> Self {
        rcc.enable(Peripheral::Syscfg);
        Self(syscfg)
    }

    /// ES0620 2.2.20 workaround: fixed I/O compensation codes -> defaults: broken!
    pub fn apply_io_compensation_workaround(&self) {
        unsafe {
            self.0.vddio1cccr().write(|w| w.bits(IO_COMPENSATION_CODE));
            self.0.vddio2cccr().write(|w| w.bits(IO_COMPENSATION_CODE));
            self.0.vddiocccr().write(|w| w.bits(IO_COMPENSATION_CODE));
        }
    }
}
