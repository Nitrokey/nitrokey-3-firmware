//! Construction and polling of the device's USB classes.
//!
//! Shared by the firmware and the USB/IP runner so that both present the same
//! descriptors, ATR and message sizes.

#![cfg_attr(not(test), no_std)]
#![warn(trivial_casts, unused, unused_qualifications)]

delog::generate_macros!();

#[cfg(feature = "storage")]
pub mod storage;

use apdu_dispatch::interchanges::{Requester as CcidRequester, SIZE as CCID_SIZE};
use ctaphid_dispatch::Requester as CtapRequester;
use embedded_time::duration::Milliseconds;
use heapless::Vec;
use ref_swap::OptionRefSwap;
use trussed_core::InterruptFlag;
use usb_device::{
    bus::{UsbBus, UsbBusAllocator},
    class::UsbClass,
    device::{StringDescriptors, UsbDevice, UsbDeviceBuilder, UsbVidPid},
    LangID,
};
use usbd_ccid::Ccid;
use usbd_ctaphid::CtapHid;

/// CCID, CTAPHID and one caller-supplied class.
const MAX_CLASSES: usize = 3;

/// Identification and descriptor settings for the USB device.
pub struct Config<'a> {
    pub manufacturer: &'a str,
    pub product: &'a str,
    pub vid: u16,
    pub pid: u16,
    /// `bcdDevice`.
    pub device_release: u16,
}

/// Enables the CCID interface. Omitting it leaves the interface out entirely.
pub struct CcidConfig<'a> {
    pub requester: CcidRequester<'static>,
    /// CCID historical bytes; `None` yields the generic ATR.
    pub card_issuer: Option<&'a [u8]>,
}

/// The USB device together with the CTAPHID class and, optionally, CCID.
pub struct UsbClasses<B: UsbBus + 'static, const CTAP_N: usize> {
    pub usbd: UsbDevice<'static, B>,
    pub ccid: Option<Ccid<'static, 'static, B, CCID_SIZE>>,
    pub ctaphid: CtapHid<'static, 'static, 'static, B, CTAP_N>,
}

impl<B: UsbBus + 'static, const CTAP_N: usize> UsbClasses<B, CTAP_N> {
    /// Runs one iteration of the USB poll loop.
    ///
    /// [`UsbDevice::poll`] only polls classes on bus activity, so queued
    /// application responses have to be picked up first.
    pub fn poll(&mut self) {
        self.poll_with(&mut []);
    }

    /// As [`UsbClasses::poll`], additionally polling caller-owned classes such
    /// as mass storage.
    pub fn poll_with(&mut self, extra: &mut [&mut dyn UsbClass<B>]) {
        self.ctaphid.check_for_app_response();
        if let Some(ccid) = &mut self.ccid {
            ccid.check_for_app_response();
        }

        let mut classes: Vec<&mut dyn UsbClass<B>, MAX_CLASSES> = Vec::new();
        if let Some(ccid) = &mut self.ccid {
            classes.push(ccid).ok();
        }
        classes.push(&mut self.ctaphid).ok();
        for class in extra {
            classes.push(&mut **class).ok();
        }

        self.usbd.poll(&mut classes);
    }
}

/// Allocates the classes and builds the device.
///
/// The device is built last: building freezes the allocator and any later
/// endpoint, interface or string allocation panics.
pub fn build<B: UsbBus + 'static, const CTAP_N: usize>(
    bus: &'static UsbBusAllocator<B>,
    ccid: Option<CcidConfig<'static>>,
    ctaphid_rq: CtapRequester<'static, CTAP_N>,
    ctap_interrupt: &'static OptionRefSwap<'static, InterruptFlag>,
    config: Config<'static>,
) -> UsbClasses<B, CTAP_N> {
    let ccid = ccid.map(|ccid| Ccid::new(bus, ccid.requester, ccid.card_issuer));
    let ctaphid = CtapHid::with_interrupt(bus, ctaphid_rq, Some(ctap_interrupt), 0u32)
        .implements_ctap1()
        .implements_ctap2()
        .implements_wink();

    let strings = StringDescriptors::new(LangID::EN)
        .product(config.product)
        .manufacturer(config.manufacturer);
    let usbd = UsbDeviceBuilder::new(bus, UsbVidPid(config.vid, config.pid))
        .strings(&[strings])
        .expect("failed to set USB string descriptors")
        .device_release(config.device_release)
        .max_packet_size_0(64)
        .expect("invalid max packet size for EP0")
        .composite_with_iads()
        .build();

    UsbClasses {
        usbd,
        ccid,
        ctaphid,
    }
}

/// Delay after which a CCID wait extension or CTAPHID keepalive is due.
pub trait Keepalive {
    fn delay(self) -> Option<Milliseconds>;
}

impl Keepalive for usbd_ccid::Status {
    fn delay(self) -> Option<Milliseconds> {
        match self {
            usbd_ccid::Status::ReceivedData(ms) => Some(ms),
            usbd_ccid::Status::Idle => None,
        }
    }
}

impl Keepalive for usbd_ctaphid::types::Status {
    fn delay(self) -> Option<Milliseconds> {
        match self {
            usbd_ctaphid::types::Status::ReceivedData(ms) => Some(ms),
            usbd_ctaphid::types::Status::Idle => None,
        }
    }
}
