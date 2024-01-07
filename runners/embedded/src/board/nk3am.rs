use apps::{Apps, Dispatch, InitStatus};
use littlefs2::{
    fs::{Allocation, Filesystem},
    io::Result as LfsResult,
};
use memory_regions::MemoryRegions;
use nfc_device::traits::nfc::{Device as NfcDevice, Error as NfcError, State as NfcState};
use nrf52840_hal::{
    gpio::{p0, p1, Level, Output, Pin, PushPull},
    gpiote::Gpiote,
    spim, twim, Spim,
};
use nrf52840_pac::SPIM3;

use crate::{
    board::Board,
    flash::ExtFlashStorage,
    soc::nrf52840::{flash::FlashStorage, Nrf52},
    store::{impl_storage_pointers, RunnerStore},
    types::{Runner, Trussed},
};

use migrations::ftl_journal::{self, ifs_flash_old::FlashStorage as OldFlashStorage};
use ui::{HardwareButtons, RgbLed};

pub mod ui;

mod migrations;

type OutPin = Pin<Output<PushPull>>;

const MEMORY_REGIONS: &'static MemoryRegions = &MemoryRegions::NRF52;

pub struct NK3AM;

impl Board for NK3AM {
    type Soc = Nrf52;

    type Apps = Apps<Runner<Self>>;
    type Dispatch = Dispatch<Self::Twi, Self::Se050Timer>;

    type NfcDevice = DummyNfc;
    type Buttons = HardwareButtons;
    type Led = RgbLed;

    #[cfg(feature = "se050")]
    type Twi = nrf52840_hal::twim::Twim<nrf52840_pac::TWIM1>;
    #[cfg(feature = "se050")]
    type Se050Timer = nrf52840_hal::Timer<nrf52840_pac::TIMER1>;
    #[cfg(not(feature = "se050"))]
    type Twi = ();
    #[cfg(not(feature = "se050"))]
    type Se050Timer = ();

    const BOARD_NAME: &'static str = "NK3AM";

    fn init_apps(
        trussed: &mut Trussed<Self>,
        init_status: InitStatus,
        store: &RunnerStore<Self>,
        nfc_powered: bool,
    ) -> Self::Apps
    where
        Self: Sized,
    {
        let (runner, data) = super::init_nk3_apps(init_status, store, nfc_powered);
        Apps::with_service(&runner, trussed, data)
    }

    fn init_dispatch(
        hw_key: Option<&[u8]>,
        #[cfg(feature = "se050")] se050: Option<se05x::se05x::Se05X<Self::Twi, Self::Se050Timer>>,
    ) -> Self::Dispatch {
        super::init_nk3_dispatch::<Self>(
            hw_key,
            #[cfg(feature = "se050")]
            se050,
        )
    }

    fn prepare_ifs(ifs: &mut Self::InternalStorage) {
        ifs.format_journal_blocks();
    }

    fn recover_ifs(
        ifs_storage: &mut Self::InternalStorage,
        ifs_alloc: &mut Allocation<Self::InternalStorage>,
        efs_storage: &mut Self::ExternalStorage,
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

pub type InternalFlashStorage =
    FlashStorage<{ MEMORY_REGIONS.filesystem.start }, { MEMORY_REGIONS.filesystem.end }>;
pub type ExternalFlashStorage = ExtFlashStorage<Spim<SPIM3>, OutPin>;

impl_storage_pointers!(
    NK3AM,
    Internal = InternalFlashStorage,
    External = ExternalFlashStorage,
);

pub struct DummyNfc;

impl NfcDevice for DummyNfc {
    fn read(&mut self, _buf: &mut [u8]) -> Result<NfcState, NfcError> {
        Err(NfcError::NoActivity)
    }
    fn send(&mut self, _buf: &[u8]) -> Result<(), NfcError> {
        Err(NfcError::NoActivity)
    }
    fn frame_size(&self) -> usize {
        0
    }
}

pub struct BoardGPIO {
    /* interactive elements */
    pub rgb_led: [Option<Pin<Output<PushPull>>>; 3],
    pub touch: Option<Pin<Output<PushPull>>>,

    /* Secure Element (through TWIM1) */
    pub se_pins: Option<twim::Pins>,
    pub se_power: Option<Pin<Output<PushPull>>>,

    /* External Flash & NFC (through SxPIM3) */
    pub flashnfc_spi: Option<spim::Pins>,
    pub flash_cs: Option<Pin<Output<PushPull>>>,
}

pub fn init_pins(_gpiote: &Gpiote, gpio_p0: p0::Parts, gpio_p1: p1::Parts) -> BoardGPIO {
    /* touch sensor */
    let touch = gpio_p0.p0_04.into_push_pull_output(Level::High).degrade();
    // not used, just ensure output + low
    gpio_p0.p0_06.into_push_pull_output(Level::Low).degrade();

    /* RGB LED */
    let led_r = gpio_p0.p0_09.into_push_pull_output(Level::Low).degrade();
    let led_g = gpio_p0.p0_10.into_push_pull_output(Level::Low).degrade();
    let led_b = gpio_p1.p1_02.into_push_pull_output(Level::Low).degrade();

    /* SE050 */
    let se_pwr = gpio_p1.p1_10.into_push_pull_output(Level::Low).degrade();
    let se_scl = gpio_p1.p1_15.into_floating_input().degrade();
    let se_sda = gpio_p0.p0_02.into_floating_input().degrade();

    let se_pins = nrf52840_hal::twim::Pins {
        scl: se_scl,
        sda: se_sda,
    };

    /* Ext. Flash SPI */
    // Flash WP# gpio_p0.p0_22
    // Flash HOLD# gpio_p0.p0_23
    let flash_spi_cs = gpio_p0.p0_24.into_push_pull_output(Level::High).degrade();
    let flash_spi_clk = gpio_p1.p1_06.into_push_pull_output(Level::Low).degrade();
    let flash_spi_mosi = gpio_p1.p1_04.into_push_pull_output(Level::Low).degrade();
    let flash_spi_miso = gpio_p1.p1_00.into_floating_input().degrade();
    //let _flash_wp = gpio_p0.p0_22.into_push_pull_output(Level::Low).degrade();
    //let _flash_hold = gpio_p0.p0_23.into_push_pull_output(Level::High).degrade();

    let flash_spi = spim::Pins {
        sck: flash_spi_clk,
        miso: Some(flash_spi_miso),
        mosi: Some(flash_spi_mosi),
    };

    BoardGPIO {
        rgb_led: [Some(led_r), Some(led_g), Some(led_b)],
        touch: Some(touch),
        se_pins: Some(se_pins),
        se_power: Some(se_pwr),
        flashnfc_spi: Some(flash_spi),
        flash_cs: Some(flash_spi_cs),
    }
}
