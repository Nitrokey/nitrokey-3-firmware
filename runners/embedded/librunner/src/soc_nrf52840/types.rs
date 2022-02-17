use littlefs2::const_ram_storage;
use nrf52840_hal::usbd::{Usbd, UsbPeripheral};
use trussed::platform::{consent, reboot, ui};
use trussed::types::{LfsStorage, LfsResult};

const_ram_storage!(FlashStorage, 4096);
const_ram_storage!(ExternalStorage, 8192);

pub type UsbBus<'a> = Usbd<UsbPeripheral<'a>>;

pub type Rng = chacha20::ChaCha8Rng;

pub const SYSCALL_IRQ: nrf52840_pac::Interrupt = nrf52840_pac::Interrupt::SWI0_EGU0;

pub struct TrussedUI {
}

impl TrussedUI {
	pub fn new() -> Self { Self {} }
}

impl trussed::platform::UserInterface for TrussedUI {
	fn check_user_presence(&mut self) -> consent::Level {
		consent::Level::None
	}

	fn set_status(&mut self, _status: ui::Status) {
		info!("UI SetStatus");
	}

	fn refresh(&mut self) {}

	fn uptime(&mut self) -> core::time::Duration {
		// let _cyccnt = cortex_m::peripheral::DWT::get_cycle_count();
		core::time::Duration::new(0, 0)
	}

	fn reboot(&mut self, _to: reboot::To) -> ! {
		error!("TrussedUI::reboot() is deprecated!");
		panic!();
	}
}

