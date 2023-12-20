#![no_std]
#![warn(trivial_casts, unused, unused_qualifications)]

use apdu_dispatch::{
    dispatch::ApduDispatch,
    interchanges::{Channel as CcidChannel, Responder as CcidResponder},
};
use apps::InitStatus;
use ctaphid_dispatch::{dispatch::Dispatch as CtaphidDispatch, types::Channel as CtapChannel};
use interchange::Channel;
use nfc_device::Iso14443;
use ref_swap::OptionRefSwap;
use trussed::interrupt::InterruptFlag;
use usb_device::{
    bus::UsbBusAllocator,
    device::{UsbDeviceBuilder, UsbVidPid},
};

use store::RunnerStore;
use types::{
    usbnfc::{UsbClasses, UsbNfcInit},
    Soc,
};

delog::generate_macros!();

pub mod flash;
pub mod runtime;
#[macro_use]
pub mod store;
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
    let (ccid_rq, ccid_rp) = CCID_CHANNEL.split().unwrap();
    let (ctaphid_rq, ctaphid_rp) = CTAP_CHANNEL.split().unwrap();

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
    store: &RunnerStore<S>,
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
        let int_flash_ref = unsafe { store::steal_internal_storage::<S>() };
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
