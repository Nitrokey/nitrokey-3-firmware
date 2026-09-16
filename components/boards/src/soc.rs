use cortex_m::interrupt::InterruptNumber;
use embedded_time::duration::Milliseconds;
use usb_device::bus::UsbBus;

use apps::Variant;

use crate::ui::Clock;

#[cfg(feature = "soc-lpc55")]
pub mod lpc55;
#[cfg(feature = "soc-nrf52")]
pub mod nrf52;

pub type Uuid = [u8; 16];

pub trait Soc: 'static {
    type UsbBus: UsbBus + 'static;
    type Clock: Clock;

    type Duration: From<Milliseconds>;

    type Interrupt: InterruptNumber;
    const SYSCALL_IRQ: Self::Interrupt;

    const SOC_NAME: &'static str;
    const VARIANT: Variant;

    fn uuid(&self) -> &Uuid;

    fn reboot() -> !;

    fn reboot_to_firmware_update();

    fn locked() -> bool;
}
