use apdu_dispatch::interchanges::{
    Channel as CcidChannel, Requester as CcidRequester, Responder as CcidResponder,
};
use apps::{Endpoints, InitStatus};
use boards::{
    flash::ExtFlashStorage,
    init::{self, UsbNfc, UsbResources},
    nk3xn::{
        button::ThreeButtons,
        led::RgbLed,
        nfc::{self, NfcChip},
        prince,
        spi::{self, FlashCs, FlashCsPin, Spi, SpiConfig},
        ButtonsTimer, InternalFlashStorage, NK3xN, PwmTimer, I2C,
    },
    soc::{lpc55::Lpc55, Soc},
    store::{self, RunnerStore, StoreResources},
    ui::{
        buttons::{self, Press},
        rgb_led::RgbLed as _,
        UserInterface,
    },
    Apps, Trussed,
};
use embedded_hal::{
    blocking::i2c::{Read, Write},
    timer::{Cancel, CountDown},
};
use hal::{
    drivers::{
        clocks,
        flash::FlashGordon,
        pins::{self, direction},
        Timer,
    },
    peripherals::{
        ctimer::{self, Ctimer},
        flexcomm::{Flexcomm0, Flexcomm5},
        inputmux::InputMux,
        pfr::Pfr,
        pint::Pint,
        prince::Prince,
        rng::Rng,
        usbhs::Usbhs,
        wwdt::{self, Wwdt},
    },
    raw::WWDT,
    time::{DurationExtensions as _, RateExtensions as _},
    traits::wg::digital::v2::InputPin,
    typestates::{
        init_state::{Enabled, Unknown},
        pin::state::Gpio,
    },
    Pin,
};

use interchange::Channel;
use littlefs2_core::path;
use lpc55_hal as hal;
#[cfg(any(feature = "log-info", feature = "log-all"))]
use lpc55_hal::drivers::timer::Elapsed as _;
use nfc_device::Iso14443;
use trussed::types::Location;
use utils::OptionalStorage;
#[cfg(feature = "se050")]
use {boards::nk3xn::TimerDelay, se05x::embedded_hal::Hal027};

use crate::{VERSION, VERSION_STRING};

type UsbBusType = usb_device::bus::UsbBusAllocator<<Lpc55 as Soc>::UsbBus>;

pub type WwdtEnabled = wwdt::Active;
pub type WwdtResetting = wwdt::Active;
pub type WwdtProtecting = wwdt::Inactive;
pub type MaybeEnabledWwdt = Option<Wwdt<WwdtEnabled, WwdtResetting, WwdtProtecting>>;

struct Peripherals {
    syscon: hal::Syscon,
    pmc: hal::Pmc,
    anactrl: hal::Anactrl,
}

struct Clocks {
    is_nfc_passive: bool,
    clocks: clocks::Clocks,
    nfc_irq: Option<Pin<nfc::NfcIrqPin, Gpio<direction::Input>>>,
    iocon: hal::Iocon<Enabled>,
    gpio: hal::Gpio<Enabled>,
}

pub struct Basic {
    pub delay_timer: Timer<ctimer::Ctimer0<Enabled>>,
    pub perf_timer: Option<Timer<ctimer::Ctimer4<Enabled>>>,
    three_buttons: Option<ThreeButtons>,
    rgb: Option<RgbLed>,
    old_firmware_version: u32,
}

struct Flash {
    flash_gordon: FlashGordon,
    #[allow(unused)]
    prince: Prince<Enabled>,
    rng: Rng<Enabled>,
}

pub struct Stage0 {
    status: InitStatus,
    peripherals: Peripherals,
}

impl Stage0 {
    fn enable_low_speed_for_passive_nfc(
        &mut self,
        mut iocon: hal::Iocon<Enabled>,
        gpio: &mut hal::Gpio<Enabled>,
    ) -> (
        hal::Iocon<Enabled>,
        Pin<nfc::NfcIrqPin, Gpio<direction::Input>>,
        bool,
    ) {
        let nfc_irq = nfc::NfcIrqPin::take()
            .unwrap()
            .into_gpio_pin(&mut iocon, gpio)
            .into_input();
        // Need to enable pullup for NFC IRQ input.
        let iocon = iocon.release();
        iocon.pio0_19.modify(|_, w| w.mode().pull_up());
        let iocon = hal::Iocon::from(iocon).enabled(&mut self.peripherals.syscon);
        let is_passive_mode = nfc_irq.is_low().ok().unwrap();
        debug!("IS PASSIVE MODE: {is_passive_mode}");

        (iocon, nfc_irq, is_passive_mode)
    }

    fn enable_clocks(&mut self, is_nfc_passive: bool) -> clocks::Clocks {
        // Start out with slow clock if in passive mode;
        let frequency = if is_nfc_passive { 48.MHz() } else { 96.MHz() };
        hal::ClockRequirements::default()
            .system_frequency(frequency)
            .configure(
                &mut self.peripherals.anactrl,
                &mut self.peripherals.pmc,
                &mut self.peripherals.syscon,
            )
            .expect("Clock configuration failed")
    }

