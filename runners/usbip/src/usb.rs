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

/// 16384 blocks of 512 bytes, i.e. 8 MiB.
#[cfg(feature = "usb-storage")]
const BLOCKS: u32 = 16384;

#[cfg(feature = "usb-storage")]
struct Storage {
    scsi: usb_classes::storage::StorageClass<'static, UsbIpBus, Vec<u8>>,
    device: crate::block_device::HostBlockDevice,
    state: usb_classes::storage::State,
}

pub struct NkSetup {
    pub manufacturer: &'static str,
    pub product: &'static str,
    pub vid: u16,
    pub pid: u16,
    pub device_release: u16,
    /// Backing file for the block device; memory when `None`.
    #[cfg(feature = "usb-storage")]
    pub block_device: Option<std::path::PathBuf>,
    #[cfg(feature = "usb-storage")]
    pub block_device_key: Option<[u8; 32]>,
}

pub struct NkClasses {
    classes: UsbClasses,
    timeout_ccid: Timeout,
    timeout_ctaphid: Timeout,
    #[cfg(feature = "usb-storage")]
    storage: Storage,
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

        // Must precede `build`, which freezes the allocator.
        #[cfg(feature = "usb-storage")]
        let storage = Storage {
            // usbip-device enumerates as high speed, so bulk endpoints are 512.
            scsi: usb_classes::storage::setup(allocator, 512, vec![0; 512]),
            device: crate::block_device::HostBlockDevice::open(
                self.block_device.as_deref(),
                BLOCKS,
                self.block_device_key,
            )
            .expect("failed to open block device"),
            state: Default::default(),
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
                #[cfg(feature = "usb-storage")]
                storage,
            },
            NkDispatches { ctaphid, apdu },
        )
    }
}

impl Classes for NkClasses {
    #[cfg(not(feature = "usb-storage"))]
    fn poll(&mut self) {
        self.classes.poll();
    }

    #[cfg(feature = "usb-storage")]
    fn poll(&mut self) {
        use trussed_usbip::usb_device::device::UsbDeviceState;

        let storage = &mut self.storage;
        self.classes.poll_with(&mut [&mut storage.scsi]);

        // A bus reset abandons any transfer that was in flight.
        if self.classes.usbd.state() == UsbDeviceState::Default {
            storage.state.reset();
        }

        // One `poll_command` per transport poll, and `UsbDevice::poll` did one
        // too: a from-host phase ending strands undrained data in the buffer.
        for _ in 0..2 {
            let result = storage.scsi.poll_command(|command| {
                usb_classes::storage::process_command(
                    command,
                    &mut storage.device,
                    &mut storage.state,
                )
            });
            if let Err(err) = result {
                log::warn!("storage: {err:?}");
            }
        }
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
