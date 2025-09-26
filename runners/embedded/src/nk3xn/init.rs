use core::convert::Infallible;

use apdu_dispatch::interchanges::{Channel as CcidChannel, Responder as CcidResponder};
use apps::{Endpoints, InitStatus};
use boards::{
    init::{self, UsbNfc, UsbResources},
    nk3xn::{
        button::ThreeButtons,
        led::RgbLed,
        prince,
        spi::{self, Spi, SpiConfig},
        ButtonsTimer, InternalFlashStorage, NK3xN, PwmTimer,
    },
    soc::{
        lpc55::{clock_controller::DynamicClockController, Lpc55},
        Soc,
    },
    store::{self, RunnerStore, StoreResources},
    ui::{
        buttons::{self, Press},
        rgb_led::RgbLed as _,
        UserInterface,
    },
    Apps, Trussed,
};
use embedded_hal::{
    digital::v2::OutputPin as _,
    timer::{Cancel, CountDown},
};
use embedded_hal_1::{delay as delay_1, digital as digital_1};
use embedded_hal_bus::spi::ExclusiveDevice;
use hal::{
    drivers::{clocks, flash::FlashGordon, pins, Timer},
    peripherals::{
        ctimer::{self, Ctimer},
        flexcomm::Flexcomm0,
        pfr::Pfr,
        prince::Prince,
        rng::Rng,
        usbhs::Usbhs,
        wwdt::{self, Wwdt},
    },
    raw::WWDT,
    time::{DurationExtensions as _, Microseconds, RateExtensions as _},
    typestates::{
        init_state::{Enabled, Unknown},
        pin::{gpio::direction::Output, state::Gpio},
    },
    Pin,
};
use interchange::Channel;
use littlefs2_core::path;
use lpc55_hal as hal;
#[cfg(any(feature = "log-info", feature = "log-all"))]
use lpc55_hal::drivers::timer::Elapsed as _;
use tropic01::Tropic01;
use trussed::types::Location;
use utils::OptionalStorage;
use x25519_dalek::{PublicKey, StaticSecret};

use crate::{VERSION, VERSION_STRING};

const SH0PRIV: [u8; 32] = [
    0x28, 0x3f, 0x5a, 0x0f, 0xfc, 0x41, 0xcf, 0x50, 0x98, 0xa8, 0xe1, 0x7d, 0xb6, 0x37, 0x2c, 0x3c,
    0xaa, 0xd1, 0xee, 0xee, 0xdf, 0x0f, 0x75, 0xbc, 0x3f, 0xbf, 0xcd, 0x9c, 0xab, 0x3d, 0xe9, 0x72,
];
const SH0PUB: [u8; 32] = [
    0xf9, 0x75, 0xeb, 0x3c, 0x2f, 0xd7, 0x90, 0xc9, 0x6f, 0x29, 0x4f, 0x15, 0x57, 0xa5, 0x03, 0x17,
    0x80, 0xc9, 0xaa, 0xfa, 0x14, 0x0d, 0xa2, 0x8f, 0x55, 0xe7, 0x51, 0x57, 0x37, 0xb2, 0x50, 0x2c,
];

type UsbBusType = usb_device::bus::UsbBusAllocator<<Lpc55 as Soc>::UsbBus>;

pub type WwdtEnabled = wwdt::Active;
pub type WwdtResetting = wwdt::Active;
pub type WwdtProtecting = wwdt::Inactive;
pub type EnabledWwdt = Wwdt<WwdtEnabled, WwdtResetting, WwdtProtecting>;

struct CsPin(Pin<pins::Pio1_20, Gpio<Output>>);

impl digital_1::ErrorType for CsPin {
    type Error = Infallible;
}

impl digital_1::OutputPin for CsPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0.set_low()
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.set_high()
    }
}

struct TimerDelay1<'a, T>(&'a mut T);

impl<T: CountDown<Time = Microseconds>> delay_1::DelayNs for TimerDelay1<'_, T> {
    fn delay_ns(&mut self, ns: u32) {
        self.0.start(Microseconds::new(ns / 1000));
        nb::block!(self.0.wait()).unwrap();
    }
}

struct Peripherals {
    syscon: hal::Syscon,
    pmc: hal::Pmc,
    anactrl: hal::Anactrl,
}

struct Clocks {
    clocks: clocks::Clocks,
    iocon: hal::Iocon<Enabled>,
    gpio: hal::Gpio<Enabled>,
}

