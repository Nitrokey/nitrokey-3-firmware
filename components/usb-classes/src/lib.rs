//! Construction and polling of the device's USB classes.
//!
//! Shared by the firmware and the USB/IP runner so that both present the same
//! descriptors, ATR and message sizes.

#![cfg_attr(not(test), no_std)]
#![warn(trivial_casts, unused, unused_qualifications)]

delog::generate_macros!();

#[cfg(feature = "scsi")]
pub mod scsi;

use apdu_dispatch::interchanges::{Requester as CcidRequester, SIZE as CCID_SIZE};
use ctaphid_dispatch::Requester as CtapRequester;
use embedded_time::duration::Milliseconds;
use heapless::{String, Vec};
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
use utils::{Model, Version};

/// CCID, CTAPHID and maybe SCSI.
const MAX_CLASSES: usize = 2 + if cfg!(feature = "scsi") { 1 } else { 0 };

const CARD_ISSUER: &[u8; 13] = b"Nitrokey\0\0\0\0\0";

const MANUFACTURER: &str = "Nitrokey";
const PRODUCT_NK3: &str = "Nitrokey 3";
const PRODUCT_NKPK: &str = "Nitrokey Passkey";

const VID: u16 = 0x20a0;
const PID_NK3: u16 = 0x42b2;
const PID_NKPK: u16 = 0x42f3;

pub struct Serial(String<32>);

impl Serial {
    pub fn from_uuid(uuid: [u8; 16]) -> Self {
        const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
        let mut s = String::new();
        for byte in uuid {
            s.push(HEX_CHARS[usize::from(byte >> 4)].into()).unwrap();
            s.push(HEX_CHARS[usize::from(byte & 0xf)].into()).unwrap();
        }
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Identification and descriptor settings for the USB device.
pub struct Config<'a> {
    pub model: Model,
    pub version: Version,
    pub serial: &'a Serial,
}

impl<'a> Config<'a> {
    const fn product(&self) -> &'static str {
        match self.model {
            Model::NK3 => PRODUCT_NK3,
            Model::NKPK => PRODUCT_NKPK,
        }
    }

    const fn pid(&self) -> u16 {
        match self.model {
            Model::NK3 => PID_NK3,
            Model::NKPK => PID_NKPK,
        }
    }
}

/// The USB device together with the CTAPHID class and, optionally, CCID and/or SCSI.
pub struct UsbClasses<'a, B: UsbBus + 'static, const CTAP_N: usize> {
    pub usbd: UsbDevice<'a, B>,
    pub ccid: Option<Ccid<'a, 'static, B, CCID_SIZE>>,
    pub ctaphid: CtapHid<'a, 'static, 'static, B, CTAP_N>,
    #[cfg(feature = "scsi")]
    scsi: scsi::Scsi<'a, B>,
    #[cfg(feature = "scsi")]
    scsi_state: scsi::State,
}

impl<B: UsbBus + 'static, const CTAP_N: usize> UsbClasses<'_, B, CTAP_N> {
    /// Runs one iteration of the USB poll loop.
    ///
    /// [`UsbDevice::poll`] only polls classes on bus activity, so queued
    /// application responses have to be picked up first.
    pub fn poll(&mut self) {
        self.ctaphid.check_for_app_response();
        if let Some(ccid) = &mut self.ccid {
            ccid.check_for_app_response();
        }

        let mut classes: Vec<&mut dyn UsbClass<B>, MAX_CLASSES> = Vec::new();
        if let Some(ccid) = &mut self.ccid {
            classes.push(ccid).ok();
        }
        classes.push(&mut self.ctaphid).ok();
        #[cfg(feature = "scsi")]
        classes.push(&mut self.scsi).ok();

        self.usbd.poll(&mut classes);
    }

    #[cfg(feature = "scsi")]
    pub fn poll_scsi<D: scsi::BlockDevice>(&mut self, device: &mut D) {
        scsi::poll(&mut self.usbd, &mut self.scsi, device, &mut self.scsi_state)
    }
}

/// Allocates the classes and builds the device.
///
/// The device is built last: building freezes the allocator and any later
/// endpoint, interface or string allocation panics.
pub fn build<'a, B: UsbBus + 'static, const CTAP_N: usize>(
    bus: &'a UsbBusAllocator<B>,
    ccid_rq: Option<CcidRequester<'static>>,
    ctaphid_rq: CtapRequester<'static, CTAP_N>,
    ctap_interrupt: &'static OptionRefSwap<'static, InterruptFlag>,
    config: Config<'a>,
) -> UsbClasses<'a, B, CTAP_N> {
    let ccid = ccid_rq.map(|ccid_rq| Ccid::new(bus, ccid_rq, Some(CARD_ISSUER)));
    let ctaphid = CtapHid::with_interrupt(bus, ctaphid_rq, Some(ctap_interrupt), 0u32)
        .implements_ctap1()
        .implements_ctap2()
        .implements_wink();
    #[cfg(feature = "scsi")]
    let scsi = scsi::setup(bus);

    let strings = StringDescriptors::new(LangID::EN_US)
        .product(config.product())
        .manufacturer(MANUFACTURER)
        .serial_number(config.serial.as_str());
    let usbd = UsbDeviceBuilder::new(bus, UsbVidPid(VID, config.pid()))
        .strings(&[strings])
        .expect("failed to set USB string descriptors")
        .device_release(config.version.usb_release())
        .max_packet_size_0(64)
        .expect("invalid max packet size for EP0")
        .composite_with_iads()
        .build();

    UsbClasses {
        usbd,
        ccid,
        ctaphid,
        #[cfg(feature = "scsi")]
        scsi,
        #[cfg(feature = "scsi")]
        scsi_state: scsi::State::default(),
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
