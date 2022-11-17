use crate::types::Soc;

pub type CcidClass<S> = usbd_ccid::Ccid<
    'static,
    <S as Soc>::UsbBus,
    apdu_dispatch::interchanges::Contact,
    { apdu_dispatch::interchanges::SIZE },
>;
pub type CtapHidClass<S> = usbd_ctaphid::CtapHid<'static, <S as Soc>::UsbBus>;
// pub type KeyboardClass<S> = usbd_hid::hid_class::HIDClass<'static, <S as Soc>::UsbBus>;
pub type SerialClass<S> = usbd_serial::SerialPort<'static, <S as Soc>::UsbBus>;

type Usbd<S> = usb_device::device::UsbDevice<'static, <S as Soc>::UsbBus>;

pub struct UsbClasses<S: Soc> {
    pub usbd: Usbd<S>,
    pub ccid: CcidClass<S>,
    pub ctaphid: CtapHidClass<S>,
    // pub keyboard: KeyboardClass,
    pub serial: SerialClass<S>,
}

impl<S: Soc> UsbClasses<S> {
    pub fn new(
        usbd: Usbd<S>,
        ccid: CcidClass<S>,
        ctaphid: CtapHidClass<S>,
        serial: SerialClass<S>,
    ) -> Self {
        Self {
            usbd,
            ccid,
            ctaphid,
            serial,
        }
    }
    pub fn poll(&mut self) {
        self.ctaphid.check_for_app_response();
        self.ccid.check_for_app_response();
        self.usbd
            .poll(&mut [&mut self.ccid, &mut self.ctaphid, &mut self.serial]);
    }
}

pub struct UsbNfcInit<S: Soc> {
    pub usb_classes: Option<UsbClasses<S>>,
    pub apdu_dispatch: apdu_dispatch::dispatch::ApduDispatch,
    pub ctaphid_dispatch: ctaphid_dispatch::dispatch::Dispatch,
    pub iso14443: Option<super::Iso14443<S>>,
}
