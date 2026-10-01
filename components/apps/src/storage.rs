use interchange::{Channel, Requester, Responder};
use littlefs2_core::{path, Path};
use trussed::{backend::BackendId, pipe::TrussedChannel};
use trussed_core::InterruptFlag;

use super::{App, Backend, Client, Runner};

pub type StorageApp<R> = storage_app::StorageApp<Client<R>, StorageCallback>;

pub type StorageChannel = Channel<StorageAction, ()>;
pub type StorageRequester<'a> = Requester<'a, StorageAction, ()>;
pub type StorageResponder<'a> = Responder<'a, StorageAction, ()>;

pub enum StorageAction {
    Lock,
    Unlock([u8; 32]),
}

pub struct StorageCallback(StorageRequester<'static>);

impl StorageCallback {
    pub fn new(rq: StorageRequester<'static>) -> Self {
        Self(rq)
    }

    fn send(&mut self, action: StorageAction) -> Result<(), storage_app::Error> {
        // discard any replies to free the channel
        self.0.take_response();
        self.0
            .request(action)
            .map_err(|_| storage_app::Error::InternalError)
    }
}

impl storage_app::Storage for StorageCallback {
    fn init(&mut self, _key: &[u8; 32]) -> Result<(), storage_app::Error> {
        info!("Storage initialized");
        Ok(())
    }

    fn unlock(&mut self, key: &[u8; 32]) -> Result<(), storage_app::Error> {
        info!("Storage unlocked");
        self.send(StorageAction::Unlock(*key))
    }

    fn lock(&mut self) -> Result<(), storage_app::Error> {
        info!("Storage locked");
        self.send(StorageAction::Lock)
    }
}

pub struct StorageData {
    pub requester: StorageRequester<'static>,
}

impl<R: Runner> App<R> for StorageApp<R> {
    const CLIENT_ID: &'static Path = path!("storage");

    type Data = StorageData;
    type Config = ();

    fn with_client(_runner: &R, trussed: Client<R>, data: Self::Data, _: &()) -> Self {
        Self::new(trussed, StorageCallback::new(data.requester))
    }

    fn channel() -> &'static TrussedChannel {
        static CHANNEL: TrussedChannel = TrussedChannel::new();
        &CHANNEL
    }

    fn interrupt() -> Option<&'static InterruptFlag> {
        static INTERRUPT: InterruptFlag = InterruptFlag::new();
        Some(&INTERRUPT)
    }

    fn backends(runner: &R, _: &()) -> &'static [BackendId<Backend>] {
        const BACKENDS_STORAGE: &[BackendId<Backend>] = &[
            #[cfg(feature = "se050")]
            BackendId::Custom(Backend::Se050),
            #[cfg(not(feature = "se050"))]
            BackendId::Custom(Backend::Auth),
            BackendId::Core,
        ];
        let _ = runner;
        BACKENDS_STORAGE
    }
}
