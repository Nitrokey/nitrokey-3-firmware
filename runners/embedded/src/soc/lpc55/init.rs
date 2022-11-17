use embedded_hal::timer::{Cancel, CountDown};
use hal::{
    drivers::{
        clocks,
        flash::FlashGordon,
        pins::{self, direction},
        Timer,
    },
    peripherals::{
        ctimer::{self, Ctimer},
        pfr::Pfr,
        prince::Prince,
        rng::Rng,
    },
    time::{DurationExtensions as _, RateExtensions as _},
    traits::wg::digital::v2::InputPin,
    typestates::init_state::Unknown,
    typestates::pin::state::Gpio,
};
use lpc55_hal as hal;
#[allow(unused)]
use lpc55_hal::drivers::timer::Elapsed;
use trussed::platform::UserInterface;

use super::{board, clock_controller::DynamicClockController, nfc, spi};
use crate::{
    traits::{
        buttons::{self, Press},
        rgb_led::RgbLed,
    },
    types::{self, usbnfc::UsbNfcInit, Apps, RunnerStore, Trussed},
};

type UsbNfc = UsbNfcInit<super::types::Soc>;

struct Peripherals {
    syscon: hal::Syscon,
    pmc: hal::Pmc,
    anactrl: hal::Anactrl,
}

struct Clocks {
    is_nfc_passive: bool,
    clocks: clocks::Clocks,
    nfc_irq: Option<hal::Pin<nfc::NfcIrqPin, Gpio<direction::Input>>>,
    iocon: hal::Iocon<hal::Enabled>,
    gpio: hal::Gpio<hal::Enabled>,
}

pub struct Basic {
    pub delay_timer: Timer<ctimer::Ctimer0<hal::Enabled>>,
    pub perf_timer: Timer<ctimer::Ctimer4<hal::Enabled>>,
    adc: Option<hal::Adc<hal::Enabled>>,
    three_buttons: Option<board::button::ThreeButtons>,
    rgb: Option<board::led::RgbLed>,
    old_firmware_version: u32,
}

struct Flash {
    flash_gordon: FlashGordon,
    #[allow(unused)]
    prince: Prince<hal::Enabled>,
    rng: Rng<hal::Enabled>,
}

pub struct Stage0 {
    peripherals: Peripherals,
}

impl Stage0 {
    fn enable_low_speed_for_passive_nfc(
        &mut self,
        mut iocon: hal::Iocon<hal::Enabled>,
        gpio: &mut hal::Gpio<hal::Enabled>,
    ) -> (
        hal::Iocon<hal::Enabled>,
        hal::Pin<nfc::NfcIrqPin, Gpio<direction::Input>>,
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

        (iocon, nfc_irq, is_passive_mode)
    }

    fn enable_clocks(&mut self, is_nfc_passive: bool) -> clocks::Clocks {
        // Start out with slow clock if in passive mode;
        let frequency = if is_nfc_passive { 4.MHz() } else { 96.MHz() };
        hal::ClockRequirements::default()
            .system_frequency(frequency)
            .configure(
                &mut self.peripherals.anactrl,
                &mut self.peripherals.pmc,
                &mut self.peripherals.syscon,
            )
            .expect("Clock configuration failed")
    }

    pub fn next(mut self, iocon: hal::Iocon<Unknown>, gpio: hal::Gpio<Unknown>) -> Stage1 {
        let mut iocon = iocon.enabled(&mut self.peripherals.syscon);
        let mut gpio = gpio.enabled(&mut self.peripherals.syscon);

        let (new_iocon, nfc_irq, is_nfc_passive) =
            self.enable_low_speed_for_passive_nfc(iocon, &mut gpio);
        iocon = new_iocon;
        let nfc_irq = Some(nfc_irq);

        let clocks = self.enable_clocks(is_nfc_passive);
        let clocks = Clocks {
            is_nfc_passive,
            clocks,
            nfc_irq,
            iocon,
            gpio,
        };
        Stage1 {
            peripherals: self.peripherals,
            clocks,
        }
    }
}

pub struct Stage1 {
    peripherals: Peripherals,
    clocks: Clocks,
}

