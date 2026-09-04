#![allow(non_snake_case)]

pub mod instances;
pub mod peripherals;
pub mod register;
pub mod stm32f429;

pub use crate::{modify_reg, read_reg, write_reg};

pub mod otg_global {
    #[cfg(feature = "fs")]
    pub use super::stm32f429::otg_fs_global::OTG_FS_GLOBAL as OTG_GLOBAL;
    #[cfg(feature = "fs")]
    pub use super::stm32f429::otg_fs_global::*;
    #[cfg(feature = "hs")]
    pub use super::stm32f429::otg_hs_global::OTG_HS_GLOBAL as OTG_GLOBAL;
    #[cfg(feature = "hs")]
    pub use super::stm32f429::otg_hs_global::*;
}

pub mod otg_device {
    #[cfg(feature = "fs")]
    pub use super::stm32f429::otg_fs_device::OTG_FS_DEVICE as OTG_DEVICE;
    #[cfg(feature = "fs")]
    pub use super::stm32f429::otg_fs_device::*;
    #[cfg(feature = "hs")]
    pub use super::stm32f429::otg_hs_device::OTG_HS_DEVICE as OTG_DEVICE;
    #[cfg(feature = "hs")]
    pub use super::stm32f429::otg_hs_device::*;
}

pub mod otg_pwrclk {
    #[cfg(feature = "fs")]
    pub use super::stm32f429::otg_s_pwrclk::OTG_FS_PWRCLK as OTG_PWRCLK;
    #[cfg(feature = "hs")]
    pub use super::stm32f429::otg_s_pwrclk::OTG_HS_PWRCLK as OTG_PWRCLK;
    pub use super::stm32f429::otg_s_pwrclk::*;
}

pub mod otg_global_dieptxfx {
    use super::register::RWRegister;

    #[cfg(feature = "fs")]
    pub use super::stm32f429::otg_fs_global::DIEPTXF1 as DIEPTXFx;
    #[cfg(feature = "hs")]
    pub use super::stm32f429::otg_fs_global::DIEPTXF1 as DIEPTXFx;

    pub struct RegisterBlock {
        pub DIEPTXFx: RWRegister<u32>,
    }
}

pub mod endpoint_in {
    use super::register::RWRegister;

    #[cfg(feature = "fs")]
    pub use super::stm32f429::otg_fs_device::{
        DIEPCTL1 as DIEPCTL, DIEPINT1 as DIEPINT, DIEPTSIZ1 as DIEPTSIZ, DTXFSTS1 as DTXFSTS,
    };

    #[cfg(feature = "hs")]
    pub use super::stm32f429::otg_hs_device::{
        DIEPCTL1 as DIEPCTL, DIEPINT1 as DIEPINT, DIEPTSIZ1 as DIEPTSIZ, DTXFSTS1 as DTXFSTS,
    };

    pub struct RegisterBlock {
        pub DIEPCTL: RWRegister<u32>,
        _reserved0: u32,
        pub DIEPINT: RWRegister<u32>,
        _reserved1: u32,
        pub DIEPTSIZ: RWRegister<u32>,
        _reserved2: u32,
        pub DTXFSTS: RWRegister<u32>,
        _reserved3: u32,
    }
}

pub mod endpoint0_out {
    use super::register::RWRegister;

    #[cfg(feature = "fs")]
    pub use super::stm32f429::otg_fs_device::{DOEPCTL0, DOEPINT0, DOEPTSIZ0};

    #[cfg(feature = "hs")]
    pub use super::stm32f429::otg_hs_device::{DOEPCTL0, DOEPINT0, DOEPTSIZ0};

    pub struct RegisterBlock {
        pub DOEPCTL0: RWRegister<u32>,
        _reserved0: u32,
        pub DOEPINT0: RWRegister<u32>,
        _reserved1: u32,
        pub DOEPTSIZ0: RWRegister<u32>,
        _reserved2: [u32; 3],
    }
}

pub mod endpoint_out {
    use super::register::RWRegister;

    #[cfg(feature = "fs")]
    pub use super::stm32f429::otg_fs_device::{
        DOEPCTL1 as DOEPCTL, DOEPINT1 as DOEPINT, DOEPTSIZ1 as DOEPTSIZ,
    };

    #[cfg(feature = "hs")]
    pub use super::stm32f429::otg_hs_device::{
        DOEPCTL1 as DOEPCTL, DOEPINT1 as DOEPINT, DOEPTSIZ1 as DOEPTSIZ,
    };

    pub struct RegisterBlock {
        pub DOEPCTL: RWRegister<u32>,
        _reserved0: u32,
        pub DOEPINT: RWRegister<u32>,
        _reserved1: u32,
        pub DOEPTSIZ: RWRegister<u32>,
        _reserved2: [u32; 3],
    }
}
