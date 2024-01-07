use core::marker::PhantomData;

use apdu_dispatch::{
    command::SIZE as ApduCommandSize, dispatch::Interface, response::SIZE as ApduResponseSize,
    App as ApduApp,
};
use ctaphid_dispatch::app::App as CtaphidApp;
use littlefs2::const_ram_storage;
use rand_chacha::ChaCha8Rng;
use trussed::{
    types::{LfsResult, LfsStorage},
    Platform,
};

use crate::{board::Board, soc::Soc, store::RunnerStore, ui::UserInterface};

pub mod usbnfc;

pub struct Config {
    pub card_issuer: &'static [u8; 13],
    pub usb_product: &'static str,
    pub usb_manufacturer: &'static str,
    // pub usb_release: u16 --> taken from utils::VERSION::usb_release()
    pub usb_id_vendor: u16,
    pub usb_id_product: u16,
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

pub type Trussed<B> = trussed::Service<RunnerPlatform<B>, <B as Board>::Dispatch>;

pub type ApduDispatch = apdu_dispatch::dispatch::ApduDispatch<'static>;
pub type CtaphidDispatch = ctaphid_dispatch::dispatch::Dispatch<'static, 'static>;

pub trait Apps {
    fn apdu_dispatch<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut [&mut dyn ApduApp<ApduCommandSize, ApduResponseSize>]) -> T;

    fn ctaphid_dispatch<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut [&mut dyn CtaphidApp<'static>]) -> T;

    fn poll_dispatchers(
        &mut self,
        apdu_dispatch: &mut ApduDispatch,
        ctaphid_dispatch: &mut CtaphidDispatch,
    ) -> (bool, bool) {
        let apdu_poll = self.apdu_dispatch(|apps| apdu_dispatch.poll(apps));
        let ctaphid_poll = self.ctaphid_dispatch(|apps| ctaphid_dispatch.poll(apps));

        (
            apdu_poll == Some(Interface::Contact) || ctaphid_poll,
            apdu_poll == Some(Interface::Contactless),
        )
    }
}

impl<B: Board> Apps for apps::Apps<Runner<B>> {
    fn apdu_dispatch<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut [&mut dyn ApduApp<ApduCommandSize, ApduResponseSize>]) -> T,
    {
        apps::Apps::apdu_dispatch(self, f)
    }

    fn ctaphid_dispatch<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut [&mut dyn CtaphidApp<'static>]) -> T,
    {
        apps::Apps::ctaphid_dispatch(self, f)
    }
}

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
