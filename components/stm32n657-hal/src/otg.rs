//! USB on-the-go high-speed (OTG), see Section 73 of RM0486.
//!
//! Additional information can be found in [AN4879: USB on STM32 products][an4879].
//!
//! [an4879]: https://www.st.com/resource/en/application_note/an4879-introduction-to-usb-hardware-and-pcb-guidelines-using-stm32-mcus-stmicroelectronics.pdf

use core::marker::PhantomData;

use cortex_m::interrupt;
use stm32n6::stm32n657::OTG1_S;
use synopsys_usb_otg::{UsbBus, UsbPeripheral};

use crate::{
    Rate,
    rcc::{ClockConfig, Peripheral, Rcc},
};

pub type UsbBus1 = UsbBus<Otg1>;

pub struct Otg1 {
    ahb_frequency: Rate,
    _marker: PhantomData<()>,
}

impl Otg1 {
    pub fn new(otg1: OTG1_S, clock_config: ClockConfig) -> Self {
        // TODO: check DM/DP

        let _ = otg1;

        Self {
            ahb_frequency: clock_config.sys_bus2_ck(),
            _marker: Default::default(),
        }
    }
}

unsafe impl UsbPeripheral for Otg1 {
    const REGISTERS: *const () = OTG1_S::ptr() as _;

    const HIGH_SPEED: bool = true;
    const FIFO_DEPTH_WORDS: usize = 1024;
    const ENDPOINT_COUNT: usize = 9;

    fn enable() {
        interrupt::free(|_| {
            // SAFETY: We only use the stolen RCC to configure this peripheral, so there are no race
            // conditions as this struct requires ownership of the peripheral.
            unsafe {
                let rcc = Rcc::steal();
                rcc.enable(Peripheral::Otg1);
                rcc.reset(Peripheral::Otg1);
            }
        });
    }

    fn ahb_frequency_hz(&self) -> u32 {
        self.ahb_frequency.to_Hz()
    }
}
