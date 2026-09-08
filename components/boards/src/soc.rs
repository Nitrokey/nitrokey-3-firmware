use cortex_m::interrupt::InterruptNumber;
use embedded_time::duration::Milliseconds;
use usb_device::bus::UsbBus;

use apps::{Reboot, Variant};

use crate::ui::Clock;

#[cfg(feature = "soc-lpc55")]
pub mod lpc55;
#[cfg(any(feature = "soc-lpc55", feature = "soc-stm32n6"))]
pub mod monotonic;
#[cfg(feature = "soc-nrf52")]
pub mod nrf52;
#[cfg(feature = "soc-stm32n6")]
pub mod stm32n6;

pub type Uuid = [u8; 16];

pub trait Soc: Reboot + 'static {
    type UsbBus: UsbBus + 'static;
    type Clock: Clock;

    type Duration: From<Milliseconds>;

    type Interrupt: InterruptNumber;
    const SYSCALL_IRQ: Self::Interrupt;

    const SOC_NAME: &'static str;
    const VARIANT: Variant;

    fn uuid(&self) -> &Uuid;
}
