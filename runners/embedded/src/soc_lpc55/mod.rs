use littlefs2::{
    fs::{Allocation, Filesystem},
    io::Result as LfsResult,
};

use types::{ExternalFlashStorage, InternalFlashStorage};

pub mod clock_controller;
pub mod init;
pub mod monotonic;
pub mod nfc;
pub mod prince;
pub mod spi;
pub mod types;

// modules with path attribute *are* directory owners if their path
// refers to a 'mod.rs'
#[cfg_attr(feature = "board-nk3xn", path = "board_nk3xn/mod.rs")]
pub mod board;

use delog::delog;

#[cfg(not(feature = "no-delog"))]
delog!(Delogger, 3 * 1024, 512, crate::types::DelogFlusher);

const SECURE_FIRMWARE_VERSION: u32 = utils::VERSION.encode();

pub fn init(
    device_peripherals: lpc55_hal::raw::Peripherals,
    core_peripherals: rtic::export::Peripherals,
) -> init::All {
    #[cfg(feature = "log-rtt")]
    rtt_target::rtt_init_print!();

    #[cfg(any(
        feature = "log-rtt",
        feature = "log-semihosting",
        feature = "log-serial"
    ))]
    Delogger::init_default(delog::LevelFilter::Debug, &crate::types::DELOG_FLUSHER).ok();

    crate::banner::<types::Soc>();

    let hal = lpc55_hal::Peripherals::from((device_peripherals, core_peripherals));

    let require_prince = cfg!(not(feature = "no-encrypted-storage"));
    let secure_firmware_version = Some(SECURE_FIRMWARE_VERSION);
    let nfc_enabled = true;
    let boot_to_bootrom = true;

    init::start(hal.syscon, hal.pmc, hal.anactrl)
        .next(hal.iocon, hal.gpio)
        .next(
            hal.adc,
            hal.ctimer.0,
            hal.ctimer.1,
            hal.ctimer.2,
            hal.ctimer.3,
            hal.ctimer.4,
            hal.pfr,
            secure_firmware_version,
            require_prince,
            boot_to_bootrom,
        )
        .next(
            hal.flexcomm.0,
            hal.flexcomm.5,
            hal.inputmux,
            hal.pint,
            nfc_enabled,
        )
        .next(hal.rng, hal.prince, hal.flash)
        .next()
        .next(hal.rtc)
        .next(hal.usbhs)
}

pub fn prepare_ifs(_ifs: &mut types::InternalFilesystem) {}

pub fn recover_ifs(
    ifs_storage: &mut InternalFlashStorage,
    _ifs_alloc: &mut Allocation<InternalFlashStorage>,
    _efs_storage: &mut ExternalFlashStorage,
) -> LfsResult<()> {
    Filesystem::format(ifs_storage)
}
