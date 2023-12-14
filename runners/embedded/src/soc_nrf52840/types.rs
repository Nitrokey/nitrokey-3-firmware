use crate::soc::types::pac::SCB;
use apps::Variant;
use memory_regions::MemoryRegions;
use nrf52840_hal::{
    gpio::{Input, Output, Pin, PullDown, PullUp, PushPull},
    pac, spim, twim, uarte,
    usbd::{UsbPeripheral, Usbd},
};
use nrf52840_pac::{self, Interrupt};

use crate::{flash::ExtFlashStorage, types::Uuid};
use nrf52840_hal::Spim;
use nrf52840_pac::SPIM3;

pub type OutPin = Pin<Output<PushPull>>;

//////////////////////////////////////////////////////////////////////////////
// Upper Interface (definitions towards ERL Core)

pub static mut DEVICE_UUID: Uuid = [0u8; 16];

pub const MEMORY_REGIONS: &'static MemoryRegions = &MemoryRegions::NRF52;

pub type InternalFlashStorage = super::flash::FlashStorage;
pub type ExternalFlashStorage = ExtFlashStorage<Spim<SPIM3>, OutPin>;

pub struct Soc {}
impl crate::types::Soc for Soc {
    type InternalFlashStorage = InternalFlashStorage;
    type ExternalFlashStorage = ExternalFlashStorage;
    type UsbBus = Usbd<UsbPeripheral<'static>>;
    type NfcDevice = DummyNfc;
    type TrussedUI = super::board::TrussedUI;
    #[cfg(feature = "se050")]
    type Twi = twim::Twim<pac::TWIM1>;
    #[cfg(feature = "se050")]
    type Se050Timer = nrf52840_hal::Timer<nrf52840_pac::TIMER1>;
    #[cfg(not(feature = "se050"))]
    type Twi = ();
    #[cfg(not(feature = "se050"))]
    type Se050Timer = ();

    type Duration = super::rtic_monotonic::RtcDuration;

    type Interrupt = Interrupt;
    const SYSCALL_IRQ: Interrupt = Interrupt::SWI0_EGU0;

    const SOC_NAME: &'static str = "NRF52840";
    const BOARD_NAME: &'static str = super::board::BOARD_NAME;
    const VARIANT: Variant = Variant::Nrf52;

    fn device_uuid() -> &'static Uuid {
        unsafe { &DEVICE_UUID }
    }
}

pub struct DummyNfc;
impl nfc_device::traits::nfc::Device for DummyNfc {
    fn read(
        &mut self,
        _buf: &mut [u8],
    ) -> Result<nfc_device::traits::nfc::State, nfc_device::traits::nfc::Error> {
        Err(nfc_device::traits::nfc::Error::NoActivity)
    }
    fn send(&mut self, _buf: &[u8]) -> Result<(), nfc_device::traits::nfc::Error> {
        Err(nfc_device::traits::nfc::Error::NoActivity)
    }
    fn frame_size(&self) -> usize {
        0
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
