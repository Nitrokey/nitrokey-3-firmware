include!(concat!(env!("OUT_DIR"), "/build_constants.rs"));

use core::{cell::RefCell, time::Duration};

use crate::hal;
use hal::drivers::timer;
use interchange::Interchange;
use littlefs2::{const_ram_storage, consts};
use trussed::types::ClientContext;
use trussed::types::{LfsResult, LfsStorage};
use trussed::{platform, store};
use hal::peripherals::ctimer;
use hal::traits::wg::{blocking::spi::Transfer, digital::v2::OutputPin};
use spi_memory::{BlockDevice, Read};

#[cfg(feature = "no-encrypted-storage")]
use hal::littlefs2_filesystem;
#[cfg(not(feature = "no-encrypted-storage"))]
use hal::littlefs2_prince_filesystem;

#[cfg(feature = "no-encrypted-storage")]
littlefs2_filesystem!(PlainFilesystem: (build_constants::CONFIG_FILESYSTEM_BOUNDARY));
#[cfg(not(feature = "no-encrypted-storage"))]
littlefs2_prince_filesystem!(PrinceFilesystem: (build_constants::CONFIG_FILESYSTEM_BOUNDARY));

#[cfg(feature = "no-encrypted-storage")]
pub type FlashStorage = PlainFilesystem;
#[cfg(not(feature = "no-encrypted-storage"))]
pub type FlashStorage = PrinceFilesystem;

pub mod usb;
pub use usb::{UsbClasses, EnabledUsbPeripheral, SerialClass, CcidClass, CtapHidClass};

use board::shared::Reboot;

// 8KB of RAM
const_ram_storage!(
    name=VolatileStorage,
    trait=LfsStorage,
    erase_value=0xff,
    read_size=1,
    write_size=1,
    cache_size_ty=consts::U128,
    // this is a limitation of littlefs
    // https://git.io/JeHp9
    block_size=128,
    // block_size=128,
    block_count=8192/104,
    lookaheadwords_size_ty=consts::U8,
    filename_max_plus_one_ty=consts::U256,
    path_max_plus_one_ty=consts::U256,
    result=LfsResult,
);

// minimum: 2 blocks
// TODO: make this optional
const_ram_storage!(ExternalStorage, 1024);

store!(Store,
    Internal: FlashStorage,
    External: ExternalStorage,
    Volatile: VolatileStorage
);

struct FlashProperties {
	size: usize,
	jedec: [u8; 3],
	_cont: u8,
}

const FLASH_PROPERTIES: FlashProperties = FlashProperties {
	size: 0x20_0000,
	jedec: [0xc8, 0x40, 0x15],
	_cont: 0 /* should be 6, but device doesn't report those */
};

pub struct ExtFlashStorage<SPI, CS> where SPI: Transfer<u8>, CS: OutputPin {
	s25flash: RefCell<spi_memory::series25::Flash<SPI, CS>>,
}

impl<SPI, CS> littlefs2::driver::Storage for ExtFlashStorage<SPI, CS> where SPI: Transfer<u8>, CS: OutputPin {

	const BLOCK_SIZE: usize = 4096;
	const READ_SIZE: usize = 4;
	const WRITE_SIZE: usize = 256;
	const BLOCK_COUNT: usize = FLASH_PROPERTIES.size / Self::BLOCK_SIZE;
	type CACHE_SIZE = generic_array::typenum::U256;
	type LOOKAHEADWORDS_SIZE = generic_array::typenum::U1;

	fn read(&self, off: usize, buf: &mut [u8]) -> Result<usize, littlefs2::io::Error> {
		trace!("EFr {:x} {:x}", off, buf.len());
		if buf.len() == 0 { return Ok(0); }
		if buf.len() > FLASH_PROPERTIES.size ||
			off > FLASH_PROPERTIES.size - buf.len() {
			return Err(littlefs2::io::Error::Unknown(0x6578_7046));
		}
        let mut flash = self.s25flash.borrow_mut();
		let r = flash.read(off as u32, buf);
		if r.is_ok() { trace!("r >>> {}", delog::hex_str!(&buf[0..4])); }
		map_result(r, buf.len())
	}

