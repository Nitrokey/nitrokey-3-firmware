#![no_std]

use apdu_dispatch::{
    dispatch::ApduDispatch,
    interchanges::{Channel as CcidChannel, Responder as CcidResponder},
};
use apps::InitStatus;
use ctaphid_dispatch::{dispatch::Dispatch as CtaphidDispatch, types::Channel as CtapChannel};
use delog_panic::DelogPanic as _;
use interchange::Channel;
use littlefs2::fs::Filesystem;
use nfc_device::Iso14443;
use ref_swap::OptionRefSwap;
use soc::types::Soc as SocT;
use trussed::interrupt::InterruptFlag;
use types::{
    usbnfc::{UsbClasses, UsbNfcInit},
    Soc,
};
use usb_device::{
    bus::UsbBusAllocator,
    device::{UsbDeviceBuilder, UsbVidPid},
};

#[cfg(feature = "board-nk3am")]
use soc::migrations::ftl_journal;
#[cfg(feature = "board-nk3am")]
use soc::migrations::ftl_journal::ifs_flash_old::FlashStorage as OldFlashStorage;

extern crate delog;
delog::generate_macros!();

pub mod flash;
pub mod runtime;
pub mod traits;
pub mod types;
pub mod ui;

#[cfg(not(any(feature = "soc-lpc55", feature = "soc-nrf52840")))]
compile_error!("No SoC chosen!");

#[cfg_attr(feature = "soc-nrf52840", path = "soc_nrf52840/mod.rs")]
#[cfg_attr(feature = "soc-lpc55", path = "soc_lpc55/mod.rs")]
pub mod soc;

#[cfg(feature = "alloc")]
#[global_allocator]
static ALLOCATOR: alloc_cortex_m::CortexMHeap = alloc_cortex_m::CortexMHeap::empty();

pub fn banner<S: Soc>() {
    info!(
        "Embedded Runner ({}:{}) using librunner {}",
        S::SOC_NAME,
        S::BOARD_NAME,
        utils::VERSION_STRING,
    );
}

#[cfg(feature = "alloc")]
pub fn init_alloc() {
    use core::mem::MaybeUninit;
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE) }
}

