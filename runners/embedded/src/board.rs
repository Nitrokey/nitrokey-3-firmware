#[cfg(feature = "se050")]
use embedded_hal::blocking::delay::DelayUs;
use littlefs2::{
    fs::{Allocation, Filesystem},
    io::Result as LfsResult,
};
use nfc_device::traits::nfc::Device as NfcDevice;

use crate::{
    soc::Soc,
    store::StoragePointers,
    ui::{buttons::UserPresence, rgb_led::RgbLed},
};

#[cfg(feature = "board-nk3am")]
pub mod nk3am;
#[cfg(feature = "board-nk3xn")]
pub mod nk3xn;

pub trait Board: StoragePointers {
    type Soc: Soc;

    type NfcDevice: NfcDevice;
    type Buttons: UserPresence;
    type Led: RgbLed;

    #[cfg(feature = "se050")]
    type Se050Timer: DelayUs<u32> + 'static;
    #[cfg(feature = "se050")]
    type Twi: se05x::t1::I2CForT1 + 'static;
    #[cfg(not(feature = "se050"))]
    type Se050Timer: 'static;
    #[cfg(not(feature = "se050"))]
    type Twi: 'static;

    const BOARD_NAME: &'static str;

    fn prepare_ifs(ifs: &mut Self::InternalStorage) {
        let _ = ifs;
    }

    fn recover_ifs(
        ifs_storage: &mut Self::InternalStorage,
        ifs_alloc: &mut Allocation<Self::InternalStorage>,
        efs_storage: &mut Self::ExternalStorage,
    ) -> LfsResult<()> {
        let _ = (ifs_alloc, efs_storage);
        Filesystem::format(ifs_storage)
    }
}