impl Stage1 {
    fn validate_cfpa(
        pfr: &mut Pfr<hal::Enabled>,
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

    fn is_bootrom_requested<T: Ctimer<hal::Enabled>>(
        &mut self,
        three_buttons: &mut board::button::ThreeButtons,
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

    fn init_rgb(&mut self, ctimer: board::PwmTimer) -> board::led::RgbLed {
        #[cfg(feature = "board-nk3xn")]
        {
            board::led::RgbLed::new(
                hal::drivers::Pwm::new(ctimer.enabled(
                    &mut self.peripherals.syscon,
                    self.clocks.clocks.support_1mhz_fro_token().unwrap(),
                )),
                &mut self.clocks.iocon,
            )
        }
    }

    fn init_buttons(&mut self, ctimer: board::ButtonsTimer) -> board::button::ThreeButtons {
        #[cfg(feature = "board-nk3xn")]
        {
            board::button::ThreeButtons::new(
                Timer::new(ctimer.enabled(
                    &mut self.peripherals.syscon,
                    self.clocks.clocks.support_1mhz_fro_token().unwrap(),
                )),
                &mut self.clocks.gpio,
                &mut self.clocks.iocon,
            )
        }
    }

    pub fn next(
        mut self,
        adc: hal::Adc<Unknown>,
        delay_timer: ctimer::Ctimer0,
        ctimer1: ctimer::Ctimer1,
        ctimer3: ctimer::Ctimer3,
        perf_timer: ctimer::Ctimer4,
        pfr: Pfr<Unknown>,
        secure_firmware_version: Option<u32>,
        require_prince: bool,
        boot_to_bootrom: bool,
    ) -> Stage2 {
        let pmc = &mut self.peripherals.pmc;
        let syscon = &mut self.peripherals.syscon;

        // Start out with slow clock if in passive mode;
        #[allow(unused_mut)]
        let mut adc = Some(if self.clocks.is_nfc_passive {
            // important to start Adc early in passive mode
            hal::Adc::from(adc)
                .configure(DynamicClockController::adc_configuration())
                .enabled(pmc, syscon)
        } else {
            hal::Adc::from(adc).enabled(pmc, syscon)
        });

        let mut delay_timer = Timer::new(
            delay_timer.enabled(syscon, self.clocks.clocks.support_1mhz_fro_token().unwrap()),
        );
        let mut perf_timer = Timer::new(
            perf_timer.enabled(syscon, self.clocks.clocks.support_1mhz_fro_token().unwrap()),
        );
        perf_timer.start(60_000_000.microseconds());

        let mut rgb = self.init_rgb(ctimer3);

        let mut three_buttons = if !self.clocks.is_nfc_passive {
            Some(self.init_buttons(ctimer1))
        } else {
            None
        };

        let mut pfr = pfr.enabled(&self.clocks.clocks).unwrap();
        let old_firmware_version =
            Self::validate_cfpa(&mut pfr, secure_firmware_version, require_prince);

        if boot_to_bootrom && three_buttons.is_some() {
            info!("bootrom request start {}", perf_timer.elapsed().0 / 1000);
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
            perf_timer,
            adc,
            three_buttons,
            rgb: Some(rgb),
            old_firmware_version,
        };
        Stage2 {
            peripherals: self.peripherals,
            clocks: self.clocks,
            basic,
        }
    }
}

pub struct Stage2 {
    peripherals: Peripherals,
    clocks: Clocks,
    basic: Basic,
}

type UsbPeripheralType = lpc55_hal::peripherals::usbhs::Usbhs;

type UsbBusType = usb_device::bus::UsbBusAllocator<<super::types::Soc as types::Soc>::UsbBus>;

static mut USBD: Option<UsbBusType> = None;

impl Stage2 {
    fn setup_spi(&mut self, flexcomm0: hal::peripherals::flexcomm::Flexcomm0<Unknown>) -> spi::Spi {
        let token = self.clocks.clocks.support_flexcomm_token().unwrap();
        let spi = flexcomm0.enabled_as_spi(&mut self.peripherals.syscon, &token);
        spi::init(spi, &mut self.clocks.iocon)
    }

    fn setup_usb_bus(&mut self, usbp: UsbPeripheralType) -> &'static UsbBusType {
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

        let usbd = lpc55_hal::drivers::UsbBus::new(usb, vbus_pin);
        unsafe {
            USBD.replace(usbd);
        }

        unsafe { USBD.as_ref().unwrap() }
    }

    fn setup_fm11nc08(
        &mut self,
        spi: spi::Spi,
        inputmux: lpc55_hal::peripherals::inputmux::InputMux<Unknown>,
        pint: lpc55_hal::peripherals::pint::Pint<Unknown>,
    ) -> Option<nfc::NfcChip> {
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

        let force_nfc_reconfig = cfg!(feature = "reconfigure-nfc");

        nfc::try_setup(
            spi,
            &mut self.clocks.gpio,
            &mut self.clocks.iocon,
            nfc_irq,
            &mut self.basic.delay_timer,
            force_nfc_reconfig,
        )
    }

    #[cfg(feature = "provisioner-app")]
    pub fn run_hardware_checks(
        &mut self,
        flexcomm5: hal::peripherals::flexcomm::Flexcomm5<Unknown>,
        spi: &mut spi::Spi,
    ) {
        use crate::flash::ExtFlashStorage;
        use embedded_hal::blocking::i2c::{Read, Write};
        use littlefs2::fs::Filesystem;

        // SE050 check
        let _enabled = pins::Pio1_26::take()
            .unwrap()
            .into_gpio_pin(&mut self.clocks.iocon, &mut self.clocks.gpio)
            .into_output_high();

        self.basic.delay_timer.start(100_000.microseconds());
        nb::block!(self.basic.delay_timer.wait()).ok();

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
            hal::time::Hertz::try_from(100_u32.kHz()).unwrap(),
        );

        self.basic.delay_timer.start(100_000.microseconds());
        nb::block!(self.basic.delay_timer.wait()).ok();

        // RESYNC command
        let command = [0x5a, 0xc0, 0x00, 0xff, 0xfc];
        i2c.write(0x48, &command)
            .expect("failed to send RESYNC command");

        self.basic.delay_timer.start(100_000.microseconds());
        nb::block!(self.basic.delay_timer.wait()).ok();

        // RESYNC response
        let mut response = [0; 2];
        i2c.read(0x48, &mut response)
            .expect("failed to read RESYNC response");

        if response != [0xa5, 0xe0] {
            panic!("Unexpected RESYNC response: {:?}", response);
        }

        // External Flash checks
        let spi = spi::SpiMut(spi);
        let flash_cs = pins::Pio0_13::take()
            .unwrap()
            .into_gpio_pin(&mut self.clocks.iocon, &mut self.clocks.gpio)
            .into_output_high();
        let _power = pins::Pio0_21::take()
            .unwrap()
            .into_gpio_pin(&mut self.clocks.iocon, &mut self.clocks.gpio)
            .into_output_high();

        self.basic.delay_timer.start(200_000.microseconds());
        nb::block!(self.basic.delay_timer.wait()).ok();

        let mut flash = ExtFlashStorage::new(spi, flash_cs);
        if !Filesystem::is_mountable(&mut flash) {
            debug_now!("external filesystem not mountable, trying to format");
            Filesystem::format(&mut flash).expect("failed to format external filesystem");
        }
        if !Filesystem::is_mountable(&mut flash) {
            panic!("filesystem not mountable after format");
        }

        info_now!("hardware checks successful");
    }

