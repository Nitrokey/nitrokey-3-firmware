use crate::soc::types::pac::SCB;
use apps::Variant;
use memory_regions::MemoryRegions;
use nrf52840_hal::{
    gpio::{Input, Output, Pin, PullDown, PullUp, PushPull},
    pac, spim, twim, uarte,
    usbd::{UsbPeripheral, Usbd},
};
use nrf52840_pac::{self, Interrupt};

use super::rtic_monotonic::{RtcDuration, RtcMonotonic};
use crate::types::Uuid;

pub type OutPin = Pin<Output<PushPull>>;

//////////////////////////////////////////////////////////////////////////////
// Upper Interface (definitions towards ERL Core)

pub static mut DEVICE_UUID: Uuid = [0u8; 16];

pub const MEMORY_REGIONS: &'static MemoryRegions = &MemoryRegions::NRF52;

pub struct Soc {}
impl crate::types::Soc for Soc {
    type UsbBus = Usbd<UsbPeripheral<'static>>;
    type Clock = RtcMonotonic;

    type Duration = RtcDuration;

    type Interrupt = Interrupt;
    const SYSCALL_IRQ: Interrupt = Interrupt::SWI0_EGU0;

    const SOC_NAME: &'static str = "NRF52840";
    const VARIANT: Variant = Variant::Nrf52;

    fn device_uuid() -> &'static Uuid {
        unsafe { &DEVICE_UUID }
    }
}

impl apps::Reboot for Soc {
    fn reboot() -> ! {
        SCB::sys_reset()
    }
    fn reboot_to_firmware_update() -> ! {
        let pac = unsafe { nrf52840_pac::Peripherals::steal() };
        pac.POWER.gpregret.write(|w| unsafe { w.bits(0xb1_u32) });

        SCB::sys_reset()
    }
    fn reboot_to_firmware_update_destructive() -> ! {
        // @TODO: come up with an idea how to
        // factory reset, and apply!
        SCB::sys_reset()
    }
    fn locked() -> bool {
        let pac = unsafe { nrf52840_pac::Peripherals::steal() };
        pac.UICR.approtect.read().pall().is_enabled()
    }
}

//////////////////////////////////////////////////////////////////////////////
// Lower Interface (common definitions for individual boards)

pub struct BoardGPIO {
    /* interactive elements */
    pub buttons: [Option<Pin<Input<PullUp>>>; 8],
    pub leds: [Option<Pin<Output<PushPull>>>; 4],
    pub rgb_led: [Option<Pin<Output<PushPull>>>; 3],
    pub touch: Option<Pin<Output<PushPull>>>,

    /* UARTE0 */
    pub uart_pins: Option<uarte::Pins>,

    /* Fingerprint Reader (through UARTE0) */
    pub fpr_detect: Option<Pin<Input<PullDown>>>,
    pub fpr_power: Option<Pin<Output<PushPull>>>,

    /* LCD (through SPIM0) */
    pub display_spi: Option<spim::Pins>,
    pub display_cs: Option<Pin<Output<PushPull>>>,
    pub display_reset: Option<Pin<Output<PushPull>>>,
    pub display_dc: Option<Pin<Output<PushPull>>>,
    pub display_backlight: Option<Pin<Output<PushPull>>>,
    pub display_power: Option<Pin<Output<PushPull>>>,

    /* Secure Element (through TWIM1) */
    pub se_pins: Option<twim::Pins>,
    pub se_power: Option<Pin<Output<PushPull>>>,

    /* External Flash & NFC (through SxPIM3) */
    pub flashnfc_spi: Option<spim::Pins>,
    pub flash_cs: Option<Pin<Output<PushPull>>>,
    pub flash_power: Option<Pin<Output<PushPull>>>,
    pub nfc_cs: Option<Pin<Output<PushPull>>>,
    pub nfc_irq: Option<Pin<Input<PullUp>>>,
}
