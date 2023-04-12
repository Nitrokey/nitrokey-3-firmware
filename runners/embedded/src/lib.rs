#![no_std]

use interchange::Interchange;
use littlefs2::fs::Filesystem;
use soc::types::Soc as SocT;
use types::Soc;
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};

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

pub fn banner() {
    info!(
        "Embedded Runner ({}:{}) using librunner {}",
        <SocT as Soc>::SOC_NAME,
        <SocT as Soc>::BOARD_NAME,
        utils::VERSION,
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
    status: &mut types::InitStatus,
) -> types::RunnerStore {
    let volatile_storage = types::VolatileStorage::new();

    /* Step 1: let our stack-based filesystem objects transcend into higher
    beings by blessing them with static lifetime
    */
    macro_rules! transcend {
        ($global:expr, $content:expr) => {
            unsafe {
                $global.replace($content);
                $global.as_mut().unwrap()
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
            status.insert(types::InitStatus::INTERNAL_FLASH_ERROR);

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

                info_now!("recovering from journal");
                // IFS and old-IFS cannot be mounted, try to recover from journal
                ifs_storage.recover_from_journal();
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
            // status.insert(types::InitStatus::EXTERNAL_FLASH_ERROR);
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

pub fn init_usb_nfc(
    usbbus_opt: Option<&'static usb_device::bus::UsbBusAllocator<<SocT as Soc>::UsbBus>>,
    nfcdev_opt: Option<<SocT as Soc>::NfcDevice>,
) -> types::usbnfc::UsbNfcInit {
    let config = <SocT as Soc>::INTERFACE_CONFIG;

    /* claim interchanges */
    let (ccid_rq, ccid_rp) = apdu_dispatch::interchanges::Contact::claim().unwrap();
    let (nfc_rq, nfc_rp) = apdu_dispatch::interchanges::Contactless::claim().unwrap();
    let (ctaphid_rq, ctaphid_rp) = ctaphid_dispatch::types::HidInterchange::claim().unwrap();

    /* initialize dispatchers */
    let apdu_dispatch = apdu_dispatch::dispatch::ApduDispatch::new(ccid_rp, nfc_rp);
    let ctaphid_dispatch = ctaphid_dispatch::dispatch::Dispatch::new(ctaphid_rp);

    /* populate requesters (if bus options are provided) */
    let mut usb_classes = None;

    if let Some(usbbus) = usbbus_opt {
        /* Class #1: CCID */
        let ccid = usbd_ccid::Ccid::new(usbbus, ccid_rq, Some(config.card_issuer));

        /* Class #2: CTAPHID */
        let ctaphid = usbd_ctaphid::CtapHid::new(usbbus, ctaphid_rq, 0u32)
            .implements_ctap1()
            .implements_ctap2()
            .implements_wink();

        /* Class #3: Serial */
        let serial = usbd_serial::SerialPort::new(usbbus);

        let vidpid = UsbVidPid(config.usb_id_vendor, config.usb_id_product);
        let usbdev = UsbDeviceBuilder::new(usbbus, vidpid)
			.product(config.usb_product)
			.manufacturer(config.usb_manufacturer)
			/*.serial_number(config.usb_serial)  <---- don't configure serial to not be identifiable */
			.device_release(utils::VERSION.usb_release())
			.max_packet_size_0(64)
			.composite_with_iads()
			.build();

        usb_classes = Some(types::usbnfc::UsbClasses::new(
            usbdev, ccid, ctaphid, serial,
        ));
    }

    // TODO: move up?
    let iso14443 = {
        if let Some(nfcdev) = nfcdev_opt {
            let mut iso14443 = nfc_device::Iso14443::new(nfcdev, nfc_rq);

            iso14443.poll();
            if true {
                // Give a small delay to charge up capacitors
                // basic_stage.delay_timer.start(5_000.microseconds()); nb::block!(basic_stage.delay_timer.wait()).ok();
            }

            Some(iso14443)
        } else {
            None
        }
    };

    types::usbnfc::UsbNfcInit {
        usb_classes,
        apdu_dispatch,
        ctaphid_dispatch,
        iso14443,
    }
}

pub fn init_apps(
    trussed: &mut types::Trussed,
    init_status: types::InitStatus,
    store: &types::RunnerStore,
    nfc_powered: bool,
) -> types::Apps {
    use trussed::platform::Store as _;

    let mut admin = apps::AdminData::new(<SocT as types::Soc>::VARIANT);
    admin.init_status = init_status.bits();
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
        use apps::Reboot;

        let store = store.clone();
        let int_flash_ref = unsafe { types::INTERNAL_STORAGE.as_mut().unwrap() };
        let rebooter: fn() -> ! = <SocT as types::Soc>::Reboot::reboot_to_firmware_update;

        apps::ProvisionerData {
            store,
            stolen_filesystem: int_flash_ref,
            nfc_powered,
            rebooter,
        }
    };

    let data = apps::Data {
        admin,
        #[cfg(feature = "provisioner")]
        provisioner,
        _marker: Default::default(),
    };
    types::Apps::with_service(&types::Runner, trussed, data)
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