    pub fn next(
        mut self,
        flexcomm0: hal::peripherals::flexcomm::Flexcomm0<Unknown>,
        _flexcomm5: hal::peripherals::flexcomm::Flexcomm5<Unknown>,
        mux: hal::peripherals::inputmux::InputMux<Unknown>,
        pint: hal::peripherals::pint::Pint<Unknown>,
        usbhs: hal::peripherals::usbhs::Usbhs<Unknown>,
        nfc_enabled: bool,
    ) -> Stage3 {
        #[allow(unused_mut)]
        let mut spi = self.setup_spi(flexcomm0);
        #[cfg(feature = "provisioner-app")]
        self.run_hardware_checks(_flexcomm5, &mut spi);
        let spi = spi::reconfigure(spi);

        let nfc_chip = if nfc_enabled {
            self.setup_fm11nc08(spi, mux, pint)
        } else {
            None
        };

        let usb_bus = if !self.clocks.is_nfc_passive {
            Some(self.setup_usb_bus(usbhs))
        } else {
            None
        };

        let usb_nfc = crate::init_usb_nfc(usb_bus, nfc_chip);
        Stage3 {
            peripherals: self.peripherals,
            clocks: self.clocks,
            basic: self.basic,
            usb_nfc,
        }
    }
}

pub struct Stage3 {
    peripherals: Peripherals,
    clocks: Clocks,
    basic: Basic,
    usb_nfc: UsbNfc,
}

