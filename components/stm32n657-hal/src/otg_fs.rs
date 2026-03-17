use core::marker::PhantomData;

use stm32n6::stm32n657::{OTG1_S, RCC};
use synopsys_usb_otg::{UsbBus, UsbPeripheral};

use crate::{Rate, rcc::{ClockConfig, Peripheral, Rcc}};

pub type UsbBus1Fs = UsbBus<Otg1Fs>;

pub struct Otg1Fs {
    ahb_frequency: Rate,
    _marker: PhantomData<()>,
}

impl Otg1Fs {
    pub fn new(otg1: OTG1_S, clock_config: ClockConfig) -> Self {
        // TODO: check DM/DP

        let _ = otg1;
        
        Self {
            ahb_frequency: clock_config.sys_bus2_ck(),
            _marker: Default::default(),
        }
    }
}

unsafe impl UsbPeripheral for Otg1Fs {
    const REGISTERS: *const () = OTG1_S::ptr() as _;

    const HIGH_SPEED: bool = false;
    const FIFO_DEPTH_WORDS: usize = 512;
    const ENDPOINT_COUNT: usize = 8;

    fn enable() {
        // SAFETY: We only use the stolen RCC to enable a peripheral, there are no race
        // conditions.
        unsafe {
            // TODO: Do we need a reset?
            Rcc::new(RCC::steal()).enable(Peripheral::Otg1);
        }
    }

    fn ahb_frequency_hz(&self) -> u32 {
        self.ahb_frequency.to_Hz()
    }
}
