use core::time::Duration;

pub mod buttons;
pub mod rgb_led;

pub trait Clock {
    fn uptime(&mut self) -> Duration;
}