	fn write(&mut self, off: usize, data: &[u8]) -> Result<usize, littlefs2::io::Error> {
		trace!("EFw {:x} {:x}", off, data.len());
		trace!("w >>> {}", delog::hex_str!(&data[0..4]));
        const CHUNK_SIZE: usize = 256;
        let mut buf = [0; CHUNK_SIZE];
        let mut off = off as u32;
        let mut flash = self.s25flash.borrow_mut();
        for chunk in data.chunks(CHUNK_SIZE) {
            let buf = &mut buf[..chunk.len()];
            buf.copy_from_slice(chunk);
            flash
                .write_bytes(off, buf)
                .map_err(|_| littlefs2::io::Error::Unknown(0x6565_6565))?;
            off += CHUNK_SIZE as u32;
        }
        Ok(data.len())
	}

	fn erase(&mut self, off: usize, len: usize) -> Result<usize, littlefs2::io::Error> {
		trace!("EFe {:x} {:x}", off, len);
		if len > FLASH_PROPERTIES.size ||
			off > FLASH_PROPERTIES.size - len {
			return Err(littlefs2::io::Error::Unknown(0x6578_7046));
		}
        let result = self.s25flash.borrow_mut().erase_sectors(off as u32, len / 256);
		map_result(result, len)
	}
}

fn map_result<SPI, CS>(r: Result<(), spi_memory::Error<SPI, CS>>, len: usize)
			-> Result<usize, littlefs2::io::Error>
			where SPI: Transfer<u8>, CS: OutputPin {
	match r {
		Ok(()) => Ok(len),
		Err(_) => Err(littlefs2::io::Error::Unknown(0x6565_6565))
	}
}

impl<SPI, CS> ExtFlashStorage<SPI, CS> where SPI: Transfer<u8>, CS: OutputPin {

	fn raw_command(spim: &mut SPI, cs: &mut CS, buf: &mut [u8]) {
		cs.set_low().ok().unwrap();
		spim.transfer(buf).ok().unwrap();
		cs.set_high().ok().unwrap();
	}

	pub fn new(mut spim: SPI, mut cs: CS) -> Self {
		Self::selftest(&mut spim, &mut cs);

		let mut flash = spi_memory::series25::Flash::init(spim, cs).ok().unwrap();
		let jedec_id = flash.read_jedec_id().ok().unwrap();
		info!("Ext. Flash: {:?}", jedec_id);
		if jedec_id.mfr_code() != FLASH_PROPERTIES.jedec[0] ||
			jedec_id.device_id() != &FLASH_PROPERTIES.jedec[1..] {
			panic!("Unknown Ext. Flash!");
		}
        let s25flash = RefCell::new(flash);

		Self { s25flash }
	}

	pub fn selftest(spim: &mut SPI, cs: &mut CS) {
		macro_rules! doraw {
			($buf:expr, $len:expr, $str:expr) => {
			let mut buf: [u8; $len] = $buf;
			Self::raw_command(spim, cs, &mut buf);
			trace!($str, delog::hex_str!(&buf[1..]));
		}}

		doraw!([0x9f, 0, 0, 0], 4, "JEDEC {}");
		doraw!([0x05, 0], 2, "RDSRl {}");
		doraw!([0x35, 0], 2, "RDSRh {}");
	}

	pub fn size(&self) -> usize {
		FLASH_PROPERTIES.size
	}

	pub fn erase_chip(&mut self) -> Result<usize, littlefs2::io::Error> {
		map_result(self.s25flash.borrow_mut().erase_all(), FLASH_PROPERTIES.size)
	}
}

pub struct SpiMut<'a, SPI: Transfer<u8>>(pub &'a mut SPI);

impl<'a, SPI: Transfer<u8>> Transfer<u8> for SpiMut<'a, SPI> {
    type Error = SPI::Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.0.transfer(words)
    }
}

pub type ThreeButtons = board::ThreeButtons;
pub type RgbLed = board::RgbLed;

platform!(Board,
    R: hal::peripherals::rng::Rng<hal::Enabled>,
    S: Store,
    UI: board::trussed::UserInterface<ThreeButtons, RgbLed>,
);

#[derive(Default)]
pub struct Syscall {}

impl trussed::client::Syscall for Syscall {
    #[inline]
    fn syscall(&mut self) {
        rtic::pend(board::hal::raw::Interrupt::OS_EVENT);
    }
}