pub fn init_store(
    int_flash: <SocT as Soc>::InternalFlashStorage,
    ext_flash: <SocT as Soc>::ExternalFlashStorage,
    simulated_efs: bool,
    status: &mut InitStatus,
) -> types::RunnerStore {
    let volatile_storage = types::VolatileStorage::new();

    /* Step 1: let our stack-based filesystem objects transcend into higher
    beings by blessing them with static lifetime
    */
    macro_rules! transcend {
        ($global:expr, $content:expr) => {
            unsafe {
                $global.replace($content);
                $global.as_mut().delog_unwrap()
            }
        };
    }

    let ifs_storage = transcend!(types::INTERNAL_STORAGE, int_flash);
    let ifs_alloc = transcend!(types::INTERNAL_FS_ALLOC, Filesystem::allocate());
    let efs_storage = transcend!(types::EXTERNAL_STORAGE, ext_flash);
    let efs_alloc = transcend!(types::EXTERNAL_FS_ALLOC, Filesystem::allocate());
    let vfs_storage = transcend!(types::VOLATILE_STORAGE, volatile_storage);
    let vfs_alloc = transcend!(types::VOLATILE_FS_ALLOC, Filesystem::allocate());

    /* Step 2: try mounting each FS in turn */
    if !Filesystem::is_mountable(ifs_storage) {
        // handle provisioner
        if cfg!(feature = "provisioner") {
            info_now!("IFS mount failed - provisioner => formatting");
            let _fmt_int = Filesystem::format(ifs_storage);
        } else {
            status.insert(InitStatus::INTERNAL_FLASH_ERROR);

            // handle lpc55 boards
            #[cfg(feature = "board-nk3xn")]
            {
                let _fmt_int = Filesystem::format(ifs_storage);
                error_now!("IFS (lpc55) mount-fail");
            }

            // handle nRF42 boards
            #[cfg(feature = "board-nk3am")]
            {
                error_now!("IFS (nrf42) mount-fail");

                // regular mount failed, try mounting "old" (pre-journaling) IFS
                let pac = unsafe { nrf52840_pac::Peripherals::steal() };
                let mut old_ifs_storage = OldFlashStorage::new(pac.NVMC);
                let mut old_ifs_alloc: littlefs2::fs::Allocation<OldFlashStorage> =
                    Filesystem::allocate();
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
                    // migration failed => format IFS
                    } else {
                        error_now!("failed migration, formatting IFS");
                        let _fmt_ifs = Filesystem::format(ifs_storage);
                    }
                } else {
                    info_now!("recovering from journal");
                    // IFS and old-IFS cannot be mounted, try to recover from journal
                    ifs_storage.recover_from_journal();
                }
            }
        }
    }

    #[cfg(feature = "board-nk3am")]
    ifs_storage.format_journal_blocks();

    let ifs_ = Filesystem::mount(ifs_alloc, ifs_storage).expect("Could not bring up IFS!");
    let ifs = transcend!(types::INTERNAL_FS, ifs_);

    if !littlefs2::fs::Filesystem::is_mountable(efs_storage) {
        let fmt_ext = littlefs2::fs::Filesystem::format(efs_storage);
        if simulated_efs && fmt_ext == Err(littlefs2::io::Error::NoSpace) {
            info_now!("Formatting simulated EFS failed as expected");
        } else {
            error_now!("EFS Mount Error, Reformat {:?}", fmt_ext);
            status.insert(InitStatus::EXTERNAL_FLASH_ERROR);
        }
    };
    let efs = match littlefs2::fs::Filesystem::mount(efs_alloc, efs_storage) {
        Ok(efs_) => {
            transcend!(types::EXTERNAL_FS, efs_)
        }
        Err(_e) => {
            error!("EFS Mount Error {:?}", _e);
            panic!("store");
        }
    };

    if !littlefs2::fs::Filesystem::is_mountable(vfs_storage) {
        littlefs2::fs::Filesystem::format(vfs_storage).ok();
    }
    let vfs = match littlefs2::fs::Filesystem::mount(vfs_alloc, vfs_storage) {
        Ok(vfs_) => {
            transcend!(types::VOLATILE_FS, vfs_)
        }
        Err(_e) => {
            error!("VFS Mount Error {:?}", _e);
            panic!("store");
        }
    };

    types::RunnerStore::init_raw(ifs, efs, vfs)
}

pub fn init_usb_nfc<S: Soc>(
    usbbus_opt: Option<&'static UsbBusAllocator<S::UsbBus>>,
    nfc: Option<Iso14443<S::NfcDevice>>,
    nfc_rp: CcidResponder<'static>,
) -> UsbNfcInit<S> {
    let config = &types::INTERFACE_CONFIG;

    static CCID_CHANNEL: CcidChannel = Channel::new();
    static CTAP_CHANNEL: CtapChannel = Channel::new();
    static CTAP_INTERRUPT: OptionRefSwap<'static, InterruptFlag> = OptionRefSwap::new(None);

    /* claim interchanges */
    let (ccid_rq, ccid_rp) = CCID_CHANNEL.split().delog_unwrap();
    let (ctaphid_rq, ctaphid_rp) = CTAP_CHANNEL.split().delog_unwrap();

    /* initialize dispatchers */
    let apdu_dispatch = ApduDispatch::new(ccid_rp, nfc_rp);
    let ctaphid_dispatch = CtaphidDispatch::with_interrupt(ctaphid_rp, Some(&CTAP_INTERRUPT));

    /* populate requesters (if bus options are provided) */
    let mut usb_classes = None;

    if let Some(usbbus) = usbbus_opt {
        /* Class #1: CCID */
        let ccid = usbd_ccid::Ccid::new(usbbus, ccid_rq, Some(config.card_issuer));

        /* Class #2: CTAPHID */
        let ctaphid =
            usbd_ctaphid::CtapHid::with_interrupt(usbbus, ctaphid_rq, Some(&CTAP_INTERRUPT), 0u32)
                .implements_ctap1()
                .implements_ctap2()
                .implements_wink();

        let vidpid = UsbVidPid(config.usb_id_vendor, config.usb_id_product);
        let usbd = UsbDeviceBuilder::new(usbbus, vidpid)
			.product(config.usb_product)
			.manufacturer(config.usb_manufacturer)
			/*.serial_number(config.usb_serial)  <---- don't configure serial to not be identifiable */
			.device_release(utils::VERSION.usb_release())
			.max_packet_size_0(64)
			.composite_with_iads()
			.build();

        usb_classes = Some(UsbClasses {
            usbd,
            ccid,
            ctaphid,
        });
    }

    UsbNfcInit {
        usb_classes,
        apdu_dispatch,
        ctaphid_dispatch,
        iso14443: nfc,
    }
}

