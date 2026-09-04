//! USB on-the-go high-speed (OTG), see Sections 72 to 74 of RM0486.
//!
//! Additional information can be found in [AN4879: USB on STM32 products][an4879].
//!
//! [an4879]: https://www.st.com/resource/en/application_note/an4879-introduction-to-usb-hardware-and-pcb-guidelines-using-stm32-mcus-stmicroelectronics.pdf

use core::{marker::PhantomData, ptr};

use cortex_m::{asm, interrupt};
use stm32n6::stm32n657::OTG1_S;
use synopsys_usb_otg::{PhyType, UsbBus, UsbPeripheral};

use crate::{
    Rate,
    pwr::Pwr,
    rcc::{ClockConfig, Peripheral, Rcc},
};

pub type UsbBus1 = UsbBus<Otg1>;

/// OTG1PHYCTL_CR (USBPHYC1 control register), not covered by the PAC.
const OTG1_PHY_CTL_CR: *mut u32 = 0x5803_FC00 as *mut u32;
const PHY_FSEL_MASK: u32 = 0b111 << 4;
/// FSEL value for a 24 MHz PHY reference clock (48 MHz HSE divided by two).
const PHY_FSEL_24_MHZ: u32 = 0b010 << 4;
/// OTGDISABLE0, CMN and RETENABLEN1 at their reset values.
const PHY_CR_DEFAULTS: u32 = (1 << 16) | (1 << 2) | 1;

/// GCCFG bits, VBVALEXTOEN and PULLDOWNEN are missing from the PAC.
const GCCFG_VBVALOVAL: u32 = 1 << 23;
const GCCFG_VBVALEXTOEN: u32 = 1 << 24;
const GCCFG_PULLDOWNEN: u32 = 1 << 25;

pub struct Otg1 {
    ahb_frequency: Rate,
    _marker: PhantomData<()>,
}

impl Otg1 {
    /// Brings up OTG1 and its PHY following the reset sequence of Section 72.2.2.
    pub fn new(otg1: OTG1_S, rcc: &Rcc, pwr: &Pwr, clock_config: ClockConfig) -> Self {
        let one_ms = clock_config.sys_bus_ck().to_Hz() / 1_000;

        pwr.enable_usb_supply();
        rcc.enable_hse_bypass_digital();

        rcc.assert_reset_otg1_phy_ctl();
        rcc.assert_reset(Peripheral::Otg1);
        rcc.assert_reset(Peripheral::OtgPhy1);

        rcc.select_otgphy1_hse_div2();
        rcc.enable(Peripheral::Otg1);
        rcc.enable(Peripheral::OtgPhy1);

        rcc.release_reset_otg1_phy_ctl();
        asm::delay(one_ms);
        // SAFETY: fixed peripheral register, only accessed here while the PHY is in reset.
        unsafe {
            let cr = ptr::read_volatile(OTG1_PHY_CTL_CR);
            ptr::write_volatile(
                OTG1_PHY_CTL_CR,
                (cr & !PHY_FSEL_MASK) | PHY_FSEL_24_MHZ | PHY_CR_DEFAULTS,
            );
        }

        rcc.release_reset(Peripheral::OtgPhy1);
        asm::delay(one_ms);
        rcc.release_reset(Peripheral::Otg1);

        // Device without VBUS sensing: force a valid B-session, no host pull-downs.
        // GCCFG survives the core soft reset done by the USB bus driver.
        otg1.gccfg().modify(|r, w| unsafe {
            w.bits((r.bits() & !GCCFG_PULLDOWNEN) | GCCFG_VBVALEXTOEN | GCCFG_VBVALOVAL)
        });

        Self {
            ahb_frequency: clock_config.sys_bus2_ck(),
            _marker: Default::default(),
        }
    }
}

unsafe impl UsbPeripheral for Otg1 {
    const REGISTERS: *const () = OTG1_S::ptr() as _;

    const HIGH_SPEED: bool = true;
    /// Data FIFO depth as reported by OTG_GHWCFG3.
    const FIFO_DEPTH_WORDS: usize = 952;
    const ENDPOINT_COUNT: usize = 9;

    fn enable() {
        interrupt::free(|_| {
            // SAFETY: only the OTG1 clock enable is touched, which this struct owns.
            unsafe { Rcc::steal() }.enable(Peripheral::Otg1);
        });
    }

    fn ahb_frequency_hz(&self) -> u32 {
        self.ahb_frequency.to_Hz()
    }

    fn phy_type(&self) -> PhyType {
        PhyType::InternalHighSpeed
    }
}
