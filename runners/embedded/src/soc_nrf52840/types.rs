use core::mem::MaybeUninit;

use crate::soc::types::pac::SCB;
use apps::Variant;
use littlefs2::{
    fs::{Allocation, Filesystem},
    io::Result as LfsResult,
};
use memory_regions::MemoryRegions;
use nrf52840_hal::{
    gpio::{Input, Output, Pin, PullDown, PullUp, PushPull},
    pac, spim, twim, uarte,
    usbd::{UsbPeripheral, Usbd},
};
use nrf52840_pac::{self, Interrupt};
use trussed::store::Fs;

use super::migrations::ftl_journal::{self, ifs_flash_old::FlashStorage as OldFlashStorage};
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

    unsafe fn ifs_ptr() -> *mut Fs<Self::InternalFlashStorage> {
        static mut IFS: MaybeUninit<Fs<InternalFlashStorage>> = MaybeUninit::uninit();
        IFS.as_mut_ptr()
    }

    unsafe fn efs_ptr() -> *mut Fs<Self::ExternalFlashStorage> {
        static mut EFS: MaybeUninit<Fs<ExternalFlashStorage>> = MaybeUninit::uninit();
        EFS.as_mut_ptr()
    }

    unsafe fn ifs_storage() -> &'static mut Option<Self::InternalFlashStorage> {
        static mut IFS_STORAGE: Option<InternalFlashStorage> = None;
        &mut IFS_STORAGE
    }

    unsafe fn ifs_alloc() -> &'static mut Option<Allocation<Self::InternalFlashStorage>> {
        static mut IFS_ALLOC: Option<Allocation<InternalFlashStorage>> = None;
        &mut IFS_ALLOC
    }

    unsafe fn ifs() -> &'static mut Option<Filesystem<'static, Self::InternalFlashStorage>> {
        static mut IFS: Option<Filesystem<InternalFlashStorage>> = None;
        &mut IFS
    }

    unsafe fn efs_storage() -> &'static mut Option<Self::ExternalFlashStorage> {
        static mut EFS_STORAGE: Option<ExternalFlashStorage> = None;
        &mut EFS_STORAGE
    }

    unsafe fn efs_alloc() -> &'static mut Option<Allocation<Self::ExternalFlashStorage>> {
        static mut EFS_ALLOC: Option<Allocation<ExternalFlashStorage>> = None;
        &mut EFS_ALLOC
    }

    unsafe fn efs() -> &'static mut Option<Filesystem<'static, Self::ExternalFlashStorage>> {
        static mut EFS: Option<Filesystem<ExternalFlashStorage>> = None;
        &mut EFS
    }

    fn prepare_ifs(ifs: &mut Self::InternalFlashStorage) {
        ifs.format_journal_blocks();
    }

    fn recover_ifs(
        ifs_storage: &mut Self::InternalFlashStorage,
        ifs_alloc: &mut Allocation<Self::InternalFlashStorage>,
        efs_storage: &mut Self::ExternalFlashStorage,
    ) -> LfsResult<()> {
        error_now!("IFS (nrf42) mount-fail");

        // regular mount failed, try mounting "old" (pre-journaling) IFS
        let pac = unsafe { nrf52840_pac::Peripherals::steal() };
        let mut old_ifs_storage = OldFlashStorage::new(pac.NVMC);
        let mut old_ifs_alloc: Allocation<OldFlashStorage> = Filesystem::allocate();
        let old_mountable = Filesystem::is_mountable(&mut old_ifs_storage);

        // we can mount the old ifs filesystem, thus we need to migrate
        if old_mountable {
            let mounted_ifs = ftl_journal::migrate(
                &mut old_ifs_storage,
                &mut old_ifs_alloc,
                ifs_alloc,
                ifs_storage,
                efs_storage,
            );
            // migration went fine => use its resulting IFS
            if let Ok(()) = mounted_ifs {
                info_now!("migration ok, mounting IFS");
                Ok(())
            // migration failed => format IFS
            } else {
                error_now!("failed migration, formatting IFS");
                Filesystem::format(ifs_storage)
            }
        } else {
            info_now!("recovering from journal");
            // IFS and old-IFS cannot be mounted, try to recover from journal
            ifs_storage.recover_from_journal();
            Ok(())
        }
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