impl Stage3 {
    pub fn next(
        mut self,
        rng: hal::peripherals::rng::Rng<Unknown>,
        prince: hal::peripherals::prince::Prince<Unknown>,
        flash: hal::peripherals::flash::Flash<Unknown>,
    ) -> Stage4 {
        info_now!("making flash");
        let syscon = &mut self.peripherals.syscon;

        #[allow(unused_mut)]
        let mut rng = rng.enabled(syscon);

        let prince = prince.enabled(&mut rng);
        prince.disable_all_region_2();

        let flash_gordon = FlashGordon::new(flash.enabled(syscon));

        let flash = Flash {
            flash_gordon,
            prince,
            rng,
        };
        Stage4 {
            peripherals: self.peripherals,
            clocks: self.clocks,
            basic: self.basic,
            usb_nfc: self.usb_nfc,
            flash,
        }
    }
}

pub struct Stage4 {
    peripherals: Peripherals,
    clocks: Clocks,
    basic: Basic,
    usb_nfc: UsbNfc,
    flash: Flash,
}

impl Stage4 {
    pub fn next(mut self) -> Stage5 {
        let syscon = &mut self.peripherals.syscon;
        let pmc = &mut self.peripherals.pmc;
        info_now!("making fs");

        #[cfg(not(feature = "no-encrypted-storage"))]
        let internal = {
            #[cfg(feature = "write-undefined-flash")]
            initialize_fs_flash(&mut self.flash.flash_gordon, &mut self.flash.prince);

            super::types::InternalFilesystem::new(self.flash.flash_gordon, self.flash.prince)
        };

        #[cfg(feature = "no-encrypted-storage")]
        let internal = super::types::InternalFilesystem::new(self.flash.flash_gordon);
        let external = super::types::ExternalRAMStorage::new();

        // temporarily increase clock for the storage mounting or else it takes a long time.
        if self.clocks.is_nfc_passive {
            self.clocks.clocks = unsafe {
                hal::ClockRequirements::default()
                    .system_frequency(48.MHz())
                    .reconfigure(self.clocks.clocks, pmc, syscon)
            };
        }

        info_now!(
            "mount start {} ms",
            self.basic.perf_timer.elapsed().0 / 1000
        );
        // TODO: poll iso14443
        let store = crate::init_store::<super::types::Soc>(internal, external);
        info!("mount end {} ms", self.basic.perf_timer.elapsed().0 / 1000);

        // return to slow freq
        if self.clocks.is_nfc_passive {
            self.clocks.clocks = unsafe {
                hal::ClockRequirements::default()
                    .system_frequency(12.MHz())
                    .reconfigure(self.clocks.clocks, pmc, syscon)
            };
        }

        if let Some(iso14443) = &mut self.usb_nfc.iso14443 {
            iso14443.poll();
        }

        // Cancel any possible outstanding use in delay timer
        self.basic.delay_timer.cancel().ok();

        Stage5 {
            peripherals: self.peripherals,
            clocks: self.clocks,
            basic: self.basic,
            usb_nfc: self.usb_nfc,
            rng: self.flash.rng,
            store,
        }
    }
}