    #[inline(never)]
    pub fn next(
        mut self,
        iocon: hal::Iocon<Unknown>,
        gpio: hal::Gpio<Unknown>,
        wwdt: WWDT,
        ctimer1: ctimer::Ctimer1,
    ) -> Stage1 {
        let mut iocon = iocon.enabled(&mut self.peripherals.syscon);
        let mut gpio = gpio.enabled(&mut self.peripherals.syscon);

        let (new_iocon, nfc_irq, is_nfc_passive) =
            self.enable_low_speed_for_passive_nfc(iocon, &mut gpio);
        let is_nfc_passive = true;
        iocon = new_iocon;
        let nfc_irq = Some(nfc_irq);

        let wwdt = (!is_nfc_passive).then(|| {
            let mut wwdt = Wwdt::try_new(wwdt, &self.peripherals.syscon, 63).unwrap();
            // Frequency is 1/(4*64) MHz, there is a built-in 4x multiplier
            const TIMER_COUNT: u32 =
                (1_000_000 / (4 * 64) * boards::WATCHDOG_DURATION_SECONDS) as u32;
            wwdt.set_timer(TIMER_COUNT).unwrap();
            wwdt.set_warning(0b1_1111_1111).unwrap();
            let wwdt = wwdt.set_resetting().set_enabled();
            debug_now!("Wwdt tv: {:?}", wwdt.timer());
            wwdt
        });

        let clocks_inner = self.enable_clocks(is_nfc_passive);

        // measurement stuff
        let token = clocks_inner.support_1mhz_fro_token().unwrap();
        let mut boot_timer = Timer::new(ctimer1.enabled(&mut self.peripherals.syscon, token));
        boot_timer.start(u32::MAX.microseconds());
        boards::soc::lpc55::boot_timer::install(boot_timer);
        // Capture two independent timestamps per transaction: when the first
        // APDU arrives from the chip, and when the first response goes out
        // on the wire. The NDEF URL exposes both as `?r=<rx>&t=<tx>` so the
        // delta (firmware processing) and the absolute rx (external chip +
        // reader handshake) can be observed in a single read.
        nfc_device::iso14443::install_post_request_receive_hook(boards::measurement::record_rx_us);
        nfc_device::iso14443::install_pre_response_send_hook(boards::measurement::record_tx_us);
        ndef_app::install_url_rx_reader(boards::measurement::rx_first_us);
        ndef_app::install_url_tx_reader(boards::measurement::tx_first_us);
        ndef_app::install_url_rx_count_reader(boards::measurement::rx_count);
        ndef_app::install_url_tx_count_reader(boards::measurement::tx_count);
        ndef_app::install_url_irq_count_reader(boards::measurement::irq_count);
        ndef_app::install_url_irq_first_reader(boards::measurement::irq_first_us);

        let clocks = Clocks {
            is_nfc_passive,
            clocks: clocks_inner,
            nfc_irq,
            iocon,
            gpio,
        };

        Stage1 {
            status: self.status,
            peripherals: self.peripherals,
            clocks,
            wwdt,
        }
    }
}

pub struct Stage1 {
    status: InitStatus,
    peripherals: Peripherals,
    clocks: Clocks,
    wwdt: MaybeEnabledWwdt,
}

impl Stage1 {
    fn validate_cfpa(
        pfr: &mut Pfr<Enabled>,
        current_version_maybe: Option<u32>,
        require_prince: bool,
    ) -> u32 {
        let mut cfpa = pfr.read_latest_cfpa().unwrap();
        let old_version = cfpa.secure_fw_version;
        if let Some(current_version) = current_version_maybe {
            if cfpa.secure_fw_version < current_version || cfpa.ns_fw_version < current_version {
                info!(
                    "updating cfpa from {} to {}",
                    cfpa.secure_fw_version, current_version
                );

                // All of these are monotonic counters.
                cfpa.version += 1;
                cfpa.secure_fw_version = current_version;
                cfpa.ns_fw_version = current_version;
                pfr.write_cfpa(&cfpa).unwrap();
            } else {
                info!(
                    "do not need to update cfpa version {}",
                    cfpa.secure_fw_version
                );
            }
        }

        if require_prince {
            #[cfg(not(feature = "no-encrypted-storage"))]
            assert!(cfpa.key_provisioned(hal::peripherals::pfr::KeyType::PrinceRegion2));
        }

        old_version
    }

    fn is_bootrom_requested<T: Ctimer<Enabled>>(
        &mut self,
        three_buttons: &mut ThreeButtons,
        timer: &mut Timer<T>,
    ) -> bool {
        // Boot to bootrom if buttons are all held for 5s
        timer.start(5_000_000.microseconds());
        while three_buttons.is_pressed(buttons::Button::A)
            && three_buttons.is_pressed(buttons::Button::B)
            && three_buttons.is_pressed(buttons::Button::Middle)
        {
            // info!("3 buttons pressed..");
            if timer.wait().is_ok() {
                return true;
            }
        }
        timer.cancel().ok();

        false
    }

    fn init_rgb(&mut self, ctimer: PwmTimer) -> RgbLed {
        #[cfg(feature = "board-nk3xn")]
        {
            RgbLed::new(
                hal::drivers::Pwm::new(ctimer.enabled(
                    &mut self.peripherals.syscon,
                    self.clocks.clocks.support_1mhz_fro_token().unwrap(),
                )),
                &mut self.clocks.iocon,
            )
        }
    }