pub struct Basic {
    pub delay_timer: Timer<ctimer::Ctimer0<Enabled>>,
    pub perf_timer: Timer<ctimer::Ctimer4<Enabled>>,
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
    fn enable_clocks(&mut self) -> clocks::Clocks {
        // Start out with slow clock if in passive mode;
        let frequency = 96.MHz();
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
    ) -> Stage1 {
        let mut wwdt = Wwdt::try_new(wwdt, &self.peripherals.syscon, 63).unwrap();
        // Frequency is 1/(4*64) MHz, there is a built-in 4x multiplier
        const TIMER_COUNT: u32 = (1_000_000 / (4 * 64) * boards::WATCHDOG_DURATION_SECONDS) as u32;
        wwdt.set_timer(TIMER_COUNT).unwrap();
        wwdt.set_warning(0b1_1111_1111).unwrap();
        let wwdt = wwdt.set_resetting().set_enabled();
        debug_now!("Wwdt tv: {:?}", wwdt.timer());
        let iocon = iocon.enabled(&mut self.peripherals.syscon);
        let gpio = gpio.enabled(&mut self.peripherals.syscon);

        let clocks = self.enable_clocks();
        let clocks = Clocks {
            clocks,
            iocon,
            gpio,
        };
        debug_now!("Wwdt tv again: {:?}", wwdt.timer());
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
    wwdt: EnabledWwdt,
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
        ctimer1: ctimer::Ctimer1,
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
        let spi_timer = Timer::new(
            ctimer2.enabled(syscon, self.clocks.clocks.support_1mhz_fro_token().unwrap()),
        );
        let mut perf_timer = Timer::new(
            perf_timer.enabled(syscon, self.clocks.clocks.support_1mhz_fro_token().unwrap()),
        );
        perf_timer.start(60_000_000.microseconds());

        let mut rgb = self.init_rgb(ctimer3);

        let mut three_buttons = Some(self.init_buttons(ctimer1));

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
            three_buttons,
            rgb: Some(rgb),
            old_firmware_version,
        };
        Stage2 {
            status: self.status,
            peripherals: self.peripherals,
            clocks: self.clocks,
            spi_timer,
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
    spi_timer: Timer<ctimer::Ctimer2<Enabled>>,
    wwdt: EnabledWwdt,
}

impl Stage2 {
    fn setup_spi(&mut self, flexcomm0: Flexcomm0<Unknown>, config: SpiConfig) -> Spi {
        let token = self.clocks.clocks.support_flexcomm_token().unwrap();
        let spi = flexcomm0.enabled_as_spi(&mut self.peripherals.syscon, &token);
        spi::init(spi, &mut self.clocks.iocon, config)
    }

