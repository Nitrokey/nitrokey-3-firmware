//! Reset and clock control, see Section 14 of RM0486.

use stm32n6::stm32n657::RCC;

use crate::Rate;

pub struct Rcc(RCC);

/// find the divider for the input frequency so that input_rate / divider is closest
/// to target_rate/16
fn pll_ideal_input_divider(input_rate: Rate, target_rate: Rate) -> u8 {
    let ideal_input_rate = target_rate / 20;

    for d in 1..0x3F {
        if input_rate / d > ideal_input_rate {
            continue;
        } else {
            return d as u8;
        }
    }
    // use minimal divider
    return 0x3F;
}

/// Returns divm, divn and divnfrac
fn pll_divider_rates(input_rate: Rate, target_rate: Rate) -> (u8, u16, u32) {
    let input_divider = pll_ideal_input_divider(input_rate, target_rate);
    let integer_divider = (target_rate * input_divider as u32) / input_rate;
    let frac_divider = ((target_rate.raw() as u64 * input_divider as u64) << 24)
        / (input_rate.raw() as u64)
        - ((integer_divider as u64) << 24);
    return (
        input_divider,
        integer_divider.try_into().unwrap(),
        frac_divider.try_into().unwrap(),
    );
}

impl Rcc {
    pub fn new(rcc: RCC) -> Self {
        Self(rcc)
    }

    /// # Safety
    ///
    /// See [`RCC::steal`][].
    pub unsafe fn steal() -> Self {
        unsafe { Self::new(RCC::steal()) }
    }

    pub fn clock_config(&self) -> ClockConfig {
        let cfgr1 = self.0.cfgr1().read();
        let cfgr2 = self.0.cfgr2().read();

        let system_clock = SystemClock::from_bits(cfgr1.syssws().bits());

        let prescaler_ahb = cfgr2.hpre().bits();
        let prescaler_timer = cfgr2.timpre().bits();

        ClockConfig {
            system_clock,
            prescaler_ahb,
            prescaler_timer,
        }
    }

    pub fn enable(&self, peripheral: Peripheral) {
        peripheral.enable(&self.0);
    }

    pub fn sdmmc1_kernel_clock(&self) -> Rate {
        self.sdmmc_kernel_clock(self.0.ccipr8().read().sdmmc1sel().bits())
    }

    pub fn sdmmc2_kernel_clock(&self) -> Rate {
        self.sdmmc_kernel_clock(self.0.ccipr8().read().sdmmc2sel().bits())
    }

    /// SDMMCxSEL: 0 = hclku (the AHB clock)
    fn sdmmc_kernel_clock(&self, sel: u8) -> Rate {
        match sel {
            0 => self.clock_config().sys_bus2_ck(),
            _ => unimplemented!(),
        }
    }

    /// XSPI1SEL: 0 = hclk5 (the AHB clock)
    pub fn xspi1_kernel_clock(&self) -> Rate {
        match self.0.ccipr6().read().xspi1sel().bits() {
            0 => self.clock_config().sys_bus2_ck(),
            _ => unimplemented!(),
        }
    }

    pub fn enable_pll1(&self, target_rate: Rate) {
        self.0.ccr().write(|w| w.pll1onc().set_bit());

        let hsi_rate = SystemClock::Hsi.frequency();
        let (input_divider, integer_divider, frac_divider) =
            pll_divider_rates(hsi_rate, target_rate);
        self.0.pll1cfgr1().modify(|_, w| unsafe {
            w.pll1divm()
                .bits(input_divider)
                .pll1divn()
                .bits(integer_divider)
                .pll1sel()
                .bits(SystemClock::Hsi as u8)
        });
        self.0
            .pll1cfgr2()
            .modify(|_, w| unsafe { w.pll1divnfrac().bits((frac_divider >> 24) as u32) });
        self.0.pll1cfgr3().modify(|_, w| {
            w.pll1moddsen()
                .set_bit()
                .pll1dacen()
                .set_bit()
                .pll1dacen()
                .set_bit()
                .pll1modssrst()
                .set_bit()
        });

        self.0.csr().write(|w| w.pll1ons().bit(true));
        debug_now!("Waiting for pll1rdy");
        while !self.0.sr().read().pll1rdy().bit() {}

        self.0.ic1cfgr().write(|w| unsafe {
            // Select PLL1 output for IC1
            w.ic1sel()
                .bits(0b00)
                // Set divider to 1 (PLL outout straight to CPU)
                .ic1int()
                .bits(0)
        });
        // Enable IC1
        self.0.divenr().modify(|_, w| w.ic1en().set_bit());

        // Select CPU clock source and system clock to be PLL1
        self.0
            .cfgr1()
            .modify(|_, w| unsafe { w.cpusw().bits(0b11) });

        debug_now!("Waiting for cpsws");
        while self.0.cfgr1().read().cpusws().bits() != 0b11 {}
        // debug_now!("Waiting for syssws");
        // while self.0.cfgr1().read().syssws().bits() != 0b11 {}
    }