pub type Trussed = trussed::Service<Board>;
pub type TrussedClient = trussed::ClientImplementation<Syscall>;

pub type Iso14443 = nfc_device::Iso14443<board::nfc::NfcChip>;

pub type ExternalInterrupt = hal::Pint<hal::typestates::init_state::Enabled>;

pub type ApduDispatch = apdu_dispatch::dispatch::ApduDispatch;
pub type CtaphidDispatch = ctaphid_dispatch::dispatch::Dispatch;

#[cfg(feature = "admin-app")]
pub type AdminApp = admin_app::App<TrussedClient, Reboot>;
#[cfg(feature = "oath-authenticator")]
pub type OathApp = oath_authenticator::Authenticator<TrussedClient>;
#[cfg(feature = "fido-authenticator")]
pub type FidoApp = fido_authenticator::Authenticator<fido_authenticator::Conforming, TrussedClient>;
#[cfg(feature = "fido-authenticator")]
pub type FidoConfig = fido_authenticator::Config;
#[cfg(feature = "ndef-app")]
pub type NdefApp = ndef_app::App<'static>;
#[cfg(feature = "provisioner-app")]
pub type ProvisionerApp = provisioner_app::Provisioner<Store, FlashStorage, TrussedClient>;

pub type WebcryptApp = Webcrypt<TrussedClient>;

use apdu_dispatch::{App as ApduApp, command::SIZE as CommandSize, response::SIZE as ResponseSize};
use ctaphid_dispatch::app::{App as CtaphidApp};
use webcrypt::Webcrypt;

pub type DynamicClockController = board::clock_controller::DynamicClockController;
pub type NfcWaitExtender = timer::Timer<ctimer::Ctimer0<hal::typestates::init_state::Enabled>>;
pub type PerformanceTimer = timer::Timer<ctimer::Ctimer4<hal::typestates::init_state::Enabled>>;

pub trait TrussedApp: Sized {

    /// non-portable resources needed by this Trussed app
    type NonPortable;

    /// the desired client ID
    const CLIENT_ID: &'static [u8];
    const ENCRYPTED: bool = false;

    fn with_client(trussed: TrussedClient, non_portable: Self::NonPortable) -> Self;

    fn with(trussed: &mut trussed::Service<crate::Board>, non_portable: Self::NonPortable) -> Self {
        let (trussed_requester, trussed_responder) = trussed::pipe::TrussedInterchange::claim()
            .expect("could not setup TrussedInterchange");

        let mut client_id = littlefs2::path::PathBuf::new();
        client_id.push(Self::CLIENT_ID.try_into().unwrap());

        let pin = if cfg!(feature = "transparent-encryption") && Self::ENCRYPTED { Some("1234") } else { None };  // FIXME replace with DEFAULT_ENCRYPTION_PIN
        let client_ctx = ClientContext::new(littlefs2::path::PathBuf::from(Self::CLIENT_ID), pin);
        assert!(trussed.add_endpoint(trussed_responder, client_ctx).is_ok());

        let syscaller = Syscall::default();
        let trussed_client = TrussedClient::new(
            trussed_requester,
            syscaller,
        );

        let app = Self::with_client(trussed_client, non_portable);
        app
    }
}

#[cfg(feature = "oath-authenticator")]
impl TrussedApp for OathApp {
    const CLIENT_ID: &'static [u8] = b"oath\0";

    type NonPortable = ();
    fn with_client(trussed: TrussedClient, _: ()) -> Self {
        Self::new(trussed)
    }
}

#[cfg(feature = "admin-app")]
impl TrussedApp for AdminApp {
    const CLIENT_ID: &'static [u8] = b"admin\0";

    // TODO: declare uuid + version
    type NonPortable = ();
    fn with_client(trussed: TrussedClient, _: ()) -> Self {
        Self::new(trussed, hal::uuid(), build_constants::CARGO_PKG_VERSION)
    }
}

impl TrussedApp for WebcryptApp {
    type NonPortable = ();

    const CLIENT_ID: &'static [u8] = b"webcrypt\0";
    const ENCRYPTED: bool = true;

    fn with_client(trussed: TrussedClient, _: ()) -> Self {
        Self::new(trussed)
    }
}

