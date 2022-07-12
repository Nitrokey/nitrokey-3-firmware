use core::{ops::Range, time::Duration};

use trussed::platform::ui;

use crate::traits::rgb_led::Intensities;

const BLACK: Intensities = Intensities { red: 0, green: 0, blue: 0 };
const RED: Intensities = Intensities { red: u8::MAX, green: 0, blue: 0 };
const TEAL: Intensities = Intensities { red: 0, green: u8::MAX, blue: 0x5a };
const WHITE: Intensities = Intensities { red: u8::MAX, green: u8::MAX, blue: u8::MAX };

pub enum Status {
    Startup(Duration),
    Idle,
    Processing,
    WaitingForUserPresence(Duration),
    Winking(Range<Duration>),
    Error,
}

impl Status {
    pub fn update(&mut self, status: ui::Status, uptime: Duration) {
        if status == ui::Status::Idle {
            if matches!(self, Self::Startup(_) | Self::Winking(_)) {
                return;
            }
        }
        *self = (status, uptime).into();
    }

    pub fn refresh(&mut self, uptime: Duration) {
        let end = match self {
            Self::Startup(ref start) => Some(*start + Duration::from_millis(500)),
            Self::Winking(ref range) => Some(range.end),
            _ => None,
        };
        if let Some(end) = end {
            if uptime > end {
                *self = Self::Idle;
            }
        }
    }

    pub fn led_mode(&self, is_provisioner: bool) -> LedMode {
        match self {
            Self::Startup(_) => LedMode::constant(WHITE),
            Self::Idle => if is_provisioner {
                LedMode::constant(WHITE)
            } else {
                LedMode::constant(BLACK)
            },
            Self::Processing => LedMode::constant(TEAL),
            Self::WaitingForUserPresence(start) => LedMode::simple_blinking(WHITE, *start),
            Self::Error => LedMode::constant(RED),
            Self::Winking(range) => LedMode::simple_blinking(WHITE, range.start),
        }
    }
}

impl From<(ui::Status, Duration)> for Status {
    fn from((status, uptime): (ui::Status, Duration)) -> Self {
        match status {
            ui::Status::Idle => Self::Idle,
            ui::Status::Processing => Self::Processing,
            ui::Status::WaitingForUserPresence => Self::WaitingForUserPresence(uptime),
            ui::Status::Error => Self::Error,
        }
    }
}

pub enum LedMode {
    Constant {
        color: Intensities,
    },
    Blinking {
        on_color: Intensities,
        off_color: Intensities,
        period: Duration,
        start: Duration,
    },
}

impl LedMode {
    pub fn constant(color: Intensities) -> Self {
        Self::Constant { color }
    }

    pub fn blinking(
        on_color: Intensities,
        off_color: Intensities,
        period: Duration,
        start: Duration,
    ) -> Self {
        Self::Blinking { on_color, off_color, period, start }
    }

    pub fn simple_blinking(color: Intensities, start: Duration) -> Self {
        Self::blinking(color, BLACK, Duration::from_millis(500), start)
    }

    pub fn color(&self, uptime: Duration) -> Intensities {
        match self {
            Self::Constant { color } => *color,
            Self::Blinking { on_color, off_color, period, start } => {
                let delta = (uptime - *start).as_millis() % period.as_millis();
                let is_on = delta < period.as_millis() / 2;
                if is_on {
                    *on_color
                } else {
                    *off_color
                }
            },
        }
    }
}
