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

#[derive(Debug)]
struct DividerRates {
    timer_divider: u8,
    input_divider: u8,
    integer_divider: u16,
    frac_divider: u32,
    ppre1: u8,
}

/// Returns divm, divn and divnfrac
fn pll_divider_rates(input_rate: Rate, target_rate: Rate) -> DividerRates {
    let input_divider = pll_ideal_input_divider(input_rate, target_rate);
    let integer_divider = (target_rate * input_divider as u32) / input_rate;
    let frac_divider = ((target_rate.raw() as u64 * input_divider as u64) << 24)
        / (input_rate.raw() as u64)
        - ((integer_divider as u64) << 24);
    // Set timpre to run timers close to 64MHz
    let timer_prescaler_need = target_rate / SystemClock::Hsi.frequency();
    let timpre = match timer_prescaler_need {
        0 | 1 => 0b00u8,
        2 => 0b01,
        3 | 4 => 0b10,
        _ => 0b11,
    };

    let timer_bus_prescaler_need = target_rate / SystemClock::Hsi.frequency();
    let ppre1 = match timer_bus_prescaler_need {
        0 | 1 => 0b000u8,
        2 => 0b001,
        3..=4 => 0b010,
        5..=8 => 0b011,
        9..=16 => 0b100,
        17..=32 => 0b101,
        33..=64 => 0b110,
        65.. => 0b111,
    };

    DividerRates {
        timer_divider: timpre,
        input_divider,
        integer_divider: integer_divider.try_into().unwrap(),
        frac_divider: frac_divider.try_into().unwrap(),
        ppre1,
    }
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
        let ic2 = self.0.ic2cfgr().read();
        let pll1_cfg = self.0.pll1cfgr1().read();
        let pll1_cfg2 = self.0.pll1cfgr2().read();

        let system_clock = SystemClock::from_bits(cfgr1.syssws().bits());

        let prescaler_ahb = cfgr2.hpre().bits();
        let prescaler_timer = cfgr2.timpre().bits();

        let pll1_input_divider = pll1_cfg.pll1divm().bits() as u64;
        let pll1_integer_multiplier = pll1_cfg.pll1divn().bits() as u64;
        let pll1_frac_multiplier = pll1_cfg2.pll1divnfrac().bits() as u64;

        let config = ClockConfig {
            system_clock,
            prescaler_ahb,
            prescaler_timer,
            divider_ic2: ic2.ic2int().bits(),
            input_ic2: Pll::from_bits(ic2.ic2sel().bits()),
            pll1_input_clock: SystemClock::from_bits(pll1_cfg.pll1sel().bits()),
            pll1_multiplier: ((pll1_frac_multiplier / (1 << 24) + pll1_integer_multiplier)
                / pll1_input_divider) as _,
        };
        debug!("{config:?}");
        config
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

    pub fn enable_pll1(&self, target_rate: Rate) {
        self.0.ccr().write(|w| w.pll1onc().set_bit());

        let hsi_rate = SystemClock::Hsi.frequency();
        let divider_rates = pll_divider_rates(hsi_rate, target_rate);
        debug_now!("{divider_rates:?}");
        self.0.pll1cfgr1().modify(|_, w| unsafe {
            w.pll1divm()
                .bits(divider_rates.input_divider)
                .pll1divn()
                .bits(divider_rates.integer_divider)
                .pll1sel()
                .bits(SystemClock::Hsi as u8)
        });
        self.0.pll1cfgr2().modify(|_, w| unsafe {
            w.pll1divnfrac()
                .bits((divider_rates.frac_divider >> 24) as u32)
        });
        self.0.pll1cfgr3().modify(|_, w| unsafe {
            w.pll1moddsen()
                .set_bit()
                .pll1dacen()
                .set_bit()
                .pll1dacen()
                .set_bit()
                .pll1modssrst()
                .set_bit()
                .pll1pdiven()
                .set_bit()
                .pll1pdiv1()
                .bits(1)
                .pll1pdiv2()
                .bits(1)
        });

        self.0.cfgr2().modify(|_, w| unsafe {
            w.timpre()
                .bits(divider_rates.timer_divider)
                .ppre1()
                .bits(divider_rates.ppre1)
                .ppre2()
                .bits(divider_rates.ppre1)
                .ppre4()
                .bits(divider_rates.ppre1)
                .ppre5()
                .bits(divider_rates.ppre1)
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
        self.0.ic2cfgr().write(|w| unsafe {
            // Select PLL1 output for IC2
            w.ic2sel()
                .bits(0b00)
                // Set divider to 1 (PLL outout straight to ahb)h
                .ic2int()
                .bits(0)
        });
        self.0.ic6cfgr().write(|w| unsafe {
            // Select PLL1 output for IC6
            w.ic6sel()
                .bits(0b00)
                // Set divider to max (outputs to NPU)
                .ic6int()
                .bits(0xFF)
        });
        self.0.ic11cfgr().write(|w| unsafe {
            // Select PLL1 output for IC11
            w.ic11sel()
                .bits(0b00)
                // Set divider to max (outputs to NPU)
                .ic11int()
                .bits(0xFF)
        });
        // Enable IC1 (used then as sys_cpu_ck)
        // Enable IC2 (used then as sysb_ck) // AHB bus
        // Enable IC6 (used then as sysc_ck), only used in NPU
        // Enable IC11 (used then as sysd_ck), only used in NPU
        self.0.divenr().modify(|_, w| {
            w.ic1en()
                .set_bit()
                .ic2en()
                .set_bit()
                .ic6en()
                .set_bit()
                .ic11en()
                .set_bit()
        });

        // Select CPU clock source and system clock to be PLL1
        self.0
            .cfgr1()
            .modify(|_, w| unsafe { w.cpusw().bits(0b11).syssw().bits(0b11) });

        debug_now!("Waiting for cpsws");
        while self.0.cfgr1().read().cpusws().bits() != 0b11 {}
        debug_now!("Waiting for syssws");
        while self.0.cfgr1().read().syssws().bits() != 0b11 {}
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

    /// Starts the HSE in digital bypass mode (external oscillator) and waits until it is ready.
    pub fn enable_hse_bypass_digital(&self) {
        self.0.cr().modify(|_, w| w.hseon().clear_bit());
        self.0
            .hsecfgr()
            .modify(|_, w| w.hsebyp().set_bit().hseext().set_bit());
        self.0.cr().modify(|_, w| w.hseon().set_bit());
        while self.0.sr().read().hserdy().bit_is_clear() {}
    }

    /// Feeds the OTGPHY1 reference input with the HSE divided by two (see Section 14.7).
    ///
    /// Must be called while the OTGPHY1 clock is disabled.
    pub fn select_otgphy1_hse_div2(&self) {
        // The PAC names HSEDIV2SEL hsediv2byp.
        self.0.hsecfgr().modify(|_, w| w.hsediv2byp().set_bit());
        self.0.ccipr6().modify(|_, w| {
            // SAFETY: 0b00 selects hse_div2_ck for the kernel clock mux.
            unsafe { w.otgphy1sel().bits(0) }
                .otgphy1ckrefsel()
                .set_bit()
        });
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
        (GpioP, gpiopens, gpioprsts, gpioprstc),
    ],
    (ahb5ensr, ahb5rstsr, ahb5rstcr) => [
        (Otg1, otg1ens, otg1rsts, otg1rstc),
        (OtgPhy1, otgphy1ens, otgphy1rsts, otgphy1rstc),
        (Sdmmc1, sdmmc1ens, sdmmc1rsts, sdmmc1rstc),
        (Sdmmc2, sdmmc2ens, sdmmc2rsts, sdmmc2rstc),
    ],
    (apb1lensr, apb1lrstsr, apb1lrstcr) => [
        (Tim6, tim6ens, tim6rsts, tim6rstc),
        (Tim7, tim7ens, tim7rsts, tim7rstc),
    ],
    (apb4hensr, apb4hrstsr, apb4hrstcr) => [
        (Syscfg, syscfgens, syscfgrsts, syscfgrstc),
    ],
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pll {
    Pll1 = 0b00,
    Pll2 = 0b01,
    Pll3 = 0b10,
    Pll4 = 0b11,
}

impl Pll {
    const fn from_bits(value: u8) -> Self {
        match value {
            0b00 => Self::Pll1,
            0b01 => Self::Pll2,
            0b10 => Self::Pll3,
            0b11 => Self::Pll4,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClockConfig {
    pub system_clock: SystemClock,
    pub divider_ic2: u8,
    pub input_ic2: Pll,
    pub pll1_input_clock: SystemClock,
    pub pll1_multiplier: u32,
    pub prescaler_ahb: u8,
    pub prescaler_timer: u8,
}

impl ClockConfig {
    pub const DEFAULT: Self = Self {
        system_clock: SystemClock::Hsi,
        prescaler_ahb: 1,
        prescaler_timer: 0,
        input_ic2: Pll::Pll1,
        divider_ic2: 0,
        pll1_input_clock: SystemClock::Hsi,
        pll1_multiplier: 0,
    };

    pub fn sys_bus_ck(&self) -> Rate {
        const HSI: Rate = Rate::MHz(64);

        match self.system_clock {
            SystemClock::Hsi => HSI,
            SystemClock::Ic2 => {
                if !matches!(self.input_ic2, Pll::Pll1) {
                    unimplemented!();
                }

                self.pll1_input_clock.frequency() * self.pll1_multiplier
                    / (self.divider_ic2 as u32 + 1)
            }
            _ => unimplemented!(),
        }
    }

    pub fn sys_bus2_ck(&self) -> Rate {
        let rate = scale(self.sys_bus_ck(), self.prescaler_ahb);
        debug!("{rate:?}");
        rate
    }

    pub fn timg_ck(&self) -> Rate {
        let rate = scale(self.sys_bus_ck(), self.prescaler_timer);
        debug!("{rate:?}");
        rate
    }
}

const fn scale(f: Rate, prescaler: u8) -> Rate {
    let mut frequency = f.raw();
    let mut i = 0;
    while i < prescaler {
        assert!(frequency.is_multiple_of(2));
        frequency /= 2;
        i += 1;
    }
    Rate::from_raw(frequency)
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
        rcc::{Pll, SystemClock, pll_divider_rates, pll_ideal_input_divider},
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
            let divider_rates = pll_divider_rates(input_rate, target_rate);
            assert!(divider_rates.input_divider >= 1);
            assert!(divider_rates.integer_divider >= 20);
            assert!(divider_rates.integer_divider <= 320);
            let two_24 = 16777216.0; // 2^24
            let input_rate_f64 = input_rate.raw() as f64;
            let (divm, divn, fracdiv) = (
                divider_rates.input_divider as f64,
                divider_rates.integer_divider as f64,
                divider_rates.frac_divider as f64,
            );
            let pll1_multiplier = (divn + fracdiv / two_24) / divm;
            let fvco = input_rate_f64 * pll1_multiplier;
            assert!(
                (fvco - target_rate.raw() as f64).abs() < 1e-3,
                "{divm}, {divn} {fracdiv}"
            );

            if input_rate != Rate::MHz(64) {
                continue;
            }

            let clock_config = ClockConfig {
                system_clock: SystemClock::Ic2,
                divider_ic2: 0,
                input_ic2: Pll::Pll1,
                pll1_input_clock: SystemClock::Hsi,
                pll1_multiplier: pll1_multiplier as u32,
                prescaler_ahb: 1,
                prescaler_timer: divider_rates.timer_divider,
            };
            assert!((clock_config.timg_ck().raw() as f64 - 64e6).abs() <= 32e6);
        }
    }
}
