#[cfg(feature = "se050")]
use embedded_hal::blocking::delay::DelayUs;
use littlefs2::{
    fs::{Allocation, Filesystem},
    io::Result as LfsResult,
};
use nfc_device::traits::nfc::Device as NfcDevice;
use trussed::{
    backend::Dispatch,
    types::{Bytes, Location},
};

use apps::InitStatus;

use crate::{
    soc::Soc,
    store::{RunnerStore, StoragePointers},
    types::{Apps, Runner, Trussed},
    ui::{buttons::UserPresence, rgb_led::RgbLed},
};

#[cfg(feature = "board-nk3am")]
pub mod nk3am;
#[cfg(feature = "board-nk3xn")]
pub mod nk3xn;
#[cfg(feature = "board-nkpk")]
pub mod nkpk;

pub trait Board: StoragePointers {
    type Soc: Soc;

    type Apps: Apps;
    type Dispatch: Dispatch;

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
    const USB_PRODUCT: &'static str;

    fn init_apps(
        trussed: &mut Trussed<Self>,
        init_status: InitStatus,
        store: &RunnerStore<Self>,
        nfc_powered: bool,
    ) -> Self::Apps
    where
        Self: Sized;

    fn init_dispatch(
        hw_key: Option<&[u8]>,
        #[cfg(feature = "se050")] se050: Option<se05x::se05x::Se05X<Self::Twi, Self::Se050Timer>>,
    ) -> Self::Dispatch;

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

fn init_nk3_apps<B: Board>(
    init_status: InitStatus,
    store: &RunnerStore<B>,
    nfc_powered: bool,
) -> (Runner<B>, apps::Data<Runner<B>>) {
    let mut admin = apps::AdminData::new(*store, B::Soc::VARIANT);
    admin.init_status = init_status;
    if !nfc_powered {
        admin.update_blocks();
    }

    #[cfg(feature = "provisioner")]
    let provisioner = {
        use apps::Reboot as _;

        let store = store.clone();
        let int_flash_ref = unsafe { crate::store::steal_internal_storage::<B>() };
        let rebooter: fn() -> ! = B::Soc::reboot_to_firmware_update;

        apps::ProvisionerData {
            store,
            stolen_filesystem: int_flash_ref,
            nfc_powered,
            rebooter,
        }
    };

    let runner = Runner {
        is_efs_available: !nfc_powered,
        _marker: Default::default(),
    };
    let data = apps::Data {
        admin,
        #[cfg(feature = "provisioner")]
        provisioner,
        _marker: Default::default(),
    };
    (runner, data)
}

fn init_nk3_dispatch<B: Board>(
    hw_key: Option<&[u8]>,
    #[cfg(feature = "se050")] se050: Option<se05x::se05x::Se05X<B::Twi, B::Se050Timer>>,
) -> apps::Dispatch<B::Twi, B::Se050Timer> {
    if let Some(hw_key) = hw_key {
        apps::Dispatch::with_hw_key(
            Location::Internal,
            Bytes::from_slice(&hw_key).unwrap(),
            #[cfg(feature = "se050")]
            se050,
        )
    } else {
        apps::Dispatch::new(
            Location::Internal,
            #[cfg(feature = "se050")]
            se050,
        )
    }
}
