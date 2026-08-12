//! Capacitive touch buttons for the SoloKeys Solo2.
//!
//! The Solo2 uses three capacitive touch pads driven by the LPC55 ADC together
//! with a DMA channel and two timers, unlike the Nitrokey 3 which uses a single
//! GPIO button (see [`super::button`]).  This is a port of the `solo2` board
//! driver from the upstream `solokeys/solo2` repository, adapted to this repo's
//! [`crate::ui::buttons`] traits.
use core::convert::Infallible;

use lpc55_hal::{
    drivers::{
        pins,
        touch::{ButtonPins, Compare, Edge as TouchEdge, TouchSensor, TouchSensorChannel},
    },
    peripherals::{adc, ctimer, dma},
    typestates::{pin::PinId, ClocksSupportTouchToken},
    Enabled, Gpio, Iocon,
};
use trussed_core::types::consent;

use crate::ui::buttons::{Button, Edge, Press, UserPresence};

pub type ChargeMatchPin = pins::Pio1_16;
pub type ButtonTopPin = pins::Pio0_23;
pub type ButtonBotPin = pins::Pio0_31;
pub type ButtonMidPin = pins::Pio0_15;

type Adc = adc::Adc<Enabled>;
type Dma = dma::Dma<Enabled>;

type AdcTimer = ctimer::Ctimer1<Enabled>;
type SampleTimer = ctimer::Ctimer2<Enabled>;

pub type ThreeButtons = SoloThreeTouchButtons<ButtonTopPin, ButtonBotPin, ButtonMidPin>;

pub struct SoloThreeTouchButtons<P1, P2, P3>
where
    P1: PinId,
    P2: PinId,
    P3: PinId,
{
    touch_sensor: TouchSensor<P1, P2, P3>,
}

impl SoloThreeTouchButtons<ButtonTopPin, ButtonBotPin, ButtonMidPin> {
    pub fn new(
        adc: Adc,
        adc_timer: AdcTimer,
        sample_timer: SampleTimer,
        dma: &mut Dma,
        token: ClocksSupportTouchToken,
        gpio: &mut Gpio<Enabled>,
        iocon: &mut Iocon<Enabled>,
    ) -> SoloThreeTouchButtons<ButtonTopPin, ButtonBotPin, ButtonMidPin> {
        let top = ButtonTopPin::take().unwrap().into_analog_input(iocon, gpio);
        let mid = ButtonMidPin::take().unwrap().into_analog_input(iocon, gpio);
        let bot = ButtonBotPin::take().unwrap().into_analog_input(iocon, gpio);
        let charge_match = ChargeMatchPin::take().unwrap().into_match_output(iocon);
        let button_pins = ButtonPins(top, bot, mid);
        let touch_sensor = TouchSensor::new(
            [12_000, 12_000, 12_000],
            5,
            adc,
            adc_timer,
            sample_timer,
            charge_match,
            button_pins,
        );
        let touch_sensor = touch_sensor.enabled(dma, token);
        Self { touch_sensor }
    }

    /// Map a [`Button`] to its touch channel and read the debounced state.
    fn button_get_state(&self, button: Button, ctype: Compare) -> bool {
        let channel = match button {
            Button::A => TouchSensorChannel::Channel1,
            Button::B => TouchSensorChannel::Channel2,
            Button::Middle => TouchSensorChannel::Channel3,
        };
        self.touch_sensor.get_state(channel, ctype).is_active
    }

    fn button_has_edge(&self, button: Button, edge_type: TouchEdge) -> bool {
        let channel = match button {
            Button::A => TouchSensorChannel::Channel1,
            Button::B => TouchSensorChannel::Channel2,
            Button::Middle => TouchSensorChannel::Channel3,
        };
        self.touch_sensor.has_edge(channel, edge_type)
    }

    fn button_reset_state(&self, button: Button, offset: i32) {
        let channel = match button {
            Button::A => TouchSensorChannel::Channel1,
            Button::B => TouchSensorChannel::Channel2,
            Button::Middle => TouchSensorChannel::Channel3,
        };
        self.touch_sensor.reset_results(channel, offset);
    }
}

impl Press for SoloThreeTouchButtons<ButtonTopPin, ButtonBotPin, ButtonMidPin> {
    fn is_pressed(&mut self, button: Button) -> bool {
        self.button_get_state(button, Compare::BelowThreshold)
    }

    fn is_released(&mut self, button: Button) -> bool {
        self.button_get_state(button, Compare::AboveThreshold)
    }
}

impl Edge for SoloThreeTouchButtons<ButtonTopPin, ButtonBotPin, ButtonMidPin> {
    fn wait_for_new_press(&mut self, button: Button) -> nb::Result<(), Infallible> {
        if self.button_has_edge(button, TouchEdge::Falling) {
            // Erase edge with pressed status.
            self.button_reset_state(button, -1);
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn wait_for_new_release(&mut self, button: Button) -> nb::Result<(), Infallible> {
        if self.button_has_edge(button, TouchEdge::Rising) {
            self.button_reset_state(button, 1);
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn wait_for_any_new_press(&mut self) -> nb::Result<Button, Infallible> {
        if self.wait_for_new_press(Button::A).is_ok() {
            Ok(Button::A)
        } else if self.wait_for_new_press(Button::B).is_ok() {
            Ok(Button::B)
        } else if self.wait_for_new_press(Button::Middle).is_ok() {
            Ok(Button::Middle)
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn wait_for_any_new_release(&mut self) -> nb::Result<Button, Infallible> {
        if self.wait_for_new_release(Button::A).is_ok() {
            Ok(Button::A)
        } else if self.wait_for_new_release(Button::B).is_ok() {
            Ok(Button::B)
        } else if self.wait_for_new_release(Button::Middle).is_ok() {
            Ok(Button::Middle)
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn wait_for_new_squeeze(&mut self) -> nb::Result<(), Infallible> {
        let a = self.button_has_edge(Button::A, TouchEdge::Falling);
        let b = self.button_has_edge(Button::B, TouchEdge::Falling);
        if a && b {
            self.button_reset_state(Button::A, -1);
            self.button_reset_state(Button::B, -1);
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl UserPresence for SoloThreeTouchButtons<ButtonTopPin, ButtonBotPin, ButtonMidPin> {
    fn check_user_presence(&mut self) -> consent::Level {
        let state = self.state();
        if self.wait_for_any_new_press().is_ok() {
            if state.a && state.b {
                consent::Level::Strong
            } else {
                consent::Level::Normal
            }
        } else {
            consent::Level::None
        }
    }
}