pub fn init_apps<S: Soc>(
    trussed: &mut types::Trussed<S>,
    init_status: InitStatus,
    store: &types::RunnerStore,
    nfc_powered: bool,
) -> types::Apps<S> {
    use trussed::platform::Store as _;

    let mut admin = apps::AdminData::new(*store, S::VARIANT);
    admin.init_status = init_status;
    if !nfc_powered {
        if let Ok(ifs_blocks) = store.ifs().available_blocks() {
            if let Ok(ifs_blocks) = u8::try_from(ifs_blocks) {
                admin.ifs_blocks = ifs_blocks;
            }
        }
        if let Ok(efs_blocks) = store.efs().available_blocks() {
            if let Ok(efs_blocks) = u16::try_from(efs_blocks) {
                admin.efs_blocks = efs_blocks;
            }
        }
    }

    #[cfg(feature = "provisioner")]
    let provisioner = {
        let store = store.clone();
        let int_flash_ref = unsafe { types::INTERNAL_STORAGE.as_mut().delog_unwrap() };
        let rebooter: fn() -> ! = S::reboot_to_firmware_update;

        apps::ProvisionerData {
            store,
            stolen_filesystem: int_flash_ref,
            nfc_powered,
            rebooter,
        }
    };

    let runner = types::Runner {
        is_efs_available: !nfc_powered,
        _marker: Default::default(),
    };
    let data = apps::Data {
        admin,
        #[cfg(feature = "provisioner")]
        provisioner,
        _marker: Default::default(),
    };
    types::Apps::with_service(&runner, trussed, data)
}

#[cfg(feature = "se050")]
pub fn init_se050<
    I2C: se05x::t1::I2CForT1,
    D: embedded_hal::blocking::delay::DelayUs<u32>,
    R: rand::CryptoRng + rand::RngCore,
>(
    i2c: I2C,
    delay: D,
    dev_rng: &mut R,
    init_status: &mut InitStatus,
) -> (se05x::se05x::Se05X<I2C, D>, rand_chacha::ChaCha8Rng) {
    use rand::{Rng as _, SeedableRng};
    use rand_chacha::ChaCha8Rng;
    use se05x::se05x::commands::GetRandom;

    let seed: [u8; 32] = dev_rng.gen();
    let mut se050 = se05x::se05x::Se05X::new(i2c, 0x48, delay);
    let seed = (|| {
        se050.enable()?;
        let buf = &mut [0; 100];
        let se050_rand = se050.run_command(&GetRandom { length: 32.into() }, buf)?;
        let mut s: [u8; 32] = se050_rand
            .data
            .try_into()
            .or(Err(se05x::se05x::Error::Unknown))?;
        for (se050, orig) in s.iter_mut().zip(seed) {
            *se050 ^= orig;
        }
        Ok::<_, se05x::se05x::Error>(s)
    })()
    .unwrap_or_else(|_err| {
        debug_now!("Got error when getting SE050 initial entropy: {_err:?}");
        *init_status |= InitStatus::SE050_RAND_ERROR;
        seed
    });
    (se050, ChaCha8Rng::from_seed(seed))
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    error_now!("{}", _info);
    soc::board::set_panic_led();
    loop {
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
}