    pub fn reset(&self, peripheral: Peripheral) {
        self.assert_reset(peripheral);
        self.release_reset(peripheral);
    }

    pub fn assert_reset(&self, peripheral: Peripheral) {
        peripheral.assert_reset(&self.0);
    }

    pub fn release_reset(&self, peripheral: Peripheral) {
        peripheral.release_reset(&self.0);
    }

    /// Enable HSE crystal on OSC_IN/OSC_OUT and waits until it's ready
    pub fn enable_hse(&self) {
        self.0.cr().modify(|_, w| w.hseon().clear_bit());
        self.0
            .hsecfgr()
            .modify(|_, w| w.hsebyp().clear_bit().hseext().clear_bit());
        self.0.cr().modify(|_, w| w.hseon().set_bit());
        while self.0.sr().read().hserdy().bit_is_clear() {}
    }

    /// Feeds the OTGPHY1 clock with 48 MHz HSE divided by two (see Section 14.7).
    /// Must be called while the OTGPHY1 clock is disabled.
    pub fn select_otgphy1_hse_div2(&self) {
        // HSEDIV2SEL (hsediv2byp in the PAC): 1 = hse_div2_osc_ck is hse_osc_ck / 2
        self.0.hsecfgr().modify(|_, w| w.hsediv2byp().set_bit());
        self.0.ccipr6().modify(|_, w| {
            // SAFETY: 0b00 selects hse_div2_ck for the kernel clock mux.
            unsafe { w.otgphy1sel().bits(0) }
                .otgphy1ckrefsel()
                .set_bit()
        });
    }

    /// I2C1 kernel clock from hsi_div_ck (64 MHz), independent of the bus prescalers.
    pub fn select_i2c1_hsi(&self) {
        // 0b101 selects hsi_div_ck (RM0486 RCC_CCIPR4 I2C1SEL).
        self.0
            .ccipr4()
            .modify(|_, w| unsafe { w.i2c1sel().bits(0b101) });
    }

    /// The OTG1 PHY controller is clocked through OTG1 and has no enable bit of its own.
    pub fn assert_reset_otg1_phy_ctl(&self) {
        self.0
            .ahb5rstsr()
            .write(|w| w.syscfgotghsphy1rsts().set_bit());
    }

    pub fn release_reset_otg1_phy_ctl(&self) {
        self.0
            .ahb5rstcr()
            .write(|w| w.syscfgotghsphy1rstc().set_bit());
    }
}

macro_rules! impl_peripheral {
    ($(($ensr:ident, $rstsr:ident, $rstcr:ident) => [
        $(($peripheral:ident, $ens:ident, $rsts:ident, $rstc:ident),)*
    ],)*) => {
        #[derive(Clone, Copy)]
        pub enum Peripheral {
            $($($peripheral,)*)*
        }

        impl Peripheral {
            fn enable(&self, rcc: &RCC) {
                match self {
                    $($(
                        Self::$peripheral => rcc.$ensr().write(|w| w.$ens().set_bit()),
                    )*)*
                };
            }

            fn assert_reset(&self, rcc: &RCC) {
                match self {
                    $($(
                        Self::$peripheral => {
                            rcc.$rstsr().write(|w| w.$rsts().set_bit());
                        }
                    )*)*
                }
            }

            fn release_reset(&self, rcc: &RCC) {
                match self {
                    $($(
                        Self::$peripheral => {
                            rcc.$rstcr().write(|w| w.$rstc().set_bit());
                        }
                    )*)*
                }
            }
        }
    };
}

