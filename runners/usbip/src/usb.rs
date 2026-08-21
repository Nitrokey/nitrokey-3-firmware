//! USB classes for the virtual device, built from the same component as the
//! firmware so both present identical descriptors.

use std::time::Instant;

use apdu_dispatch::{dispatch::ApduDispatch, interchanges::Channel as CcidChannel};
use apps::{Apps, Dispatch};
use ctaphid_dispatch::Channel as CtapChannel;
use interchange::Channel;
use ref_swap::OptionRefSwap;
use trussed_core::InterruptFlag;
use trussed_usbip::{
    usb_device::bus::UsbBusAllocator, Classes, Dispatches, Options, Setup, Timeout, UsbIpBus,
};
use usb_classes::Config;

use crate::Runner;

/// Matches the firmware, which caps the message size to reduce stack usage.
pub const CTAPHID_MESSAGE_SIZE: usize = 3072;

const CARD_ISSUER: &[u8; 13] = b"Nitrokey\0\0\0\0\0";

type UsbClasses = usb_classes::UsbClasses<UsbIpBus, CTAPHID_MESSAGE_SIZE>;
type CtaphidDispatch = ctaphid_dispatch::Dispatch<'static, 'static, CTAPHID_MESSAGE_SIZE>;

pub struct NkSetup {
    pub manufacturer: &'static str,
    pub product: &'static str,
    pub vid: u16,
    pub pid: u16,
    pub device_release: u16,
}

pub struct NkClasses {
    classes: UsbClasses,
    timeout_ccid: Timeout,
    timeout_ctaphid: Timeout,
}

pub struct NkDispatches {
    ctaphid: CtaphidDispatch,
    apdu: ApduDispatch<'static>,
}

impl Setup<Dispatch> for NkSetup {
    type Classes = NkClasses;
    type Dispatches = NkDispatches;

    fn setup(
        self,
        allocator: &'static UsbBusAllocator<UsbIpBus>,
        _options: &'static Options,
    ) -> (NkClasses, NkDispatches) {
        static CCID_CHANNEL: CcidChannel = Channel::new();
        static CONTACTLESS_CHANNEL: CcidChannel = Channel::new();
        static CTAP_CHANNEL: CtapChannel<CTAPHID_MESSAGE_SIZE> = Channel::new();
        static CTAP_INTERRUPT: OptionRefSwap<'static, InterruptFlag> = OptionRefSwap::new(None);

        let (ccid_rq, ccid_rp) = CCID_CHANNEL.split().unwrap();
        let (ctaphid_rq, ctaphid_rp) = CTAP_CHANNEL.split().unwrap();

        let ccid = if cfg!(feature = "ccid") {
            Some(usb_classes::CcidConfig {
                requester: ccid_rq,
                card_issuer: Some(CARD_ISSUER),
            })
        } else {
            None
        };

        let classes = usb_classes::build(
            allocator,
            ccid,
            ctaphid_rq,
            &CTAP_INTERRUPT,
            Config {
                manufacturer: self.manufacturer,
                product: self.product,
                vid: self.vid,
                pid: self.pid,
                device_release: self.device_release,
            },
        );

        // There is no contactless interface here; the dispatcher needs a second
        // responder regardless.
        let apdu = ApduDispatch::new(ccid_rp, CONTACTLESS_CHANNEL.split().unwrap().1);
        let ctaphid = CtaphidDispatch::with_interrupt(ctaphid_rp, Some(&CTAP_INTERRUPT));

        (
            NkClasses {
                classes,
                timeout_ccid: Timeout::new(),
                timeout_ctaphid: Timeout::new(),
            },
            NkDispatches { ctaphid, apdu },
        )
    }
}

impl Classes for NkClasses {
    fn poll(&mut self) {
        self.classes.poll();
    }

    fn keepalive(&mut self, epoch: Instant) {
        if let Some(ccid) = &mut self.classes.ccid {
            trussed_usbip::ccid::keepalive(ccid, &mut self.timeout_ccid, epoch);
        }
        trussed_usbip::ctaphid::keepalive(
            &mut self.classes.ctaphid,
            &mut self.timeout_ctaphid,
            epoch,
        );
    }
}

impl Dispatches<Apps<Runner>> for NkDispatches {
    fn poll(&mut self, apps: &mut Apps<Runner>) {
        apps.ctaphid_dispatch(|apps| self.ctaphid.poll(apps));
        apps.apdu_dispatch(|apps| self.apdu.poll(apps));
    }
}