#[cfg(feature = "fido-authenticator")]
impl TrussedApp for FidoApp {
    const CLIENT_ID: &'static [u8] = b"fido\0";

    type NonPortable = ();
    fn with_client(trussed: TrussedClient, _: ()) -> Self {
        let authnr = fido_authenticator::Authenticator::new(
            trussed,
            fido_authenticator::Conforming {},
            FidoConfig {
                max_msg_size: usbd_ctaphid::constants::MESSAGE_SIZE,
                // max_creds_in_list: ctap_types::sizes::MAX_CREDENTIAL_COUNT_IN_LIST,
                // max_cred_id_length: ctap_types::sizes::MAX_CREDENTIAL_ID_LENGTH,
                skip_up_timeout: Some(Duration::from_secs(2)),
            },
        );

        // Self::new(authnr)
        authnr
    }
}

pub struct ProvisionerNonPortable {
    pub store: Store,
    pub stolen_filesystem: &'static mut FlashStorage,
    pub nfc_powered: bool,
    pub uuid: [u8; 16],
    pub rebooter: fn() -> !,
}

#[cfg(feature = "provisioner-app")]
impl TrussedApp for ProvisionerApp {
    const CLIENT_ID: &'static [u8] = b"attn\0";

    type NonPortable = ProvisionerNonPortable;
    fn with_client(trussed: TrussedClient, ProvisionerNonPortable { store, stolen_filesystem, nfc_powered, uuid, rebooter }: Self::NonPortable) -> Self {
        Self::new(trussed, store, stolen_filesystem, nfc_powered, uuid, rebooter)
    }

}

pub struct Apps {
    #[cfg(feature = "admin-app")]
    pub admin: AdminApp,
    #[cfg(feature = "fido-authenticator")]
    pub fido: FidoApp,
    #[cfg(feature = "oath-authenticator")]
    pub oath: OathApp,
    #[cfg(feature = "ndef-app")]
    pub ndef: NdefApp,
    #[cfg(feature = "provisioner-app")]
    pub provisioner: ProvisionerApp,
    pub webcrypt: WebcryptApp,
}

impl Apps {
    pub fn new(
        trussed: &mut trussed::Service<crate::Board>,
        #[cfg(feature = "provisioner-app")]
        provisioner: ProvisionerNonPortable
    ) -> Self {
        #[cfg(feature = "admin-app")]
        let admin = AdminApp::with(trussed, ());
        #[cfg(feature = "fido-authenticator")]
        let fido = FidoApp::with(trussed, ());
        #[cfg(feature = "oath-authenticator")]
        let oath = OathApp::with(trussed, ());
        #[cfg(feature = "ndef-app")]
        let ndef = NdefApp::new();
        #[cfg(feature = "provisioner-app")]
        let provisioner = ProvisionerApp::with(trussed, provisioner);
        let webcrypt = WebcryptApp::with(trussed, ());

        Self {
            #[cfg(feature = "admin-app")]
            admin,
            #[cfg(feature = "fido-authenticator")]
            fido,
            #[cfg(feature = "oath-authenticator")]
            oath,
            #[cfg(feature = "ndef-app")]
            ndef,
            #[cfg(feature = "provisioner-app")]
            provisioner,
            webcrypt,
        }
    }

    #[inline(never)]
    pub fn apdu_dispatch<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut [&mut dyn
                ApduApp<CommandSize, ResponseSize>
            ]) -> T
    {
        f(&mut [
            &mut self.webcrypt,
            #[cfg(feature = "ndef-app")]
            &mut self.ndef,
            #[cfg(feature = "oath-authenticator")]
            &mut self.oath,
            #[cfg(feature = "fido-authenticator")]
            &mut self.fido,
            #[cfg(feature = "admin-app")]
            &mut self.admin,
            #[cfg(feature = "provisioner-app")]
            &mut self.provisioner,
        ])
    }

    #[inline(never)]
    pub fn ctaphid_dispatch<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut [&mut dyn CtaphidApp ]) -> T
    {
        f(&mut [
            &mut self.webcrypt,
            #[cfg(feature = "fido-authenticator")]
            &mut self.fido,
            #[cfg(feature = "admin-app")]
            &mut self.admin,
        ])
    }
}
