use std::{
    process,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::{Duration, SystemTime},
};

use dialoguer::Confirm;
use log::{debug, info};
use signal_hook::{consts::signal::SIGUSR1, flag};
use trussed::platform::{consent, reboot, ui::Status};

pub struct UserInterface {
    start_time: std::time::Instant,
    user_presence: UserPresence,
    status: Status,
    cached_user_presence: Option<bool>,
    show_prompt: bool,
}

impl UserInterface {
    pub fn new(user_presence: UserPresence) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            user_presence,
            status: Status::Idle,
            cached_user_presence: None,
            show_prompt: false,
        }
    }

    fn is_user_present(&mut self) -> bool {
        if let Some(user_presence) = self.cached_user_presence {
            user_presence
        } else {
            match &self.user_presence {
                UserPresence::Fixed(user_present) => *user_present,
                UserPresence::Interactive => {
                    let user_present = Confirm::new()
                        .with_prompt("User presence?")
                        .interact()
                        .unwrap();
                    if !user_present {
                        self.cached_user_presence = Some(user_present);
                    }
                    user_present
                }
                UserPresence::Signal(signals) => {
                    if self.show_prompt {
                        eprintln!("Confirm user presence request with SIGUSR1.");
                        self.show_prompt = false;
                    }
                    signals.user_presence()
                }
            }
        }
    }
}

impl trussed::platform::UserInterface for UserInterface {
    fn check_user_presence(&mut self) -> consent::Level {
        // The call is repeated until it times out or returns something else than None so we cache
        // the user selection.
        let user_present = self.is_user_present();
        let consent = if user_present {
            consent::Level::Normal
        } else {
            consent::Level::None
        };
        debug!("Answering user presence check with consent level {consent:?}");
        if consent == consent::Level::None {
            thread::sleep(Duration::from_millis(100));
        }
        consent
    }

    fn set_status(&mut self, status: Status) {
        info!("Set status: {:?}", status);

        let is_waiting = status == Status::WaitingForUserPresence;
        trussed_usbip::set_waiting(is_waiting);
        if is_waiting {
            info!(">>>> Received confirmation request");
        } else if self.cached_user_presence.is_some() {
            debug!("Resetting cached user consent");
            self.cached_user_presence = None;
        }
        self.show_prompt = is_waiting && status != self.status;

        self.status = status;
    }

    fn refresh(&mut self) {}

    fn uptime(&mut self) -> core::time::Duration {
        self.start_time.elapsed()
    }

    fn reboot(&mut self, to: reboot::To) -> ! {
        info!("Restart!  ({:?})", to);
        process::exit(25);
    }
}

#[derive(Clone)]
pub enum UserPresence {
    Fixed(bool),
    Interactive,
    Signal(Arc<Signals>),
}

pub struct Signals {
    epoch: SystemTime,
    usr1: Arc<AtomicBool>,
    usr1_timeout: AtomicU64,
}

impl Signals {
    pub fn new() -> Self {
        let signals = Signals {
            epoch: SystemTime::now(),
            usr1: Default::default(),
            usr1_timeout: Default::default(),
        };
        flag::register(SIGUSR1, Arc::clone(&signals.usr1))
            .expect("failed to register signal handler");
        signals
    }

    fn user_presence(&self) -> bool {
        let timestamp = self.epoch.elapsed().unwrap();
        let timeout = Duration::from_millis(self.usr1_timeout.swap(0, Ordering::Relaxed));
        timestamp < timeout
    }

    pub fn update(&self) -> ! {
        loop {
            if self.usr1.swap(false, Ordering::Relaxed) {
                let timeout = self.epoch.elapsed().unwrap() + Duration::from_secs(1);
                let timeout_millis = timeout.as_millis().try_into().unwrap();
                self.usr1_timeout.store(timeout_millis, Ordering::Relaxed);
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}
