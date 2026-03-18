//! Reset and clock control, see Section 14 of RM0486.

use stm32n6::stm32n657::RCC;

use crate::Rate;

pub struct Rcc(RCC);

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

    pub fn reset(&self, peripheral: Peripheral) {
        peripheral.reset(&self.0);
    }
}

macro_rules! impl_peripheral {
    ($(($ensr:ident, $rstsr:ident, $rstcr:ident) => [
        $(($peripheral:ident, $ens:ident, $rsts:ident, $rstc:ident),)*
    ],)*) => {
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

            fn reset(&self, rcc: &RCC) {
                match self {
                    $($(
                        Self::$peripheral => {
                            rcc.$rstsr().write(|w| w.$rsts().set_bit());
                            rcc.$rstcr().write(|w| w.$rstc().set_bit());
                        }
                    )*)*
                }
            }
        }
    };
}

impl_peripheral!(
    (ahb4ensr, ahb4rstsr, ahb4rstcr) => [
        (GpioC, gpiocens, gpiocrsts, gpiocrstc),
        (GpioG, gpiogens, gpiogrsts, gpiogrstc),
    ],
    (ahb5ensr, ahb5rstsr, ahb5rstcr) => [
        (Otg1, otg1ens, otg1rsts, otg1rstc),
    ],
    (apb1lensr, apb1lrstsr, apb1lrstcr) => [
        (Tim6, tim6ens, tim6rsts, tim6rstc),
        (Tim7, tim7ens, tim7rsts, tim7rstc),
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
    Hsi,
    /// sysb_ck = msi_ck
    Msi,
    /// sysb_ck = hse_ck
    Hse,
    /// sysb_ck = ic2_ck
    Ic2,
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
    use super::ClockConfig;

    #[test]
    fn test_clock_config() {
        let config = ClockConfig::DEFAULT;

        assert_eq!(config.sys_bus_ck().to_Hz(), 64_000_000);
        assert_eq!(config.sys_bus2_ck().to_Hz(), 32_000_000);
        assert_eq!(config.timg_ck().to_Hz(), 64_000_000);
    }
}
