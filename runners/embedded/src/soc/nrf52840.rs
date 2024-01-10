use apps::Variant;
use nrf52840_hal::{
    clocks::Clocks,
    usbd::{UsbPeripheral, Usbd},
};
use nrf52840_pac::{self, Interrupt, SCB};

use super::{Soc, Uuid};
use rtic_monotonic::{RtcDuration, RtcMonotonic};

pub mod flash;
pub mod rtic_monotonic;

static mut DEVICE_UUID: Uuid = [0u8; 16];

pub struct Nrf52;

impl Soc for Nrf52 {
    type UsbBus = Usbd<UsbPeripheral<'static>>;
    type Clock = RtcMonotonic;

    type Duration = RtcDuration;

    type Interrupt = Interrupt;
    const SYSCALL_IRQ: Interrupt = Interrupt::SWI0_EGU0;

    const SOC_NAME: &'static str = "nrf52";
    const VARIANT: Variant = Variant::Nrf52;

    fn device_uuid() -> &'static Uuid {
        unsafe { &DEVICE_UUID }
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

pub fn init_bootup(
    ficr: &nrf52840_pac::FICR,
    uicr: &nrf52840_pac::UICR,
    power: &mut nrf52840_pac::POWER,
) {
    let deviceid0 = ficr.deviceid[0].read().bits();
    let deviceid1 = ficr.deviceid[1].read().bits();
    unsafe {
        DEVICE_UUID[0..4].copy_from_slice(&deviceid0.to_be_bytes());
        DEVICE_UUID[4..8].copy_from_slice(&deviceid1.to_be_bytes());
        #[cfg(feature = "alpha")]
        {
            DEVICE_UUID[14] = 0xa1;
            DEVICE_UUID[15] = 0xfa;
        }
    }

    info!("RESET Reason: {:x}", power.resetreas.read().bits());
    power.resetreas.write(|w| w);

    info!(
        "FICR DeviceID {}",
        delog::hex_str!(unsafe { &types::DEVICE_UUID[0..8] })
    );
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

    if uicr.approtect.read().pall().is_enabled() {
        info!("UICR APPROTECT is ENABLED!");
    } else {
        info!("UICR APPROTECT is DISABLED!");
    };
}

type UsbClockType = Clocks<
    nrf52840_hal::clocks::ExternalOscillator,
    nrf52840_hal::clocks::Internal,
    nrf52840_hal::clocks::LfOscStarted,
>;
type UsbBusType = usb_device::bus::UsbBusAllocator<<Nrf52 as Soc>::UsbBus>;

static mut USB_CLOCK: Option<UsbClockType> = None;
static mut USBD: Option<UsbBusType> = None;

pub fn setup_usb_bus(
    clock: nrf52840_pac::CLOCK,
    usb_pac: nrf52840_pac::USBD,
) -> &'static UsbBusType {
    let usb_clock = Clocks::new(clock).start_lfclk().enable_ext_hfosc();
    unsafe {
        USB_CLOCK.replace(usb_clock);
    }
    let usb_clock_ref = unsafe { USB_CLOCK.as_ref().unwrap() };

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

    let usb_peripheral = nrf52840_hal::usbd::UsbPeripheral::new(usb_pac, usb_clock_ref);

    let usbd = nrf52840_hal::usbd::Usbd::new(usb_peripheral);
    unsafe {
        USBD.replace(usbd);
    }
    let usbd_ref = unsafe { USBD.as_ref().unwrap() };

    usbd_ref
}