    #[inline(never)]
    pub fn next(mut self, flexcomm0: Flexcomm0<Unknown>) -> Stage3 {
        static NFC_CHANNEL: CcidChannel = Channel::new();
        let (_, nfc_rp) = NFC_CHANNEL.split().unwrap();

        let spi = self.setup_spi(flexcomm0, SpiConfig::Tropic01);

        Stage3 {
            status: self.status,
            peripherals: self.peripherals,
            clocks: self.clocks,
            basic: self.basic,
            nfc_rp,
            spi,
            spi_timer: self.spi_timer,
            wwdt: self.wwdt,
        }
    }
}

pub struct Stage3 {
    status: InitStatus,
    peripherals: Peripherals,
    clocks: Clocks,
    basic: Basic,
    nfc_rp: CcidResponder<'static>,
    spi: Spi,
    spi_timer: Timer<ctimer::Ctimer2<Enabled>>,
    wwdt: EnabledWwdt,
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
            nfc_rp: self.nfc_rp,
            spi: self.spi,
            spi_timer: self.spi_timer,
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
    nfc_rp: CcidResponder<'static>,
    spi: Spi,
    flash: Flash,
    spi_timer: Timer<ctimer::Ctimer2<Enabled>>,
    wwdt: EnabledWwdt,
}

impl Stage4 {
    #[inline(never)]
    pub fn next(mut self, resources: &'static mut StoreResources<NK3xN>) -> Stage5 {
        info_now!("making fs");

        info_now!("simulating external flash with RAM");
        let external = OptionalStorage::default();

        #[cfg(not(feature = "no-encrypted-storage"))]
        let internal = {
            #[cfg(feature = "write-undefined-flash")]
            initialize_fs_flash(&mut self.flash.flash_gordon, &mut self.flash.prince);

            InternalFlashStorage::new(self.flash.flash_gordon, self.flash.prince)
        };

        #[cfg(feature = "no-encrypted-storage")]
        let internal = InternalFlashStorage::new(self.flash.flash_gordon);

        info_now!(
            "mount start {} ms",
            self.basic.perf_timer.elapsed().0 / 1000
        );
        // TODO: poll iso14443
        let simulated_efs = external.is_ram();
        let store = store::init_store(
            resources,
            internal,
            external,
            simulated_efs,
            &mut self.status,
        );
        info!("mount end {} ms", self.basic.perf_timer.elapsed().0 / 1000);

        Stage5 {
            status: self.status,
            peripherals: self.peripherals,
            clocks: self.clocks,
            basic: self.basic,
            rng: self.flash.rng,
            nfc_rp: self.nfc_rp,
            spi: self.spi,
            spi_timer: self.spi_timer,
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
    nfc_rp: CcidResponder<'static>,
    rng: Rng<Enabled>,
    store: RunnerStore<NK3xN>,
    spi: Spi,
    spi_timer: Timer<ctimer::Ctimer2<Enabled>>,
    wwdt: EnabledWwdt,
}

#[inline(never)]
fn get_seed_from_tropic01(
    iocon: &mut hal::Iocon<Enabled>,
    gpio: &mut hal::Gpio<Enabled>,
    spi: Spi,
    timer: &mut Timer<ctimer::Ctimer2<Enabled>>,
    rng: &mut Rng<Enabled>,
) -> [u8; 32] {
    let cs = pins::Pio1_20::take()
        .unwrap()
        .into_gpio_pin(iocon, gpio)
        .into_output_high();
    let cs = CsPin(cs);
    let delay = TimerDelay1(timer);
    let spi = ExclusiveDevice::new(spi, cs, delay).expect("failed to set up SPI device");
    let mut tropic01 = Tropic01::new(spi);

    let ehpriv = StaticSecret::random_from_rng(rng);
    let ehpub = PublicKey::from(&ehpriv);
    tropic01
        .session_start(SH0PUB.into(), SH0PRIV.into(), ehpub, ehpriv, 0)
        .expect("failed to start session");
    tropic01
        .get_random_value(32)
        .expect("failed to get random value")
        .try_into()
        .expect("failed to convert random value")
}

impl Stage5 {
    #[inline(never)]
    pub fn next(mut self, rtc: hal::peripherals::rtc::Rtc<Unknown>) -> Stage6 {
        let syscon = &mut self.peripherals.syscon;
        let pmc = &mut self.peripherals.pmc;
        let clocks = self.clocks.clocks;

        let mut rtc = rtc.enabled(syscon, clocks.enable_32k_fro(pmc));
        rtc.reset();

        let rgb = self.basic.rgb.take();

        let three_buttons = self.basic.three_buttons.take();

        let user_interface = UserInterface::new(rtc, three_buttons, rgb);

        let seed = get_seed_from_tropic01(
            &mut self.clocks.iocon,
            &mut self.clocks.gpio,
            self.spi,
            &mut self.spi_timer,
            &mut self.rng,
        );

        let trussed = init::init_trussed(
            &mut self.rng,
            self.store,
            user_interface,
            &mut self.status,
            Some(seed),
            None,
            #[cfg(feature = "se050")]
            None,
        );

        Stage6 {
            status: self.status,
            peripherals: self.peripherals,
            clocks: self.clocks,
            basic: self.basic,
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
    nfc_rp: CcidResponder<'static>,
    store: RunnerStore<NK3xN>,
    trussed: Trussed<NK3xN>,
    wwdt: EnabledWwdt,
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
            false,
            VERSION,
            VERSION_STRING,
        );

        let usb_bus = Some(self.setup_usb_bus(usbhs));

        let usb_nfc = crate::init_usb_nfc(resources, usb_bus, None, self.nfc_rp);

        // Cancel any possible outstanding use in delay timer
        self.basic.delay_timer.cancel().ok();

        let clock_controller = None;

        info!("init took {} ms", self.basic.perf_timer.elapsed().0 / 1000);
        debug_now!("Wwdt tv again: {:?}", self.wwdt.timer());

        All {
            basic: self.basic,
            trussed: self.trussed,
            apps,
            endpoints,
            clock_controller,
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
    pub clock_controller: Option<DynamicClockController>,
    pub wwdt: EnabledWwdt,
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
