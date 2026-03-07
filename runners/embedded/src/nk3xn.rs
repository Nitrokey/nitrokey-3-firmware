pub mod init;

use boards::{init::Resources, nk3xn::NK3xN};

use crate::{VERSION, VERSION_STRING};

pub fn init(
    device_peripherals: lpc55_hal::raw::Peripherals,
    core_peripherals: rtic::export::Peripherals,
    resources: &'static mut Resources<NK3xN>,
) -> init::All {
    const SECURE_FIRMWARE_VERSION: u32 = VERSION.encode();

    boards::init::init_logger::<NK3xN>(VERSION_STRING);

    let hal = lpc55_hal::Peripherals::from((device_peripherals, core_peripherals));

    let require_prince = cfg!(not(feature = "no-encrypted-storage"));
    let secure_firmware_version = Some(SECURE_FIRMWARE_VERSION);
    let boot_to_bootrom = true;

    init::start(hal.syscon, hal.pmc, hal.anactrl)
        .next(hal.iocon, hal.gpio, hal.wwdt)
        .next(
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
        .next(hal.flexcomm.0, hal.flexcomm.5)
        .next(hal.rng, hal.prince, hal.flash)
        .next(&mut resources.store)
        .next(hal.rtc)
        .next(&mut resources.usb, hal.usbhs)
}
