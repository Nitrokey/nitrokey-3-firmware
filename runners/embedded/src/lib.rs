#![no_std]

use interchange::Interchange;
use littlefs2::fs::Filesystem;
use types::Soc;
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};

extern crate delog;
delog::generate_macros!();

pub mod flash;
pub mod runtime;
pub mod store;
pub mod traits;
pub mod types;
pub mod ui;

#[cfg(not(any(feature = "soc-lpc55", feature = "soc-nrf52840")))]
compile_error!("No SoC chosen!");

#[cfg_attr(feature = "soc-nrf52840", path = "soc_nrf52840/mod.rs")]
#[cfg_attr(feature = "soc-lpc55", path = "soc_lpc55/mod.rs")]
pub mod soc;

#[cfg(feature = "provisioner-app")]
use admin_app::Reboot;

pub fn banner<S: Soc>() {
    info!(
        "Embedded Runner ({}:{}) using librunner {}.{}.{}",
        S::SOC_NAME,
        S::BOARD_NAME,
        types::build_constants::CARGO_PKG_VERSION_MAJOR,
        types::build_constants::CARGO_PKG_VERSION_MINOR,
        types::build_constants::CARGO_PKG_VERSION_PATCH
    );
}

fn transcend<T>(global: &'static mut Option<T>, content: T) -> &'static mut T {
    global.replace(content);
    global.as_mut().unwrap()
}

pub fn init_store<S: Soc>(
    int_flash: S::InternalFlashStorage,
    ext_flash: S::ExternalFlashStorage,
) -> types::RunnerStore<S> {
    let volatile_storage = types::VolatileStorage::new();

    /* Step 1: let our stack-based filesystem objects transcend into higher
    beings by blessing them with static lifetime
    */
    let internal = unsafe { S::internal_storage() };
    let external = unsafe { S::external_storage() };

    let ifs_storage = transcend(&mut internal.storage, int_flash);
    let ifs_alloc = transcend(&mut internal.alloc, Filesystem::allocate());
    let efs_storage = transcend(&mut external.storage, ext_flash);
    let efs_alloc = transcend(&mut external.alloc, Filesystem::allocate());
    let (vfs_storage, vfs_alloc) = unsafe {
        (
            transcend(&mut types::VOLATILE_STORAGE.storage, volatile_storage),
            transcend(&mut types::VOLATILE_STORAGE.alloc, Filesystem::allocate()),
        )
    };

    /* Step 2: try mounting each FS in turn */
    if !littlefs2::fs::Filesystem::is_mountable(ifs_storage) {
        let _fmt_ext = littlefs2::fs::Filesystem::format(ifs_storage);
        error!("IFS Mount Error, Reformat {:?}", _fmt_ext);
    };
    let ifs = match littlefs2::fs::Filesystem::mount(ifs_alloc, ifs_storage) {
        Ok(ifs_) => transcend(&mut internal.fs, ifs_),
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
        Ok(efs_) => transcend(&mut external.fs, efs_),
        Err(_e) => {
            error!("EFS Mount Error {:?}", _e);
            panic!("store");
        }
    };

    if !littlefs2::fs::Filesystem::is_mountable(vfs_storage) {
        littlefs2::fs::Filesystem::format(vfs_storage).ok();
    }
    let vfs = match littlefs2::fs::Filesystem::mount(vfs_alloc, vfs_storage) {
        Ok(vfs_) => unsafe { transcend(&mut types::VOLATILE_STORAGE.fs, vfs_) },
        Err(_e) => {
            error!("VFS Mount Error {:?}", _e);
            panic!("store");
        }
    };

    types::RunnerStore::init_raw(ifs, efs, vfs)
}

pub fn init_usb_nfc<S: Soc>(
    usbbus_opt: Option<&'static usb_device::bus::UsbBusAllocator<S::UsbBus>>,
    nfcdev_opt: Option<S::NfcDevice>,
) -> types::usbnfc::UsbNfcInit<S> {
    let config = S::INTERFACE_CONFIG;

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

#[cfg(feature = "provisioner-app")]
pub fn init_apps<S: Soc>(
    trussed: &mut types::Trussed<S>,
    store: &types::RunnerStore<S>,
    on_nfc_power: bool,
) -> types::Apps<S> {
    let store_2 = store.clone();
    let int_flash_ref = unsafe { S::internal_storage().storage.as_mut().unwrap() };
    let uuid: [u8; 16] = S::device_uuid();
    let rebooter: fn() -> ! = S::Reboot::reboot_to_firmware_update;

    let pnp = types::ProvisionerNonPortable {
        store: store_2,
        stolen_filesystem: int_flash_ref,
        nfc_powered: on_nfc_power,
        uuid,
        rebooter,
    };
    types::Apps::new(trussed, pnp)
}

#[cfg(not(feature = "provisioner-app"))]
pub fn init_apps<S: Soc>(
    trussed: &mut types::Trussed<S>,
    _store: &types::RunnerStore<S>,
    _on_nfc_power: bool,
) -> types::Apps<S> {
    types::Apps::new(trussed)
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
