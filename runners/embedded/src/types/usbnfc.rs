use apdu_dispatch::interchanges::SIZE as CCID_SIZE;
use nfc_device::Iso14443;
use usb_device::device::UsbDevice;
use usbd_ccid::Ccid;
use usbd_ctaphid::CtapHid;

use crate::types::{ApduDispatch, Board, CtaphidDispatch, Soc};

pub struct UsbClasses<S: Soc> {
    pub usbd: UsbDevice<'static, S::UsbBus>,
    pub ccid: Ccid<'static, 'static, S::UsbBus, CCID_SIZE>,
    pub ctaphid: CtapHid<'static, 'static, 'static, S::UsbBus>,
}

impl<S: Soc> UsbClasses<S> {
    pub fn poll(&mut self) {
        self.ctaphid.check_for_app_response();
        self.ccid.check_for_app_response();
        self.usbd.poll(&mut [&mut self.ccid, &mut self.ctaphid]);
    }
}

pub struct UsbNfcInit<B: Board> {
    pub usb_classes: Option<UsbClasses<B::Soc>>,
    pub apdu_dispatch: ApduDispatch,
    pub ctaphid_dispatch: CtaphidDispatch,
    pub iso14443: Option<Iso14443<B::NfcDevice>>,
}