#[cfg(feature = "write-undefined-flash")]
/// This is necessary if prince encryption is enabled for the first time
/// after it was first provisioned.  In this case, there can be an exception
/// reading from undefined flash.  To fix, we run a pass over all filesystem
/// flash and set it to a defined value.
fn initialize_fs_flash(flash_gordon: &mut FlashGordon, prince: &mut Prince<hal::Enabled>) {
    use crate::types::build_constants;
    use lpc55_hal::traits::flash::{Read, WriteErase};

    let page_count = ((631 * 1024 + 512) - build_constants::CONFIG_FILESYSTEM_BOUNDARY) / 512;

    let mut page_data = [0u8; 512];
    for page in 0..page_count {
        // With prince turned off, this should read as encrypted bytes.
        flash_gordon.read(
            build_constants::CONFIG_FILESYSTEM_BOUNDARY + page * 512,
            &mut page_data,
        );

        // But if it's zero, then that means the data is undefined and it doesn't bother.
        if page_data == [0u8; 512] {
            info_now!("resetting page {}", page);
            // So we should write nonzero data to initialize flash.
            // We write it as encrypted, so it is in a known state when decrypted by the filesystem layer.
            page_data[0] = 1;
            flash_gordon
                .erase_page(build_constants::CONFIG_FILESYSTEM_BOUNDARY / 512 + page)
                .ok();
            prince.write_encrypted(|prince| {
                prince.enable_region_2_for(|| {
                    flash_gordon
                        .write(
                            build_constants::CONFIG_FILESYSTEM_BOUNDARY + page * 512,
                            &page_data,
                        )
                        .unwrap();
                })
            });
        }
    }
}

pub struct Stage5 {
    peripherals: Peripherals,
    clocks: Clocks,
    basic: Basic,
    usb_nfc: UsbNfc,
    rng: Rng<hal::Enabled>,
    store: RunnerStore<super::types::Soc>,
}

impl Stage5 {
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

        let provisioner = cfg!(feature = "provisioner-app");
        let mut solobee_interface =
            super::trussed::UserInterface::new(rtc, three_buttons, rgb, provisioner);
        solobee_interface.set_status(trussed::platform::ui::Status::Idle);

        let board = types::RunnerPlatform::new(self.rng, self.store, solobee_interface);
        let trussed = trussed::service::Service::new(board);

        Stage6 {
            peripherals: self.peripherals,
            clocks: self.clocks,
            basic: self.basic,
            usb_nfc: self.usb_nfc,
            store: self.store,
            trussed,
        }
    }
}

pub struct Stage6 {
    peripherals: Peripherals,
    clocks: Clocks,
    basic: Basic,
    usb_nfc: UsbNfc,
    store: RunnerStore<super::types::Soc>,
    trussed: Trussed<super::types::Soc>,
}

impl Stage6 {
    fn perform_data_migrations(&self) {
        // FIDO2 attestation cert (<= 1.0.2)
        if self.basic.old_firmware_version <= 4194306 {
            debug!("data migration: updating FIDO2 attestation cert");
            let res = trussed::store::store(
                self.store,
                trussed::types::Location::Internal,
                &littlefs2::path::PathBuf::from("fido/x5c/00"),
                include_bytes!("../../../data/fido-cert.der"),
            );
            if res.is_err() {
                error!("failed to replace attestation cert");
            }
        }
    }

    pub fn next(mut self) -> All {
        self.perform_data_migrations();
        let apps = crate::init_apps(&mut self.trussed, &self.store, self.clocks.is_nfc_passive);
        let clock_controller = if self.clocks.is_nfc_passive {
            let adc = self.basic.adc.take();
            let clocks = self.clocks.clocks;

            let pmc = self.peripherals.pmc;
            let syscon = self.peripherals.syscon;

            let gpio = &mut self.clocks.gpio;
            let iocon = &mut self.clocks.iocon;

            let mut new_clock_controller =
                DynamicClockController::new(adc.unwrap(), clocks, pmc, syscon, gpio, iocon);
            new_clock_controller.start_high_voltage_compare();

            Some(new_clock_controller)
        } else {
            None
        };

        info!("init took {} ms", self.basic.perf_timer.elapsed().0 / 1000);

        All {
            basic: self.basic,
            usb_nfc: self.usb_nfc,
            trussed: self.trussed,
            apps,
            clock_controller,
        }
    }
}

pub struct All {
    pub basic: Basic,
    pub usb_nfc: UsbNfc,
    pub trussed: Trussed<super::types::Soc>,
    pub apps: Apps<super::types::Soc>,
    pub clock_controller: Option<DynamicClockController>,
}

pub fn start(syscon: hal::Syscon, pmc: hal::Pmc, anactrl: hal::Anactrl) -> Stage0 {
    let peripherals = Peripherals {
        syscon,
        pmc,
        anactrl,
    };
    Stage0 { peripherals }
}
