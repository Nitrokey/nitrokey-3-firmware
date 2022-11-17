//! Implementation of `trussed::Platform` for the board,
//! using the specific implementation of our `crate::traits`.

use core::time::Duration;

use crate::traits::{
    buttons::{Edge, Press},
    rgb_led::RgbLed,
};
use crate::{runtime::UserPresenceStatus, ui::Status};
use lpc55_hal::{peripherals::rtc::Rtc, typestates::init_state};
use trussed::platform::{consent, reboot};

pub struct UserInterface<BUTTONS, RGB>
where
    BUTTONS: Press + Edge,
    RGB: RgbLed,
{
    rtc: Rtc<init_state::Enabled>,
    buttons: Option<BUTTONS>,
    rgb: Option<RGB>,
    status: Status,
    provisioner: bool,
}

impl<BUTTONS, RGB> UserInterface<BUTTONS, RGB>
where
    BUTTONS: Press + Edge,
    RGB: RgbLed,
{
    pub fn new(
        rtc: Rtc<init_state::Enabled>,
        _buttons: Option<BUTTONS>,
        rgb: Option<RGB>,
        provisioner: bool,
    ) -> Self {
        let uptime = rtc.uptime();
        let status = Status::Startup(uptime);

        #[cfg(not(feature = "no-buttons"))]
        let mut ui = Self {
            rtc,
            buttons: _buttons,
            status,
            rgb,
            provisioner,
        };
        #[cfg(feature = "no-buttons")]
        let mut ui = Self {
            rtc,
            buttons: None,
            status,
            rgb,
            provisioner,
        };

        ui.refresh_ui(uptime);
        ui
    }
    fn refresh_ui(&mut self, uptime: Duration) {
        if let Some(rgb) = &mut self.rgb {
            self.status.refresh(uptime);
            let mode = self.status.led_mode(self.provisioner);
            rgb.set(mode.color(uptime));
        }
    }
}

impl<BUTTONS, RGB> trussed::platform::UserInterface for UserInterface<BUTTONS, RGB>
where
    BUTTONS: Press + Edge,
    RGB: RgbLed,
{
    fn check_user_presence(&mut self) -> consent::Level {
        match &mut self.buttons {
            Some(buttons) => {
                // important to read state before checking for edge,
                // since reading an edge could clear the state.
                let state = buttons.state();
                UserPresenceStatus::set_waiting(true);
                let press_result = buttons.wait_for_any_new_press();
                UserPresenceStatus::set_waiting(false);
                if press_result.is_ok() {
                    if state.a && state.b {
                        consent::Level::Strong
                    } else {
                        consent::Level::Normal
                    }
                } else {
                    consent::Level::None
                }
            }
            None => {
                // With configured with no buttons, that means Solo is operating
                // in passive NFC mode, which means user tapped to indicate presence.
                consent::Level::Normal
            }
        }
    }

    fn set_status(&mut self, status: trussed::platform::ui::Status) {
        let uptime = self.uptime();
        self.status.update(status, uptime);
        self.refresh_ui(uptime);
    }

    fn refresh(&mut self) {
        let uptime = self.uptime();
        self.refresh_ui(uptime);
    }

    fn uptime(&mut self) -> Duration {
        self.rtc.uptime()
    }

    // delete this function after trussed is updated
    fn reboot(&mut self, _to: reboot::To) -> ! {
        panic!("this should no longer be called.");
    }

    fn wink(&mut self, duration: Duration) {
        let uptime = self.uptime();
        self.status = Status::Winking(uptime..uptime + duration);
        self.refresh_ui(uptime);
    }
}
