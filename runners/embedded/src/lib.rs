#![no_std]
#![cfg_attr(feature = "alloc", feature(alloc_error_handler))]

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
        "Embedded Runner ({}:{}) using librunner {}.{}.{}",
        <SocT as Soc>::SOC_NAME,
        <SocT as Soc>::BOARD_NAME,
        types::build_constants::CARGO_PKG_VERSION_MAJOR,
        types::build_constants::CARGO_PKG_VERSION_MINOR,
        types::build_constants::CARGO_PKG_VERSION_PATCH
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
    if !littlefs2::fs::Filesystem::is_mountable(ifs_storage) {
        let _fmt_ext = littlefs2::fs::Filesystem::format(ifs_storage);
        error!("IFS Mount Error, Reformat {:?}", _fmt_ext);
    };
    let ifs = match littlefs2::fs::Filesystem::mount(ifs_alloc, ifs_storage) {
        Ok(ifs_) => {
            transcend!(types::INTERNAL_FS, ifs_)
        }
        Err(_e) => {
            error!("IFS Mount Error {:?}", _e);
            panic!("store");
        }
    };
    if !littlefs2::fs::Filesystem::is_mountable(efs_storage) {
        let _fmt_ext = littlefs2::fs::Filesystem::format(efs_storage);
        error!("EFS Mount Error, Reformat {:?}", _fmt_ext);
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
			.device_release(crate::types::build_constants::USB_RELEASE)
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
    _store: &types::RunnerStore,
    _on_nfc_power: bool,
) -> types::Apps {
    #[cfg(feature = "provisioner")]
    let provisioner = {
        use apps::Reboot;

        let store = _store.clone();
        let int_flash_ref = unsafe { types::INTERNAL_STORAGE.as_mut().unwrap() };
        let rebooter: fn() -> ! = <SocT as types::Soc>::Reboot::reboot_to_firmware_update;

        apps::ProvisionerNonPortable {
            store,
            stolen_filesystem: int_flash_ref,
            nfc_powered: _on_nfc_power,
            rebooter,
        }
    };
    let non_portable = apps::NonPortable {
        #[cfg(feature = "provisioner")]
        provisioner,
        _marker: Default::default(),
    };
    types::Apps::new(&types::Runner, trussed, non_portable)
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

#[cfg(feature = "alloc")]
#[alloc_error_handler]
fn oom(_: core::alloc::Layout) -> ! {
    error_now!("Failed alloc");
    loop {
        soc::board::set_panic_led();
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
}
