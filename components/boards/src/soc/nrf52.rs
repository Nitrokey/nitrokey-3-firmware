use apps::Variant;
use nrf52840_hal::{
    clocks::Clocks,
    usbd::{UsbPeripheral, Usbd},
    wdt::{self, count::One, handles::Hdl0, Watchdog, WatchdogHandle},
};
use nrf52840_pac::{power::RESETREAS, Interrupt, SCB, WDT};

use super::{Soc, Uuid};
use rtic_monotonic::{RtcDuration, RtcMonotonic};

pub mod flash;
pub mod rtic_monotonic;

pub struct Nrf52 {
    uuid: Uuid,
}

impl Soc for Nrf52 {
    type UsbBus = Usbd<UsbPeripheral<'static>>;
    type Clock = RtcMonotonic;

    type Duration = RtcDuration;

    type Interrupt = Interrupt;
    const SYSCALL_IRQ: Interrupt = Interrupt::SWI0_EGU0;

    const SOC_NAME: &'static str = "nrf52";
    const VARIANT: Variant = Variant::Nrf52;

    fn uuid(&self) -> &Uuid {
        &self.uuid
    }
}

impl apps::Reboot for Nrf52 {
    fn reboot() -> ! {
        SCB::sys_reset()
    }
    fn reboot_to_firmware_update() -> ! {
        let pac = unsafe { nrf52840_pac::Peripherals::steal() };
        pac.POWER.gpregret.write(|w| unsafe { w.bits(0xb1_u32) });

        SCB::sys_reset()
    }
    fn reboot_to_firmware_update_destructive() -> ! {
        // @TODO: come up with an idea how to
        // factory reset, and apply!
        SCB::sys_reset()
    }
    fn locked() -> bool {
        let pac = unsafe { nrf52840_pac::Peripherals::steal() };
        pac.UICR.approtect.read().pall().is_enabled()
    }
}

pub fn init_bootup(ficr: &nrf52840_pac::FICR, uicr: &nrf52840_pac::UICR) -> Nrf52 {
    let deviceid0 = ficr.deviceid[0].read().bits();
    let deviceid1 = ficr.deviceid[1].read().bits();

    let mut uuid = Uuid::default();
    uuid[0..4].copy_from_slice(&deviceid0.to_be_bytes());
    uuid[4..8].copy_from_slice(&deviceid1.to_be_bytes());

    info!("FICR DeviceID {}", delog::hex_str!(&uuid),);
    info!(
        "FICR IdtRoot  {:08x} {:08x} {:08x} {:08x}",
        ficr.ir[0].read().bits(),
        ficr.ir[1].read().bits(),
        ficr.ir[2].read().bits(),
        ficr.ir[3].read().bits()
    );
    info!(
        "FICR EncRoot  {:08x} {:08x} {:08x} {:08x}",
        ficr.er[0].read().bits(),
        ficr.er[1].read().bits(),
        ficr.er[2].read().bits(),
        ficr.er[3].read().bits()
    );
    let mut deviceaddr: [u8; 6] = [0u8; 6];
    deviceaddr[2..6].copy_from_slice(&ficr.deviceaddr[0].read().bits().to_be_bytes());
    deviceaddr[0..2].copy_from_slice(&(ficr.deviceaddr[1].read().bits() as u16).to_be_bytes());
    info!("FICR DevAddr  {}", delog::hex_str!(&deviceaddr));

    info!(
        "UICR REGOUT0 {:x} NFCPINS {:x}",
        uicr.regout0.read().bits(),
        uicr.nfcpins.read().bits()
    );
    if !uicr.regout0.read().vout().is_3v3() {
        error!("REGOUT0 is not at 3.3V - external flash will fail!");
    }

    #[allow(clippy::if_same_then_else)]
    if uicr.approtect.read().pall().is_enabled() {
        info!("UICR APPROTECT is ENABLED!");
    } else {
        info!("UICR APPROTECT is DISABLED!");
    };

    Nrf52 { uuid }
}

pub type UsbClockType = Clocks<
    nrf52840_hal::clocks::ExternalOscillator,
    nrf52840_hal::clocks::Internal,
    nrf52840_hal::clocks::LfOscStarted,
>;
type UsbBusType = usb_device::bus::UsbBusAllocator<<Nrf52 as Soc>::UsbBus>;

pub fn setup_usb_bus(
    static_usb_clock: &'static mut Option<UsbClockType>,
    clock: nrf52840_pac::CLOCK,
    usb_pac: nrf52840_pac::USBD,
) -> UsbBusType {
    let usb_clock = static_usb_clock.insert(Clocks::new(clock).start_lfclk().enable_ext_hfosc());

    usb_pac.intenset.write(|w| {
        w.usbreset()
            .set_bit()
            .usbevent()
            .set_bit()
            .sof()
            .set_bit()
            .ep0datadone()
            .set_bit()
            .ep0setup()
            .set_bit()
    });

    Usbd::new(UsbPeripheral::new(usb_pac, usb_clock))
}

#[derive(Debug)]
pub struct ResetReason {
    pub resetpin: bool,
    /// Reset from watchdog
    pub dog: bool,
    /// Soft Reset
    pub sreq: bool,
    pub lockup: bool,
    pub off: bool,
    pub lpcomp: bool,
    pub dif: bool,
    pub nfc: bool,
    pub vbus: bool,
}

pub fn reset_reason(reset_reason: &RESETREAS) -> ResetReason {
    debug_now!("Reset Reason: {:b}", reset_reason.read().bits());
    let read = reset_reason.read();
    let res = ResetReason {
        resetpin: read.resetpin().bits(),
        dog: read.dog().bits(),
        sreq: read.sreq().bits(),
        lockup: read.lockup().bits(),
        off: read.off().bits(),
        lpcomp: read.lpcomp().bits(),
        dif: read.dif().bits(),
        nfc: read.nfc().bits(),
        vbus: read.vbus().bits(),
    };
    reset_reason.write(|w| w);
    res
}

pub fn init_watchdog(wdt: WDT) -> Result<wdt::Parts<(WatchdogHandle<Hdl0>,)>, WDT> {
    const WDT_FREQUENCY: u32 = 32_768;
    // Watchdog triggers after 3 minutes
    const DURATION_SECONS: u32 = 15 * 60;
    const TICKS: u32 = DURATION_SECONS * WDT_FREQUENCY;

    match Watchdog::try_new(wdt) {
        Ok(mut watchdog) => {
            watchdog.set_lfosc_ticks(TICKS);
            watchdog.enable_interrupt();
            let mut parts = watchdog.activate::<One>();
            parts.handles.0.pet();
            Ok(parts)
        }
        Err(wdt) => {
            let mut parts = Watchdog::try_recover::<One>(wdt)?;
            parts.handles.0.pet();
            Ok(parts)
        }
    }
}
