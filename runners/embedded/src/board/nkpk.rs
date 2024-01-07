use littlefs2::{fs::Allocation, io::Result as LfsResult};

use apps::{Apps, Dispatch, InitStatus};
use utils::RamStorage;

use super::{
    nk3am::{
        self,
        ui::{HardwareButtons, RgbLed},
        DummyNfc, InternalFlashStorage,
    },
    Board,
};
use crate::{
    soc::nrf52840::Nrf52,
    store::{impl_storage_pointers, RunnerStore},
    types::{Runner, Trussed},
};

pub struct NKPK;

impl Board for NKPK {
    type Soc = Nrf52;

    type Apps = Apps<Runner<Self>>;
    type Dispatch = Dispatch<Self::Twi, Self::Se050Timer>;

    type NfcDevice = DummyNfc;
    type Buttons = HardwareButtons;
    type Led = RgbLed;

    type Twi = ();
    type Se050Timer = ();

    const BOARD_NAME: &'static str = "NKPK";

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
        let _ = (ifs_alloc, efs_storage);
        error_now!("IFS (nkpk) mount-fail");
        info_now!("recovering from journal");
        ifs_storage.recover_from_journal();
        Ok(())
    }
}

// TODO: do we really want to mirror the NK3AM EFS?
pub type ExternalFlashStorage = RamStorage<nk3am::ExternalFlashStorage, 256>;

impl_storage_pointers!(
    NKPK,
    Internal = InternalFlashStorage,
    External = ExternalFlashStorage,
);
