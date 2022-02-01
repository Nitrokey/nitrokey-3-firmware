use nrf52840_hal::{
	clocks::{Clocks, Internal, ExternalOscillator, LfOscStarted},
	usbd::{Usbd, UsbPeripheral},
};
use trussed::{
	Interchange
};
use usb_device::{
	bus::UsbBusAllocator,
	device::{UsbDevice, UsbDeviceBuilder, UsbVidPid},
};

type XUsbd<'a> = Usbd<UsbPeripheral<'a>>;
// type LFClockType = Clocks<Internal, Internal, LfOscStarted>;
type LFHFClockType = Clocks<ExternalOscillator, Internal, LfOscStarted>;

pub struct USBPreinitObjects {
	usb_pac: nrf52840_pac::USBD,
	clk: LFHFClockType
}

static mut USBD: Option<UsbBusAllocator<XUsbd>> = None;
static mut USBCLK: Option<LFHFClockType> = None;

pub struct USBObjects<'a> {
	usbdevice: UsbDevice<'a, XUsbd<'static>>,
	ccid_class: usbd_ccid::Ccid<XUsbd<'static>, apdu_dispatch::interchanges::Contact, {apdu_dispatch::interchanges::SIZE}>,
	ctaphid_class: usbd_ctaphid::CtapHid<'a, XUsbd<'static>>,
}

pub struct USBDispatcher {
	apdu_dispatch: apdu_dispatch::dispatch::ApduDispatch,
	ctaphid_dispatch: ctaphid_dispatch::dispatch::Dispatch,
}

pub fn preinit(usb_pac: nrf52840_pac::USBD, clk: nrf52840_hal::clocks::Clocks<ExternalOscillator, Internal, LfOscStarted>) -> USBPreinitObjects {
	USBPreinitObjects { usb_pac, clk }
}

pub fn init(preinit: USBPreinitObjects) -> (USBObjects<'static>, USBDispatcher) {
	preinit.usb_pac.intenset.write(|w| w
			.usbreset().set_bit()
			.usbevent().set_bit()
			.sof().set_bit()
			/* .epdata().set_bit() */
			.ep0datadone().set_bit()
			.ep0setup().set_bit());

	unsafe { USBCLK.replace(preinit.clk); }
	let usbclk_ref = unsafe { USBCLK.as_ref().unwrap() };

	let usb_peripheral = UsbPeripheral::new(preinit.usb_pac, usbclk_ref);
	unsafe { USBD.replace(Usbd::new(usb_peripheral)); }
	let usbd_ref = unsafe { USBD.as_ref().unwrap() };

	trace!("USB: Glbl ok");

	/* Class #1: CCID */
	let (ccid_rq, ccid_rp) = apdu_dispatch::interchanges::Contact::claim().unwrap();
	let (_nfc_rq, nfc_rp) = apdu_dispatch::interchanges::Contactless::claim().unwrap();
	let ccid = usbd_ccid::Ccid::new(usbd_ref, ccid_rq, Some(b"PTB/EMC"));
	let apdu_dispatch = apdu_dispatch::dispatch::ApduDispatch::new(ccid_rp, nfc_rp);

	/* Class #2: CTAPHID */
	let (ctaphid_rq, ctaphid_rp) = ctaphid_dispatch::types::HidInterchange::claim().unwrap();
	let ctaphid = usbd_ctaphid::CtapHid::new(usbd_ref, ctaphid_rq, 0u32)
			.implements_ctap1()
			.implements_ctap2()
			.implements_wink();
	let ctaphid_dispatch = ctaphid_dispatch::dispatch::Dispatch::new(ctaphid_rp);

	/* Finally: create device */
	let usbdevice = UsbDeviceBuilder::new(usbd_ref, UsbVidPid(0x1209, 0x5090))
			.product("EMC Stick").manufacturer("Nitrokey/PTB")
			.serial_number("imagine-a-uuid-here")
			.device_release(0x0001u16)
			.max_packet_size_0(64)
			.composite_with_iads()
			.build();

	trace!("USB: Objx ok");

	( USBObjects { usbdevice, ccid_class: ccid, ctaphid_class: ctaphid },
		USBDispatcher { apdu_dispatch, ctaphid_dispatch } )
}

impl USBObjects<'static> {
	// Polls for activity from the host (called from the USB IRQ handler) //
	pub fn poll(&mut self) {
		self.ccid_class.check_for_app_response();
		self.ctaphid_class.check_for_app_response();
		self.usbdevice.poll(&mut [&mut self.ccid_class, &mut self.ctaphid_class]);
	}

	pub fn send_keepalives(&mut self) {
		if let usbd_ctaphid::types::Status::ReceivedData(_) = self.ctaphid_class.did_start_processing() {
			debug!("-KeepH");
			// self.ctaphid_class.send_keepalive(false);
		}
		if let usbd_ccid::types::Status::ReceivedData(_) = self.ccid_class.did_start_processing() {
			debug!("+KeepC");
			self.ccid_class.send_wait_extension();
		}
	}
}

impl USBDispatcher {
	// Polls for activity from the userspace applications (called during IDLE) //
	pub fn poll_ctaphid_apps(&mut self, ctaphid_apps: &mut [&mut dyn ctaphid_dispatch::app::App]) -> (bool, bool) {
		let raise_usb = self.ctaphid_dispatch.poll(ctaphid_apps);
		(raise_usb, false)
	}

	pub fn poll_apdu_apps(&mut self, apdu_apps: &mut [&mut dyn apdu_dispatch::app::App<{apdu_dispatch::command::SIZE}, {apdu_dispatch::response::SIZE}>]) -> (bool, bool) {
		let mut raise_usb = false;
		let mut raise_nfc = false;
		match self.apdu_dispatch.poll(apdu_apps) {
		Some(apdu_dispatch::dispatch::Interface::Contact) => { raise_usb = true; },
		Some(apdu_dispatch::dispatch::Interface::Contactless) => { raise_nfc = true; },
		_ => {}
		}
		(raise_usb, raise_nfc)
	}
}

macro_rules! bit_event {
($reg:expr, $shift:expr) => {
	match $reg.read().bits() {
	0 => { 0u32 },
	_ => { (1u32 << $shift) }
	}
};
}

#[allow(dead_code)]
pub fn usbd_debug_events() -> u32 {
	let mut v: u32 = 0;
	unsafe {
		let usb_pac = nrf52840_hal::pac::Peripherals::steal().USBD;
		for i in 0..8 {
			v |= bit_event!(usb_pac.events_endepin[i], 2+i);
			v |= bit_event!(usb_pac.events_endepout[i], 12+i);
		}
		v |= bit_event!(usb_pac.events_endisoin, 11);
		v |= bit_event!(usb_pac.events_endisoout, 20);
		v |= bit_event!(usb_pac.events_ep0datadone, 10);
		v |= bit_event!(usb_pac.events_ep0setup, 23);
		v |= bit_event!(usb_pac.events_epdata, 24);
		v |= bit_event!(usb_pac.events_sof, 21);
		v |= bit_event!(usb_pac.events_started, 1);
		v |= bit_event!(usb_pac.events_usbevent, 22);
		v |= bit_event!(usb_pac.events_usbreset, 0);
	}
	v
}