    fn init_buttons(&mut self, ctimer: ButtonsTimer) -> ThreeButtons {
        #[cfg(feature = "board-nk3xn")]
        {
            ThreeButtons::new(
                Timer::new(ctimer.enabled(
                    &mut self.peripherals.syscon,
                    self.clocks.clocks.support_1mhz_fro_token().unwrap(),
                )),
                &mut self.clocks.gpio,
                &mut self.clocks.iocon,
            )
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[inline(never)]
    pub fn next(
        mut self,
        delay_timer: ctimer::Ctimer0,
        ctimer2: ctimer::Ctimer2,
        ctimer3: ctimer::Ctimer3,
        perf_timer: ctimer::Ctimer4,
        pfr: Pfr<Unknown>,
        secure_firmware_version: Option<u32>,
        require_prince: bool,
        boot_to_bootrom: bool,
    ) -> Stage2 {
        let syscon = &mut self.peripherals.syscon;

        let mut delay_timer = Timer::new(
            delay_timer.enabled(syscon, self.clocks.clocks.support_1mhz_fro_token().unwrap()),
        );
        let se050_timer = Timer::new(
            ctimer2.enabled(syscon, self.clocks.clocks.support_1mhz_fro_token().unwrap()),
        );
        let perf_timer = Timer::new(
            perf_timer.enabled(syscon, self.clocks.clocks.support_1mhz_fro_token().unwrap()),
        );
        // perf_timer.start(60_000_000.microseconds());

        let mut rgb = self.init_rgb(ctimer3);

        // CTimer1 is now owned by the boot/measurement timer (set up in
        // stage 0); buttons would need a different ctimer to come back.
        let mut three_buttons: Option<ThreeButtons> = None;

        let mut pfr = pfr.enabled(&self.clocks.clocks).unwrap();
        let old_firmware_version =
            Self::validate_cfpa(&mut pfr, secure_firmware_version, require_prince);

        if boot_to_bootrom && three_buttons.is_some() {
            // info!("bootrom request start {}", perf_timer.elapsed().0 / 1000);
            if self.is_bootrom_requested(three_buttons.as_mut().unwrap(), &mut delay_timer) {
                // Give a small red blink show success
                rgb.red(200);
                rgb.green(200);
                rgb.blue(0);
                delay_timer.start(100_000.microseconds());
                nb::block!(delay_timer.wait()).ok();

                hal::boot_to_bootrom()
            }
        }

        let basic = Basic {
            delay_timer,
            perf_timer: Some(perf_timer),
            three_buttons,
            rgb: Some(rgb),
            old_firmware_version,
        };
        Stage2 {
            status: self.status,
            peripherals: self.peripherals,
            clocks: self.clocks,
            se050_timer,
            basic,
            wwdt: self.wwdt,
        }
    }
}

pub struct Stage2 {
    status: InitStatus,
    peripherals: Peripherals,
    clocks: Clocks,
    basic: Basic,
    se050_timer: Timer<ctimer::Ctimer2<Enabled>>,
    wwdt: MaybeEnabledWwdt,
}

impl Stage2 {
    fn setup_spi(&mut self, flexcomm0: Flexcomm0<Unknown>, config: SpiConfig) -> Spi {
        let token = self.clocks.clocks.support_flexcomm_token().unwrap();
        let spi = flexcomm0.enabled_as_spi(&mut self.peripherals.syscon, &token);
        spi::init(spi, &mut self.clocks.iocon, config)
    }

    fn setup_fm11nc08(
        &mut self,
        spi: Spi,
        inputmux: InputMux<Unknown>,
        pint: Pint<Unknown>,
        nfc_rq: CcidRequester<'static>,
    ) -> Option<Iso14443<NfcChip>> {
        // TODO save these so they can be released later
        let mut mux = inputmux.enabled(&mut self.peripherals.syscon);
        let mut pint = pint.enabled(&mut self.peripherals.syscon);
        let nfc_irq = self.clocks.nfc_irq.take().unwrap();
        pint.enable_interrupt(
            &mut mux,
            &nfc_irq,
            lpc55_hal::peripherals::pint::Slot::Slot0,
            lpc55_hal::peripherals::pint::Mode::ActiveLow,
        );
        mux.disabled(&mut self.peripherals.syscon);

        let nfc = nfc::try_setup_old_chip(
            spi,
            &mut self.clocks.gpio,
            &mut self.clocks.iocon,
            nfc_irq,
            &mut self.basic.delay_timer,
            &mut self.status,
        )?;

        let mut iso14443 = Iso14443::new(nfc_device::either::Either::A(nfc), nfc_rq);
        //iso14443.poll();
        // Give a small delay to charge up capacitors
        // basic_stage.delay_timer.start(5_000.microseconds()); nb::block!(basic_stage.delay_timer.wait()).ok();
        Some(iso14443)
    }

    fn setup_fm11nt08c(
        &mut self,
        i2c: I2C,
        inputmux: InputMux<Unknown>,
        pint: Pint<Unknown>,
        nfc_rq: CcidRequester<'static>,
    ) -> Option<Iso14443<NfcChip>> {
        // TODO save these so they can be released later
        let mut mux = inputmux.enabled(&mut self.peripherals.syscon);
        let mut pint = pint.enabled(&mut self.peripherals.syscon);
        let nfc_irq = self.clocks.nfc_irq.take().unwrap();
        pint.enable_interrupt(
            &mut mux,
            &nfc_irq,
            lpc55_hal::peripherals::pint::Slot::Slot0,
            lpc55_hal::peripherals::pint::Mode::ActiveLow,
        );
        mux.disabled(&mut self.peripherals.syscon);

        // let nfc = nfc::try_setup(
        //     spi,
        //     &mut self.clocks.gpio,
        //     &mut self.clocks.iocon,
        //     nfc_irq,
        //     &mut self.basic.delay_timer,
        //     &mut self.status,
        // )?;

        let mut nfc = nfc::try_setup_new(
            i2c,
            &mut self.clocks.gpio,
            &mut self.clocks.iocon,
            nfc_irq,
            self.basic.perf_timer.take().unwrap(),
        );

        // Only run EEPROM configuration on USB power; energy-harvested boots
        // must never write the chip's NV memory.
        nfc.init(!self.clocks.is_nfc_passive).ok();

        let mut iso14443 = Iso14443::new(nfc_device::either::Either::B(nfc), nfc_rq);
        #[cfg(not(feature = "no-delog"))]
        boards::init::Delogger::flush();
        //iso14443.poll();
        Some(iso14443)
    }

    fn get_se050_i2c(&mut self, flexcomm5: Flexcomm5<Unknown>) -> I2C {
        // SE050 check
        // let _enabled = pins::Pio1_26::take()
        //     .unwrap()
        //     .into_gpio_pin(&mut self.clocks.iocon, &mut self.clocks.gpio)
        //     .into_output_high();

        // self.basic.delay_timer.start(100_000.microseconds());
        // nb::block!(self.basic.delay_timer.wait()).ok();

        let token = self.clocks.clocks.support_flexcomm_token().unwrap();
        let i2c = flexcomm5.enabled_as_i2c(&mut self.peripherals.syscon, &token);
        let scl = pins::Pio0_9::take()
            .unwrap()
            .into_i2c5_scl_pin(&mut self.clocks.iocon);
        let sda = pins::Pio1_14::take()
            .unwrap()
            .into_i2c5_sda_pin(&mut self.clocks.iocon);
        let mut i2c = hal::I2cMaster::new(
            i2c,
            (scl, sda),
            hal::time::Hertz::try_from(1000_u32.kHz()).unwrap(),
        );

        // self.basic.delay_timer.start(100_000.microseconds());
        // nb::block!(self.basic.delay_timer.wait()).ok();

        // RESYNC command
        // let command = [0x5a, 0xc0, 0x00, 0xff, 0xfc];
        // i2c.write(0x48, &command)
        //     .expect("failed to send RESYNC command");

        // self.basic.delay_timer.start(100_000.microseconds());
        // nb::block!(self.basic.delay_timer.wait()).ok();

        // // RESYNC response
        // let mut response = [0; 5];
        // i2c.read(0x48, &mut response)
        //     .expect("failed to read RESYNC response");

        // if response != [0xa5, 0xe0, 0x00, 0x3F, 0x19] {
        //     panic!("Unexpected RESYNC response: {:?}", response);
        // }

        // debug_now!("Communication with SE050 worked");

        // let command = [0x00, 0x00];
        // i2c.write(0x57, &command)
        //     .expect("failed to send RESYNC command");

        // // RESYNC response
        // let mut response = [0; 4];
        // i2c.read(0x57, &mut response)
        //     .expect("failed to read RESYNC response");

        info_now!("hardware checks successful");
        i2c
    }

    /// Reduce power draw by disabling everything not used over NFC
    fn reduce_power_draw(mut self) -> Self {
        let iocon = self.clocks.iocon.release();
        // Put all unused pins in pulldown so that they're not drawing power by floating
        iocon.pio0_0.modify(|_, w| w.mode().pull_down());
        iocon.pio0_1.modify(|_, w| w.mode().pull_down());
        iocon.pio0_2.modify(|_, w| w.mode().pull_down());
        iocon.pio0_3.modify(|_, w| w.mode().pull_down());
        iocon.pio0_4.modify(|_, w| w.mode().pull_down());
        iocon.pio0_6.modify(|_, w| w.mode().pull_down());
        iocon.pio0_7.modify(|_, w| w.mode().pull_down());
        iocon.pio0_8.modify(|_, w| w.mode().pull_down());
        iocon.pio0_10.modify(|_, w| w.mode().pull_down());
        iocon.pio0_11.modify(|_, w| w.mode().pull_down());
        iocon.pio0_12.modify(|_, w| w.mode().pull_down());
        // iocon.pio0_13.modify(|_, w| w.mode().pull_down());
        iocon.pio0_14.modify(|_, w| w.mode().pull_down());
        iocon.pio0_15.modify(|_, w| w.mode().pull_down());
        iocon.pio0_16.modify(|_, w| w.mode().pull_down());
        iocon.pio0_17.modify(|_, w| w.mode().pull_down());
        iocon.pio0_18.modify(|_, w| w.mode().pull_down());
        // iocon.pio0_19.modify(|_, w| w.mode().pull_down());
        iocon.pio0_20.modify(|_, w| w.mode().pull_down());
        iocon.pio0_21.modify(|_, w| w.mode().pull_down()); // ext. flash power

        // iocon.pio0_22.modify(|_, w| w.mode().pull_down());
        iocon.pio0_23.modify(|_, w| w.mode().pull_down());
        // iocon.pio0_24.modify(|_, w| w.mode().pull_down());
        // iocon.pio0_25.modify(|_, w| w.mode().pull_down());
        iocon.pio0_26.modify(|_, w| w.mode().pull_down());
        iocon.pio0_27.modify(|_, w| w.mode().pull_down());
        // iocon.pio0_28.modify(|_, w| w.mode().pull_down());
        iocon.pio0_29.modify(|_, w| w.mode().pull_down());
        iocon.pio0_30.modify(|_, w| w.mode().pull_down());
        iocon.pio0_31.modify(|_, w| w.mode().pull_down());
        iocon.pio1_0.modify(|_, w| w.mode().pull_down());
        iocon.pio1_1.modify(|_, w| w.mode().pull_down());
        iocon.pio1_2.modify(|_, w| w.mode().pull_down());
        iocon.pio1_3.modify(|_, w| w.mode().pull_down());
        iocon.pio1_4.modify(|_, w| w.mode().pull_down());
        iocon.pio1_5.modify(|_, w| w.mode().pull_down());
        iocon.pio1_6.modify(|_, w| w.mode().pull_down());
        iocon.pio1_7.modify(|_, w| w.mode().pull_down());
        iocon.pio1_8.modify(|_, w| w.mode().pull_down());
        iocon.pio1_9.modify(|_, w| w.mode().pull_down());
        iocon.pio1_10.modify(|_, w| w.mode().pull_down());
        iocon.pio1_11.modify(|_, w| w.mode().pull_down());
        iocon.pio1_12.modify(|_, w| w.mode().pull_down());
        iocon.pio1_13.modify(|_, w| w.mode().pull_down());
        // iocon.pio1_14.modify(|_, w| w.mode().pull_down());
        iocon.pio1_15.modify(|_, w| w.mode().pull_down());
        iocon.pio1_16.modify(|_, w| w.mode().pull_down());
        iocon.pio1_17.modify(|_, w| w.mode().pull_down());
        // iocon.pio1_18.modify(|_, w| w.mode().pull_down());
        // iocon.pio1_19.modify(|_, w| w.mode().pull_down());
        // iocon.pio1_20.modify(|_, w| w.mode().pull_down());
        // iocon.pio1_21.modify(|_, w| w.mode().pull_down());
        iocon.pio1_22.modify(|_, w| w.mode().pull_down());
        iocon.pio1_23.modify(|_, w| w.mode().pull_down());
        iocon.pio1_24.modify(|_, w| w.mode().pull_down());
        iocon.pio1_25.modify(|_, w| w.mode().pull_down());
        iocon.pio1_26.modify(|_, w| w.mode().pull_down()); //  se050 enable
        iocon.pio1_27.modify(|_, w| w.mode().pull_down());
        iocon.pio1_28.modify(|_, w| w.mode().pull_down());
        iocon.pio1_29.modify(|_, w| w.mode().pull_down());
        iocon.pio1_30.modify(|_, w| w.mode().pull_down());
        iocon.pio1_31.modify(|_, w| w.mode().pull_down());
        self.clocks.iocon = hal::Iocon::from(iocon).enabled(&mut self.peripherals.syscon);

        // Gate off unused peripheral clocks
        let syscon = self.peripherals.syscon.release();
        syscon.ahbclkctrl0.modify(|_, w| {
            w.wwdt()
                .disable() // watchdog not used in NFC
                .rtc()
                .disable() // RTC not used
                .crcgen()
                .disable() // CRC engine not used
                .dma0()
                .disable() // DMA not used
                // .pint()
                // .disable() // pin interrupts not used (RTIC uses SW-triggered vectors)
                .gint()
                .disable() // group interrupt not used
                .mailbox()
                .disable() // dual-core mailbox not used
                .adc()
                .disable() // ADC not used (dynamic clock controller removed)
                .gpio2()
                .disable() // GPIO2/3 ports not used
                .gpio3()
                .disable()
            // .iocon()
            // .disable() // iocon is disabled after everything pin mux is fixed after init; no further IOCON access
        });
        syscon.ahbclkctrl1.modify(|_, w| {
            w.mrt()
                .disable() // multi-rate timer not used
                .ostimer()
                .disable() // OS event timer not used
                .sct()
                .disable() // SCTimer not used
                .utick()
                .disable() // micro-tick timer not used
                .usb0_dev()
                .disable() // USB not used
                .fc0()
                .disable() // FLEXCOMM0..4, 6..7 not used (only FC5/I2C5 is)
                .fc1()
                .disable()
                .fc2()
                .disable()
                .fc3()
                .disable()
                .fc4()
                .disable()
                .fc6()
                .disable()
                .fc7()
                .disable()
        });
        syscon.ahbclkctrl2.modify(|_, w| {
            w.dma1()
                .disable()
                .comp()
                .disable()
                .sdio()
                .disable()
                .usb1_host()
                .disable()
                .usb1_dev()
                .disable()
                .usb1_ram()
                .disable()
                .usb1_phy()
                .disable()
                .usb0_hostm()
                .disable()
                .usb0_hosts()
                .disable()
                .hash_aes()
                .disable() // AES/SHA not used
                .pq()
                .disable() // math coprocessor not used
                .plulut()
                .disable() // PLU not used
                .casper()
                .disable() // crypto accelerator not used
                .puf()
                .disable() // PUF not used
                // .rng()
                // .disable() // RNG is used
                .sysctl()
                .disable() // secure sysctl not used
                .hs_lspi()
                .disable() // HS-SPI not used
                .gpio_sec()
                .disable() // secure GPIO not used
                .gpio_sec_int()
                .disable()
                .freqme()
                .disable() // frequency measure not used
        });

        // enable autoclockgating
        syscon.autoclkgateoverride.write(|w| {
            w.enableupdate()
                .enable()
                .rom()
                .disable()
                .ramx_ctrl()
                .disable()
                .ram0_ctrl()
                .disable()
                .ram1_ctrl()
                .disable()
                .ram2_ctrl()
                .disable()
                .ram3_ctrl()
                .disable()
                .ram4_ctrl()
                .disable()
                .sdma0()
                .disable()
                .sdma1()
                .disable()
                .sync0_apb()
                .disable()
                .sync1_apb()
                .disable()
                .syscon()
                .disable()
                .usb0()
                .disable()
                .crcgen()
                .disable()
        });

        self.peripherals.syscon = hal::Syscon::from(syscon);

        self
    }

    #[inline(never)]
    pub fn next(
        mut self,
        flexcomm0: Flexcomm0<Unknown>,
        flexcomm5: Flexcomm5<Unknown>,
        mux: InputMux<Unknown>,
        pint: Pint<Unknown>,
        nfc_enabled: bool,
    ) -> Stage3 {
        static NFC_CHANNEL: CcidChannel = Channel::new();
        let (mut nfc_rq, nfc_rp) = NFC_CHANNEL.split().unwrap();
        *nfc_rq.callback_mut() = || rtic::pend(lpc55_hal::raw::Interrupt::PIN_INT6);

        let se050_i2c = self.get_se050_i2c(flexcomm5);

        let use_nfc =
            (nfc_enabled && (cfg!(feature = "provisioner") || self.clocks.is_nfc_passive)) || true;
        let (se050_i2c, nfc, spi) = if use_nfc {
            // TODO: Add hardware based approach to detect which chip is there
            let using_old_nfc = false;
            let nfc = if using_old_nfc {
                let spi = self.setup_spi(flexcomm0, SpiConfig::Nfc);
                self.setup_fm11nc08(spi, mux, pint, nfc_rq)
            } else {
                self.setup_fm11nt08c(se050_i2c, mux, pint, nfc_rq)
            };
            //self = self.reduce_power_draw();
            (None, nfc, None)
        } else {
            let spi = self.setup_spi(flexcomm0, SpiConfig::ExternalFlash);
            (Some(se050_i2c), None, Some(spi))
        };

        //cortex_m::asm::delay(4_000_000 * 30);

        Stage3 {
            status: self.status,
            peripherals: self.peripherals,
            clocks: self.clocks,
            basic: self.basic,
            nfc,
            nfc_rp,
            spi,
            se050_timer: self.se050_timer,
            se050_i2c,
            wwdt: self.wwdt,
        }
    }
}

pub struct Stage3 {
    status: InitStatus,
    peripherals: Peripherals,
    clocks: Clocks,
    basic: Basic,
    nfc: Option<Iso14443<NfcChip>>,
    nfc_rp: CcidResponder<'static>,
    spi: Option<Spi>,
    se050_timer: Timer<ctimer::Ctimer2<Enabled>>,
    se050_i2c: Option<I2C>,
    wwdt: MaybeEnabledWwdt,
}

impl Stage3 {
    #[inline(never)]
    pub fn next(
        mut self,
        rng: Rng<Unknown>,
        prince: Prince<Unknown>,
        flash: hal::peripherals::flash::Flash<Unknown>,
    ) -> Stage4 {
        info_now!("making flash");
        let syscon = &mut self.peripherals.syscon;

        #[allow(unused_mut)]
        let mut rng = rng.enabled(syscon);

        let mut prince = prince.enabled(&rng);
        prince::disable(&mut prince);

        let flash_gordon = FlashGordon::new(flash.enabled(syscon));

        let flash = Flash {
            flash_gordon,
            prince,
            rng,
        };
        Stage4 {
            status: self.status,
            peripherals: self.peripherals,
            clocks: self.clocks,
            basic: self.basic,
            nfc: self.nfc,
            nfc_rp: self.nfc_rp,
            spi: self.spi,
            se050_timer: self.se050_timer,
            se050_i2c: self.se050_i2c,
            flash,
            wwdt: self.wwdt,
        }
    }
}

pub struct Stage4 {
    status: InitStatus,
    peripherals: Peripherals,
    clocks: Clocks,
    basic: Basic,
    nfc: Option<Iso14443<NfcChip>>,
    nfc_rp: CcidResponder<'static>,
    spi: Option<Spi>,
    flash: Flash,
    se050_timer: Timer<ctimer::Ctimer2<Enabled>>,
    se050_i2c: Option<I2C>,
    wwdt: MaybeEnabledWwdt,
}

impl Stage4 {
    fn setup_external_flash(&mut self, spi: Spi) -> OptionalStorage<ExtFlashStorage<Spi, FlashCs>> {
        let flash_cs = FlashCsPin::take()
            .unwrap()
            .into_gpio_pin(&mut self.clocks.iocon, &mut self.clocks.gpio)
            .into_output_high();
        let _power = pins::Pio0_21::take()
            .unwrap()
            .into_gpio_pin(&mut self.clocks.iocon, &mut self.clocks.gpio)
            .into_output_high();

        self.basic.delay_timer.start(200_000.microseconds());
        nb::block!(self.basic.delay_timer.wait()).ok();

        if let Some(storage) = ExtFlashStorage::try_new(spi, flash_cs) {
            storage.into()
        } else {
            self.status.insert(InitStatus::EXTERNAL_FLASH_ERROR);
            info!("failed to initialize external flash, using fallback");
            OptionalStorage::default()
        }
    }

    #[inline(never)]
    pub fn next(mut self, resources: &'static mut StoreResources<NK3xN>) -> Stage5 {
        info_now!("making fs");

        let external = if let Some(spi) = self.spi.take() {
            info_now!("using external flash");
            self.setup_external_flash(spi)
        } else {
            info_now!("simulating external flash with RAM");
            OptionalStorage::default()
        };

        #[cfg(not(feature = "no-encrypted-storage"))]
        let internal = {
            #[cfg(feature = "write-undefined-flash")]
            initialize_fs_flash(&mut self.flash.flash_gordon, &mut self.flash.prince);

            InternalFlashStorage::new(self.flash.flash_gordon, self.flash.prince)
        };

        #[cfg(feature = "no-encrypted-storage")]
        let internal = InternalFlashStorage::new(self.flash.flash_gordon);

        // temporarily increase clock for the storage mounting or else it takes a long time.
        if self.clocks.is_nfc_passive {
            self.clocks.clocks = unsafe {
                hal::ClockRequirements::default()
                    .system_frequency(48.MHz())
                    .reconfigure(
                        self.clocks.clocks,
                        &mut self.peripherals.pmc,
                        &mut self.peripherals.syscon,
                    )
            };
        }

        // info_now!(
        //     "mount start {} ms",
        //     self.basic.perf_timer.elapsed().0 / 1000
        // );
        //if let Some(iso14443) = &mut self.nfc {
        //    iso14443.poll();
        //}
        let simulated_efs = external.is_ram();
        let store = store::init_store(
            resources,
            internal,
            external,
            simulated_efs,
            &mut self.status,
        );
        // info!("mount end {} ms", self.basic.perf_timer.elapsed().0 / 1000);

        // return to slow freq
        if self.clocks.is_nfc_passive {
            self.clocks.clocks = unsafe {
                hal::ClockRequirements::default()
                    .system_frequency(48.MHz())
                    .reconfigure(
                        self.clocks.clocks,
                        &mut self.peripherals.pmc,
                        &mut self.peripherals.syscon,
                    )
            };
        }

        //if let Some(iso14443) = &mut self.nfc {
        //    iso14443.poll();
        //}

        Stage5 {
            status: self.status,
            peripherals: self.peripherals,
            clocks: self.clocks,
            basic: self.basic,
            rng: self.flash.rng,
            nfc: self.nfc,
            nfc_rp: self.nfc_rp,
            se050_timer: self.se050_timer,
            se050_i2c: self.se050_i2c,
            store,
            wwdt: self.wwdt,
        }
    }
}

#[cfg(feature = "write-undefined-flash")]
/// This is necessary if prince encryption is enabled for the first time
/// after it was first provisioned.  In this case, there can be an exception
/// reading from undefined flash.  To fix, we run a pass over all filesystem
/// flash and set it to a defined value.
fn initialize_fs_flash(flash_gordon: &mut FlashGordon, prince: &mut Prince<Enabled>) {
    use boards::nk3xn::MEMORY_REGIONS;
    use lpc55_hal::traits::flash::{Read, WriteErase};

    let offset = MEMORY_REGIONS.filesystem.start;

    let page_count = ((631 * 1024 + 512) - offset) / 512;

    let mut page_data = [0u8; 512];
    for page in 0..page_count {
        // With prince turned off, this should read as encrypted bytes.
        flash_gordon.read(offset + page * 512, &mut page_data);

        // But if it's zero, then that means the data is undefined and it doesn't bother.
        if page_data == [0u8; 512] {
            info_now!("resetting page {}", page);
            // So we should write nonzero data to initialize flash.
            // We write it as encrypted, so it is in a known state when decrypted by the filesystem layer.
            page_data[0] = 1;
            flash_gordon.erase_page(offset / 512 + page).ok();
            prince.write_encrypted(|prince| {
                prince.enable_region_2_for(|| {
                    flash_gordon.write(offset + page * 512, &page_data).unwrap();
                })
            });
        }
    }
}

pub struct Stage5 {
    status: InitStatus,
    peripherals: Peripherals,
    clocks: Clocks,
    basic: Basic,
    nfc: Option<Iso14443<NfcChip>>,
    nfc_rp: CcidResponder<'static>,
    rng: Rng<Enabled>,
    store: RunnerStore<NK3xN>,
    se050_timer: Timer<ctimer::Ctimer2<Enabled>>,
    se050_i2c: Option<I2C>,
    wwdt: MaybeEnabledWwdt,
}

impl Stage5 {
    #[inline(never)]
    pub fn next(mut self, rtc: hal::peripherals::rtc::Rtc<Unknown>) -> Stage6 {
        let syscon = &mut self.peripherals.syscon;
        let pmc = &mut self.peripherals.pmc;
        let clocks = self.clocks.clocks;

        let mut rtc = rtc.enabled(syscon, clocks.enable_32k_fro(pmc));
        rtc.reset();

        let rgb = if self.clocks.is_nfc_passive {
            None
        } else {
            self.basic.rgb.take()
        };

        let three_buttons = self.basic.three_buttons.take();

        let user_interface = UserInterface::new(rtc, three_buttons, rgb);

        let trussed = init::init_trussed(
            &mut self.rng,
            self.store,
            user_interface,
            &mut self.status,
            None,
            #[cfg(feature = "se050")]
            self.se050_i2c
                .map(|i2c| (Hal027(i2c), Hal027(TimerDelay(self.se050_timer)))),
        );

        #[cfg(not(feature = "se050"))]
        {
            let _ = self.se050_timer;
            let _ = self.se050_i2c;
        }

        //if let Some(iso14443) = &mut self.nfc {
        //    iso14443.poll();
        //}

        Stage6 {
            status: self.status,
            peripherals: self.peripherals,
            clocks: self.clocks,
            basic: self.basic,
            nfc: self.nfc,
            nfc_rp: self.nfc_rp,
            store: self.store,
            trussed,
            wwdt: self.wwdt,
        }
    }
}

pub struct Stage6 {
    status: InitStatus,
    peripherals: Peripherals,
    clocks: Clocks,
    basic: Basic,
    nfc: Option<Iso14443<NfcChip>>,
    nfc_rp: CcidResponder<'static>,
    store: RunnerStore<NK3xN>,
    trussed: Trussed<NK3xN>,
    wwdt: MaybeEnabledWwdt,
}

impl Stage6 {
    fn perform_data_migrations(&mut self) {
        // FIDO2 attestation cert (<= 1.0.2)
        if self.basic.old_firmware_version <= 4194306 {
            debug!("data migration: updating FIDO2 attestation cert");
            let res = trussed::store::store(
                &self.store,
                Location::Internal,
                path!("fido/x5c/00"),
                include_bytes!("../../data/fido-cert.der"),
            );
            if res.is_err() {
                self.status.insert(InitStatus::MIGRATION_ERROR);
                error!("failed to replace attestation cert");
            }
        }
    }

    fn setup_usb_bus(&mut self, usbp: Usbhs) -> UsbBusType {
        let vbus_pin = pins::Pio0_22::take()
            .unwrap()
            .into_usb0_vbus_pin(&mut self.clocks.iocon);

        let mut usb = usbp.enabled_as_device(
            &mut self.peripherals.anactrl,
            &mut self.peripherals.pmc,
            &mut self.peripherals.syscon,
            &mut self.basic.delay_timer,
            self.clocks.clocks.support_usbhs_token().unwrap(),
        );
        // TODO: do we need this one?
        usb.disable_high_speed();

        lpc55_hal::drivers::UsbBus::new(usb, vbus_pin)
    }

    #[inline(never)]
    pub fn next(
        mut self,
        resources: &'static mut UsbResources<NK3xN>,
        usbhs: Usbhs<Unknown>,
    ) -> All {
        self.perform_data_migrations();
        let (apps, endpoints) = init::init_apps(
            &Lpc55::new(),
            &mut self.trussed,
            self.status,
            &self.store,
            self.clocks.is_nfc_passive,
            VERSION,
            VERSION_STRING,
        );

        let usb_bus = if !self.clocks.is_nfc_passive {
            Some(self.setup_usb_bus(usbhs))
        } else {
            None
        };

        let usb_nfc = crate::init_usb_nfc(
            resources,
            || rtic::pend(lpc55_hal::raw::Interrupt::PIN_INT6),
            usb_bus,
            self.nfc,
            self.nfc_rp,
        );

        // Cancel any possible outstanding use in delay timer
        self.basic.delay_timer.cancel().ok();

        if let Some(wwdt) = self.wwdt.as_mut() {
            debug_now!("Wwdt tv again: {:?}", wwdt.timer());
        }

        All {
            basic: self.basic,
            trussed: self.trussed,
            apps,
            endpoints,
            usb_nfc,
            wwdt: self.wwdt,
        }
    }
}

pub struct All {
    pub basic: Basic,
    pub usb_nfc: UsbNfc<NK3xN>,
    pub trussed: Trussed<NK3xN>,
    pub apps: Apps<NK3xN>,
    pub endpoints: Endpoints,
    pub wwdt: MaybeEnabledWwdt,
}

#[inline(never)]
pub fn start(syscon: hal::Syscon, pmc: hal::Pmc, anactrl: hal::Anactrl) -> Stage0 {
    let status = Default::default();
    let peripherals = Peripherals {
        syscon,
        pmc,
        anactrl,
    };
    Stage0 {
        status,
        peripherals,
    }
}
