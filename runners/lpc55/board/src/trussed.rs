//! Implementation of `trussed::Platform` for the board,
//! using the specific implementation of our `crate::traits`.

use core::time::Duration;

use crate::hal::{
    peripherals::rtc::Rtc,
    typestates::init_state,
};
use crate::ui::Status;
use crate::traits::buttons::{Press, Edge};
use crate::traits::rgb_led::RgbLed;
use trussed::platform::{self, consent};

// Assuming there will only be one way to
// get user presence, this should be fine.
// Used for Ctaphid.keepalive message status.
static mut WAITING: bool = false;
pub struct UserPresenceStatus {}
impl UserPresenceStatus {
    pub(crate) fn set_waiting(waiting: bool) {
        unsafe { WAITING = waiting };
    }
    pub fn waiting() -> bool {
        unsafe{ WAITING }
    }
}

pub struct UserInterface<BUTTONS, RGB>
where
BUTTONS: Press + Edge,
RGB: RgbLed,
{
    rtc: Rtc<init_state::Enabled>,
    buttons: Option<BUTTONS>,
    status: Status,
    rgb: Option<RGB>,
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
        let status = Default::default();
        #[cfg(not(feature = "no-buttons"))]
        let ui = Self { rtc, buttons: _buttons, status, rgb, provisioner };
        #[cfg(feature = "no-buttons")]
        let ui = Self { rtc, buttons: None, status, rgb, provisioner };

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

impl<BUTTONS, RGB> trussed::platform::UserInterface for UserInterface<BUTTONS,RGB>
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

    fn set_status(&mut self, status: platform::ui::Status) {
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

    fn wink(&mut self, duration: Duration) {
        let uptime = self.uptime();
        self.status = Status::Winking(uptime..uptime + duration);
        self.refresh_ui(uptime);
    }
}
