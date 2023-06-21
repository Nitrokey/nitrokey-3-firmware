//! Implementation of `trussed::Platform` for the board,
//! using the specific implementation of our `crate::traits`.

use core::time::Duration;

use crate::traits::{
    buttons::{Button, Press},
    rgb_led::RgbLed,
};
use trussed::platform::{consent, ui};

use crate::runtime::UserPresenceStatus;

use crate::ui::Status;

use embedded_time::duration::*;
use rtic::Monotonic;
type RtcMonotonic = crate::soc::rtic_monotonic::RtcMonotonic;

pub struct UserInterface<BUTTONS, RGB>
where
    BUTTONS: Press,
    RGB: RgbLed,
{
    buttons: Option<BUTTONS>,
    rgb: Option<RGB>,
    status: Status,
    provisioner: bool,
    rtc_mono: RtcMonotonic,
}

impl<BUTTONS, RGB> UserInterface<BUTTONS, RGB>
where
    BUTTONS: Press,
    RGB: RgbLed,
{
    pub fn new(_buttons: Option<BUTTONS>, rgb: Option<RGB>, provisioner: bool) -> Self {
        let pac = unsafe { nrf52840_pac::Peripherals::steal() };
        let mut rtc_mono = RtcMonotonic::new(pac.RTC0);

        //let uptime = rtc_mono.uptime();
        let ms: embedded_time::duration::units::Milliseconds = rtc_mono.now().into();
        let uptime: Duration = core::time::Duration::from_millis(ms.integer().into());

        let status = Status::Startup(uptime);

        #[cfg(not(feature = "no-buttons"))]
        let mut ui = Self {
            buttons: _buttons,
            status,
            rgb,
            provisioner,
            rtc_mono,
        };
        #[cfg(feature = "no-buttons")]
        let mut ui = Self {
            buttons: None,
            status,
            rgb,
            provisioner,
            rtc_mono,
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
    BUTTONS: Press,
    RGB: RgbLed,
{
    fn check_user_presence(&mut self) -> consent::Level {
        // essentially a blocking call for up to ~30secs
        // this outer loop accumulates *presses* from the
        // inner loop & maintains (loading) delays.

        // no buttons configured -> always consent
        if self.buttons.is_none() {
            return consent::Level::Normal;
        }

        let mut counter: u8 = 0;
        let mut is_pressed = false;
        let threshold: u8 = 1;

        let start_time = self.uptime().as_millis();
        let timeout_at = start_time + 1_000u128;
        let mut next_check = start_time + 25u128;

        self.set_status(ui::Status::WaitingForUserPresence);

        loop {
            let cur_time = self.uptime().as_millis();

            // timeout reached
            if cur_time > timeout_at {
                break;
            }
            // loop until next check shall be done
            if cur_time < next_check {
                continue;
            }

            if let Some(button) = self.buttons.as_mut() {
                UserPresenceStatus::set_waiting(true);
                is_pressed = button.is_pressed(Button::A);
                UserPresenceStatus::set_waiting(false);
            }

            if is_pressed {
                counter += 1;
                // with press -> delay 25ms
                next_check = cur_time + 25;
            } else {
                // w/o press -> delay 100ms
                next_check = cur_time + 100;
            }

            if counter >= threshold {
                break;
            }
        }

        // consent, if we've counted 3 "presses"
        if counter >= threshold {
            consent::Level::Normal
        } else {
            consent::Level::None
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
        let ms: embedded_time::duration::units::Milliseconds = self.rtc_mono.now().into();
        core::time::Duration::from_millis(ms.integer().into())
    }

    fn wink(&mut self, duration: Duration) {
        let uptime = self.uptime();
        self.status = Status::Winking(uptime..uptime + duration);
        self.refresh_ui(uptime);
    }
}
