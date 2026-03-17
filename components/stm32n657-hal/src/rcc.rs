//! Reset and clock control, see Section 14 of RM0486.

use stm32n6::stm32n657::RCC;

use crate::Rate;

pub struct Rcc(RCC);

impl Rcc {
    pub fn new(rcc: RCC) -> Self {
        Self(rcc)
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
        match peripheral {
            Peripheral::GpioC => self.0.ahb4ensr().write(|w| w.gpiocens().set_bit()),
            Peripheral::GpioG => self.0.ahb4ensr().write(|w| w.gpiogens().set_bit()),
            Peripheral::Otg1 => self.0.ahb5ensr().write(|w| w.otg1ens().set_bit()),
            Peripheral::Rtc => self.0.apb4lensr().write(|w| w.rtcens().set_bit()),
            Peripheral::Tim6 => self.0.apb1lensr().write(|w| w.tim6ens().set_bit()),
            Peripheral::Tim7 => self.0.apb1lensr().write(|w| w.tim7ens().set_bit()),
        };
    }
}

pub enum Peripheral {
    GpioC,
    GpioG,
    Otg1,
    Rtc,
    Tim6,
    Tim7,
}

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
