use core::marker::PhantomData;

pub use apdu_dispatch::{
    command::SIZE as ApduCommandSize, response::SIZE as ApduResponseSize, App as ApduApp,
};
use apps::{Dispatch, Reboot, Variant};
use cortex_m::interrupt::InterruptNumber;
pub use ctaphid_dispatch::app::App as CtaphidApp;
#[cfg(feature = "se050")]
use embedded_hal::blocking::delay::DelayUs;
use embedded_time::duration::Milliseconds;
use littlefs2::{
    const_ram_storage,
    fs::{Allocation, Filesystem},
};
use nfc_device::traits::nfc::Device as NfcDevice;
use rand_chacha::ChaCha8Rng;
use trussed::{
    types::{LfsResult, LfsStorage},
    Platform,
};
use usb_device::bus::UsbBus;

use crate::{
    store::{RunnerStore, StoragePointers},
    ui::{buttons::UserPresence, rgb_led::RgbLed, Clock, UserInterface},
};

pub mod usbnfc;

pub struct Config {
    pub card_issuer: &'static [u8; 13],
    pub usb_product: &'static str,
    pub usb_manufacturer: &'static str,
    // pub usb_release: u16 --> taken from utils::VERSION::usb_release()
    pub usb_id_vendor: u16,
    pub usb_id_product: u16,
}

pub const INTERFACE_CONFIG: Config = Config {
    // zero-padding for compatibility with previous implementations
    card_issuer: b"Nitrokey\0\0\0\0\0",
    usb_product: "Nitrokey 3",
    usb_manufacturer: "Nitrokey",
    usb_id_vendor: 0x20A0,
    usb_id_product: 0x42B2,
};

pub type Uuid = [u8; 16];

pub trait Board: StoragePointers {
    type Soc: Soc;

    type NfcDevice: NfcDevice;
    type Buttons: UserPresence;
    type Led: RgbLed;

    #[cfg(feature = "se050")]
    type Se050Timer: DelayUs<u32> + 'static;
    #[cfg(feature = "se050")]
    type Twi: se05x::t1::I2CForT1 + 'static;
    #[cfg(not(feature = "se050"))]
    type Se050Timer: 'static;
    #[cfg(not(feature = "se050"))]
    type Twi: 'static;

    const BOARD_NAME: &'static str;

    fn prepare_ifs(ifs: &mut Self::InternalStorage) {
        let _ = ifs;
    }

    fn recover_ifs(
        ifs_storage: &mut Self::InternalStorage,
        ifs_alloc: &mut Allocation<Self::InternalStorage>,
        efs_storage: &mut Self::ExternalStorage,
    ) -> LfsResult<()> {
        let _ = (ifs_alloc, efs_storage);
        Filesystem::format(ifs_storage)
    }
}

pub trait Soc: Reboot + 'static {
    type UsbBus: UsbBus + 'static;
    type Clock: Clock;

    type Duration: From<Milliseconds>;

    type Interrupt: InterruptNumber;
    const SYSCALL_IRQ: Self::Interrupt;

    const SOC_NAME: &'static str;
    const VARIANT: Variant;

    fn device_uuid() -> &'static Uuid;
}

pub struct Runner<B> {
    pub is_efs_available: bool,
    pub _marker: PhantomData<B>,
}

impl<B: Board> apps::Runner for Runner<B> {
    type Syscall = RunnerSyscall<B::Soc>;
    type Reboot = B::Soc;
    type Store = RunnerStore<B>;
    #[cfg(feature = "provisioner")]
    type Filesystem = B::InternalStorage;
    type Twi = B::Twi;
    type Se050Timer = B::Se050Timer;

    fn uuid(&self) -> [u8; 16] {
        *B::Soc::device_uuid()
    }

    fn is_efs_available(&self) -> bool {
        self.is_efs_available
    }
}

// 8KB of RAM
const_ram_storage!(
    name = VolatileStorage,
    trait = LfsStorage,
    erase_value = 0xff,
    read_size = 16,
    write_size = 256,
    cache_size_ty = littlefs2::consts::U256,
    // We use 256 instead of the default 512 to avoid loosing too much space to nearly empty blocks containing only folder metadata.
    block_size = 256,
    block_count = 8192/256,
    lookahead_size_ty = littlefs2::consts::U1,
    filename_max_plus_one_ty = littlefs2::consts::U256,
    path_max_plus_one_ty = littlefs2::consts::U256,
    result = LfsResult,
);

pub struct RunnerPlatform<B: Board> {
    pub rng: ChaCha8Rng,
    pub store: RunnerStore<B>,
    pub user_interface: UserInterface<B>,
}

unsafe impl<B: Board> Platform for RunnerPlatform<B> {
    type R = ChaCha8Rng;
    type S = RunnerStore<B>;
    type UI = UserInterface<B>;

    fn user_interface(&mut self) -> &mut Self::UI {
        &mut self.user_interface
    }

    fn rng(&mut self) -> &mut Self::R {
        &mut self.rng
    }

    fn store(&self) -> Self::S {
        self.store
    }
}

pub struct RunnerSyscall<S: Soc> {
    _marker: PhantomData<S>,
}

impl<S: Soc> Default for RunnerSyscall<S> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<S: Soc> trussed::client::Syscall for RunnerSyscall<S> {
    #[inline]
    fn syscall(&mut self) {
        rtic::pend(S::SYSCALL_IRQ);
    }
}

pub type Trussed<B> =
    trussed::Service<RunnerPlatform<B>, Dispatch<<B as Board>::Twi, <B as Board>::Se050Timer>>;

pub type ApduDispatch = apdu_dispatch::dispatch::ApduDispatch<'static>;
pub type CtaphidDispatch = ctaphid_dispatch::dispatch::Dispatch<'static, 'static>;

pub type Apps<S> = apps::Apps<Runner<S>>;

#[derive(Debug)]
pub struct DelogFlusher {}

impl delog::Flusher for DelogFlusher {
    fn flush(&self, _msg: &str) {
        #[cfg(feature = "log-rtt")]
        rtt_target::rprint!(_msg);

        #[cfg(feature = "log-semihosting")]
        cortex_m_semihosting::hprint!(_msg).ok();
    }
}

pub static DELOG_FLUSHER: DelogFlusher = DelogFlusher {};