impl_peripheral!(
    (ahb3ensr, ahb3rstsr, ahb3rstcr) => [
        (Cryp, crypens, cryprsts, cryprstc),
        (Rng, rngens, rngrsts, rngrstc),
        (Saes, saesens, saesrsts, saesrstc),
    ],
    (ahb4ensr, ahb4rstsr, ahb4rstcr) => [
        (GpioA, gpioaens, gpioarsts, gpioarstc),
        (GpioB, gpiobens, gpiobrsts, gpiobrstc),
        (GpioC, gpiocens, gpiocrsts, gpiocrstc),
        (GpioD, gpiodens, gpiodrsts, gpiodrstc),
        (GpioE, gpioeens, gpioersts, gpioerstc),
        (GpioF, gpiofens, gpiofrsts, gpiofrstc),
        (GpioG, gpiogens, gpiogrsts, gpiogrstc),
        (GpioH, gpiohens, gpiohrsts, gpiohrstc),
        (GpioO, gpiooens, gpioorsts, gpioorstc),
        (GpioP, gpiopens, gpioprsts, gpioprstc),
    ],
    (ahb5ensr, ahb5rstsr, ahb5rstcr) => [
        (Otg1, otg1ens, otg1rsts, otg1rstc),
        (OtgPhy1, otgphy1ens, otgphy1rsts, otgphy1rstc),
        (Sdmmc1, sdmmc1ens, sdmmc1rsts, sdmmc1rstc),
        (Sdmmc2, sdmmc2ens, sdmmc2rsts, sdmmc2rstc),
        (Xspi1, xspi1ens, xspi1rsts, xspi1rstc),
        (Xspim, xspimens, xspimrsts, xspimrstc),
    ],
    (apb1lensr, apb1lrstsr, apb1lrstcr) => [
        (I2c1, i2c1ens, i2c1rsts, i2c1rstc),
        (Tim6, tim6ens, tim6rsts, tim6rstc),
        (Tim7, tim7ens, tim7rsts, tim7rstc),
    ],
    (apb4hensr, apb4hrstsr, apb4hrstcr) => [
        (Syscfg, syscfgens, syscfgrsts, syscfgrstc),
    ],
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClockConfig {
    pub system_clock: SystemClock,
    pub prescaler_ahb: u8,
    pub prescaler_timer: u8,
}

impl ClockConfig {
    pub const DEFAULT: Self = Self {
        system_clock: SystemClock::Hsi,
        prescaler_ahb: 1,
        prescaler_timer: 0,
    };

    pub const fn sys_bus_ck(&self) -> Rate {
        self.system_clock.frequency()
    }

    pub const fn sys_bus2_ck(&self) -> Rate {
        scale(self.sys_bus_ck(), self.prescaler_ahb).unwrap()
    }

    pub const fn timg_ck(&self) -> Rate {
        scale(self.sys_bus_ck(), self.prescaler_timer).unwrap()
    }
}

const fn scale(f: Rate, prescaler: u8) -> Option<Rate> {
    let mut frequency = f.raw();
    let mut i = 0;
    while i < prescaler {
        if frequency % 2 != 0 {
            return None;
        }
        frequency /= 2;
        i += 1;
    }
    Some(Rate::from_raw(frequency))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemClock {
    /// sysb_ck = hsi_ck
    Hsi = 0b00,
    /// sysb_ck = msi_ck
    Msi = 0b01,
    /// sysb_ck = hse_ck
    Hse = 0b10,
    /// sysb_ck = ic2_ck
    Ic2 = 0b11,
}

impl SystemClock {
    const fn from_bits(bits: u8) -> Self {
        match bits {
            0b00 => Self::Hsi,
            0b01 => Self::Msi,
            0b10 => Self::Hse,
            0b11 => Self::Ic2,
            _ => unreachable!(),
        }
    }

    const fn frequency(&self) -> Rate {
        const HSI: Rate = Rate::MHz(64);

        match self {
            Self::Hsi => HSI,
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        Rate,
        rcc::{pll_divider_rates, pll_ideal_input_divider},
    };

    use super::ClockConfig;

    #[test]
    fn test_clock_config() {
        let config = ClockConfig::DEFAULT;

        assert_eq!(config.sys_bus_ck().to_Hz(), 64_000_000);
        assert_eq!(config.sys_bus2_ck().to_Hz(), 32_000_000);
        assert_eq!(config.timg_ck().to_Hz(), 64_000_000);
    }

    #[test]
    fn ideal_input_rate() {
        assert_eq!(pll_ideal_input_divider(Rate::MHz(64), Rate::MHz(800)), 2);
        assert_eq!(pll_ideal_input_divider(Rate::MHz(64), Rate::MHz(600)), 3);
        assert_eq!(pll_ideal_input_divider(Rate::MHz(64), Rate::MHz(400)), 4);
    }
    #[test]
    fn divider_rates() {
        let test_rates = [
            (Rate::MHz(64), Rate::MHz(800)),
            (Rate::MHz(64), Rate::MHz(600)),
            (Rate::MHz(64), Rate::MHz(400)),
            (Rate::MHz(64), Rate::MHz(200)),
            (Rate::MHz(32), Rate::MHz(800)),
            (Rate::MHz(32), Rate::MHz(600)),
            (Rate::MHz(32), Rate::MHz(400)),
            (Rate::MHz(32), Rate::MHz(200)),
        ];

        for (input_rate, target_rate) in test_rates {
            let (divm, divn, fracdiv) = pll_divider_rates(input_rate, target_rate);
            assert!(divm >= 1);
            assert!(divn >= 20);
            assert!(divn <= 320);
            let two_24 = 16777216.0; // 2^24
            let input_rate_f64 = input_rate.raw() as f64;
            let (divm, divn, fracdiv) = (divm as f64, divn as f64, fracdiv as f64);
            let fvco = input_rate_f64 * (divn + fracdiv / two_24) / divm;
            assert!(
                (fvco - target_rate.raw() as f64).abs() < 1e-3,
                "{divm}, {divn} {fracdiv}"
            );
        }
    }
}
