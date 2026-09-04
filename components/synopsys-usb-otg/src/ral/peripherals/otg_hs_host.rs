#![allow(non_snake_case, non_upper_case_globals)]
#![allow(non_camel_case_types)]
//! USB on the go high speed
//!
//! Used by: stm32f405, stm32f407, stm32f427, stm32f429, stm32f446, stm32f469

use super::super::register::{RORegister, RWRegister, UnsafeRWRegister};
#[cfg(not(feature = "nosync"))]
use core::marker::PhantomData;

/// OTG_HS host configuration register
pub mod HCFG {

    /// FS/LS PHY clock select
    pub mod FSLSPCS {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (2 bits: 0b11 << 0)
        pub const mask: u32 = 0b11 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// FS- and LS-only support
    pub mod FSLSS {
        /// Offset (2 bits)
        pub const offset: u32 = 2;
        /// Mask (1 bit: 1 << 2)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS Host frame interval register
pub mod HFIR {

    /// Frame interval
    pub mod FRIVL {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (16 bits: 0xffff << 0)
        pub const mask: u32 = 0xffff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS host frame number/frame time remaining register
pub mod HFNUM {

    /// Frame number
    pub mod FRNUM {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (16 bits: 0xffff << 0)
        pub const mask: u32 = 0xffff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Frame time remaining
    pub mod FTREM {
        /// Offset (16 bits)
        pub const offset: u32 = 16;
        /// Mask (16 bits: 0xffff << 16)
        pub const mask: u32 = 0xffff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS_Host periodic transmit FIFO/queue status register
pub mod HPTXSTS {

    /// Periodic transmit data FIFO space available
    pub mod PTXFSAVL {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (16 bits: 0xffff << 0)
        pub const mask: u32 = 0xffff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Periodic transmit request queue space available
    pub mod PTXQSAV {
        /// Offset (16 bits)
        pub const offset: u32 = 16;
        /// Mask (8 bits: 0xff << 16)
        pub const mask: u32 = 0xff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Top of the periodic transmit request queue
    pub mod PTXQTOP {
        /// Offset (24 bits)
        pub const offset: u32 = 24;
        /// Mask (8 bits: 0xff << 24)
        pub const mask: u32 = 0xff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS Host all channels interrupt register
pub mod HAINT {

    /// Channel interrupts
    pub mod HAINT {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (16 bits: 0xffff << 0)
        pub const mask: u32 = 0xffff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS host all channels interrupt mask register
pub mod HAINTMSK {

    /// Channel interrupt mask
    pub mod HAINTM {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (16 bits: 0xffff << 0)
        pub const mask: u32 = 0xffff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS host port control and status register
pub mod HPRT {

    /// Port connect status
    pub mod PCSTS {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (1 bit: 1 << 0)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Port connect detected
    pub mod PCDET {
        /// Offset (1 bits)
        pub const offset: u32 = 1;
        /// Mask (1 bit: 1 << 1)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Port enable
    pub mod PENA {
        /// Offset (2 bits)
        pub const offset: u32 = 2;
        /// Mask (1 bit: 1 << 2)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Port enable/disable change
    pub mod PENCHNG {
        /// Offset (3 bits)
        pub const offset: u32 = 3;
        /// Mask (1 bit: 1 << 3)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Port overcurrent active
    pub mod POCA {
        /// Offset (4 bits)
        pub const offset: u32 = 4;
        /// Mask (1 bit: 1 << 4)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Port overcurrent change
    pub mod POCCHNG {
        /// Offset (5 bits)
        pub const offset: u32 = 5;
        /// Mask (1 bit: 1 << 5)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Port resume
    pub mod PRES {
        /// Offset (6 bits)
        pub const offset: u32 = 6;
        /// Mask (1 bit: 1 << 6)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Port suspend
    pub mod PSUSP {
        /// Offset (7 bits)
        pub const offset: u32 = 7;
        /// Mask (1 bit: 1 << 7)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Port reset
    pub mod PRST {
        /// Offset (8 bits)
        pub const offset: u32 = 8;
        /// Mask (1 bit: 1 << 8)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Port line status
    pub mod PLSTS {
        /// Offset (10 bits)
        pub const offset: u32 = 10;
        /// Mask (2 bits: 0b11 << 10)
        pub const mask: u32 = 0b11 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Port power
    pub mod PPWR {
        /// Offset (12 bits)
        pub const offset: u32 = 12;
        /// Mask (1 bit: 1 << 12)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Port test control
    pub mod PTCTL {
        /// Offset (13 bits)
        pub const offset: u32 = 13;
        /// Mask (4 bits: 0b1111 << 13)
        pub const mask: u32 = 0b1111 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Port speed
    pub mod PSPD {
        /// Offset (17 bits)
        pub const offset: u32 = 17;
        /// Mask (2 bits: 0b11 << 17)
        pub const mask: u32 = 0b11 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS host channel-0 characteristics register
pub mod HCCHAR0 {

    /// Maximum packet size
    pub mod MPSIZ {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (11 bits: 0x7ff << 0)
        pub const mask: u32 = 0x7ff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Endpoint number
    pub mod EPNUM {
        /// Offset (11 bits)
        pub const offset: u32 = 11;
        /// Mask (4 bits: 0b1111 << 11)
        pub const mask: u32 = 0b1111 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Endpoint direction
    pub mod EPDIR {
        /// Offset (15 bits)
        pub const offset: u32 = 15;
        /// Mask (1 bit: 1 << 15)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Low-speed device
    pub mod LSDEV {
        /// Offset (17 bits)
        pub const offset: u32 = 17;
        /// Mask (1 bit: 1 << 17)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Endpoint type
    pub mod EPTYP {
        /// Offset (18 bits)
        pub const offset: u32 = 18;
        /// Mask (2 bits: 0b11 << 18)
        pub const mask: u32 = 0b11 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Multi Count (MC) / Error Count (EC)
    pub mod MC {
        /// Offset (20 bits)
        pub const offset: u32 = 20;
        /// Mask (2 bits: 0b11 << 20)
        pub const mask: u32 = 0b11 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Device address
    pub mod DAD {
        /// Offset (22 bits)
        pub const offset: u32 = 22;
        /// Mask (7 bits: 0x7f << 22)
        pub const mask: u32 = 0x7f << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Odd frame
    pub mod ODDFRM {
        /// Offset (29 bits)
        pub const offset: u32 = 29;
        /// Mask (1 bit: 1 << 29)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Channel disable
    pub mod CHDIS {
        /// Offset (30 bits)
        pub const offset: u32 = 30;
        /// Mask (1 bit: 1 << 30)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Channel enable
    pub mod CHENA {
        /// Offset (31 bits)
        pub const offset: u32 = 31;
        /// Mask (1 bit: 1 << 31)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS host channel-1 characteristics register
pub mod HCCHAR1 {
    pub use super::HCCHAR0::CHDIS;
    pub use super::HCCHAR0::CHENA;
    pub use super::HCCHAR0::DAD;
    pub use super::HCCHAR0::EPDIR;
    pub use super::HCCHAR0::EPNUM;
    pub use super::HCCHAR0::EPTYP;
    pub use super::HCCHAR0::LSDEV;
    pub use super::HCCHAR0::MC;
    pub use super::HCCHAR0::MPSIZ;
    pub use super::HCCHAR0::ODDFRM;
}

/// OTG_HS host channel-2 characteristics register
pub mod HCCHAR2 {
    pub use super::HCCHAR0::CHDIS;
    pub use super::HCCHAR0::CHENA;
    pub use super::HCCHAR0::DAD;
    pub use super::HCCHAR0::EPDIR;
    pub use super::HCCHAR0::EPNUM;
    pub use super::HCCHAR0::EPTYP;
    pub use super::HCCHAR0::LSDEV;
    pub use super::HCCHAR0::MC;
    pub use super::HCCHAR0::MPSIZ;
    pub use super::HCCHAR0::ODDFRM;
}

/// OTG_HS host channel-3 characteristics register
pub mod HCCHAR3 {
    pub use super::HCCHAR0::CHDIS;
    pub use super::HCCHAR0::CHENA;
    pub use super::HCCHAR0::DAD;
    pub use super::HCCHAR0::EPDIR;
    pub use super::HCCHAR0::EPNUM;
    pub use super::HCCHAR0::EPTYP;
    pub use super::HCCHAR0::LSDEV;
    pub use super::HCCHAR0::MC;
    pub use super::HCCHAR0::MPSIZ;
    pub use super::HCCHAR0::ODDFRM;
}

/// OTG_HS host channel-4 characteristics register
pub mod HCCHAR4 {
    pub use super::HCCHAR0::CHDIS;
    pub use super::HCCHAR0::CHENA;
    pub use super::HCCHAR0::DAD;
    pub use super::HCCHAR0::EPDIR;
    pub use super::HCCHAR0::EPNUM;
    pub use super::HCCHAR0::EPTYP;
    pub use super::HCCHAR0::LSDEV;
    pub use super::HCCHAR0::MC;
    pub use super::HCCHAR0::MPSIZ;
    pub use super::HCCHAR0::ODDFRM;
}

/// OTG_HS host channel-5 characteristics register
pub mod HCCHAR5 {
    pub use super::HCCHAR0::CHDIS;
    pub use super::HCCHAR0::CHENA;
    pub use super::HCCHAR0::DAD;
    pub use super::HCCHAR0::EPDIR;
    pub use super::HCCHAR0::EPNUM;
    pub use super::HCCHAR0::EPTYP;
    pub use super::HCCHAR0::LSDEV;
    pub use super::HCCHAR0::MC;
    pub use super::HCCHAR0::MPSIZ;
    pub use super::HCCHAR0::ODDFRM;
}

/// OTG_HS host channel-6 characteristics register
pub mod HCCHAR6 {
    pub use super::HCCHAR0::CHDIS;
    pub use super::HCCHAR0::CHENA;
    pub use super::HCCHAR0::DAD;
    pub use super::HCCHAR0::EPDIR;
    pub use super::HCCHAR0::EPNUM;
    pub use super::HCCHAR0::EPTYP;
    pub use super::HCCHAR0::LSDEV;
    pub use super::HCCHAR0::MC;
    pub use super::HCCHAR0::MPSIZ;
    pub use super::HCCHAR0::ODDFRM;
}

/// OTG_HS host channel-7 characteristics register
pub mod HCCHAR7 {
    pub use super::HCCHAR0::CHDIS;
    pub use super::HCCHAR0::CHENA;
    pub use super::HCCHAR0::DAD;
    pub use super::HCCHAR0::EPDIR;
    pub use super::HCCHAR0::EPNUM;
    pub use super::HCCHAR0::EPTYP;
    pub use super::HCCHAR0::LSDEV;
    pub use super::HCCHAR0::MC;
    pub use super::HCCHAR0::MPSIZ;
    pub use super::HCCHAR0::ODDFRM;
}

/// OTG_HS host channel-8 characteristics register
pub mod HCCHAR8 {
    pub use super::HCCHAR0::CHDIS;
    pub use super::HCCHAR0::CHENA;
    pub use super::HCCHAR0::DAD;
    pub use super::HCCHAR0::EPDIR;
    pub use super::HCCHAR0::EPNUM;
    pub use super::HCCHAR0::EPTYP;
    pub use super::HCCHAR0::LSDEV;
    pub use super::HCCHAR0::MC;
    pub use super::HCCHAR0::MPSIZ;
    pub use super::HCCHAR0::ODDFRM;
}

/// OTG_HS host channel-9 characteristics register
pub mod HCCHAR9 {
    pub use super::HCCHAR0::CHDIS;
    pub use super::HCCHAR0::CHENA;
    pub use super::HCCHAR0::DAD;
    pub use super::HCCHAR0::EPDIR;
    pub use super::HCCHAR0::EPNUM;
    pub use super::HCCHAR0::EPTYP;
    pub use super::HCCHAR0::LSDEV;
    pub use super::HCCHAR0::MC;
    pub use super::HCCHAR0::MPSIZ;
    pub use super::HCCHAR0::ODDFRM;
}

/// OTG_HS host channel-10 characteristics register
pub mod HCCHAR10 {
    pub use super::HCCHAR0::CHDIS;
    pub use super::HCCHAR0::CHENA;
    pub use super::HCCHAR0::DAD;
    pub use super::HCCHAR0::EPDIR;
    pub use super::HCCHAR0::EPNUM;
    pub use super::HCCHAR0::EPTYP;
    pub use super::HCCHAR0::LSDEV;
    pub use super::HCCHAR0::MC;
    pub use super::HCCHAR0::MPSIZ;
    pub use super::HCCHAR0::ODDFRM;
}

/// OTG_HS host channel-11 characteristics register
pub mod HCCHAR11 {
    pub use super::HCCHAR0::CHDIS;
    pub use super::HCCHAR0::CHENA;
    pub use super::HCCHAR0::DAD;
    pub use super::HCCHAR0::EPDIR;
    pub use super::HCCHAR0::EPNUM;
    pub use super::HCCHAR0::EPTYP;
    pub use super::HCCHAR0::LSDEV;
    pub use super::HCCHAR0::MC;
    pub use super::HCCHAR0::MPSIZ;
    pub use super::HCCHAR0::ODDFRM;
}

/// OTG_HS host channel-0 split control register
pub mod HCSPLT0 {

    /// Port address
    pub mod PRTADDR {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (7 bits: 0x7f << 0)
        pub const mask: u32 = 0x7f << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Hub address
    pub mod HUBADDR {
        /// Offset (7 bits)
        pub const offset: u32 = 7;
        /// Mask (7 bits: 0x7f << 7)
        pub const mask: u32 = 0x7f << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// XACTPOS
    pub mod XACTPOS {
        /// Offset (14 bits)
        pub const offset: u32 = 14;
        /// Mask (2 bits: 0b11 << 14)
        pub const mask: u32 = 0b11 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Do complete split
    pub mod COMPLSPLT {
        /// Offset (16 bits)
        pub const offset: u32 = 16;
        /// Mask (1 bit: 1 << 16)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Split enable
    pub mod SPLITEN {
        /// Offset (31 bits)
        pub const offset: u32 = 31;
        /// Mask (1 bit: 1 << 31)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS host channel-1 split control register
pub mod HCSPLT1 {
    pub use super::HCSPLT0::COMPLSPLT;
    pub use super::HCSPLT0::HUBADDR;
    pub use super::HCSPLT0::PRTADDR;
    pub use super::HCSPLT0::SPLITEN;
    pub use super::HCSPLT0::XACTPOS;
}

/// OTG_HS host channel-2 split control register
pub mod HCSPLT2 {
    pub use super::HCSPLT0::COMPLSPLT;
    pub use super::HCSPLT0::HUBADDR;
    pub use super::HCSPLT0::PRTADDR;
    pub use super::HCSPLT0::SPLITEN;
    pub use super::HCSPLT0::XACTPOS;
}

/// OTG_HS host channel-3 split control register
pub mod HCSPLT3 {
    pub use super::HCSPLT0::COMPLSPLT;
    pub use super::HCSPLT0::HUBADDR;
    pub use super::HCSPLT0::PRTADDR;
    pub use super::HCSPLT0::SPLITEN;
    pub use super::HCSPLT0::XACTPOS;
}

/// OTG_HS host channel-4 split control register
pub mod HCSPLT4 {
    pub use super::HCSPLT0::COMPLSPLT;
    pub use super::HCSPLT0::HUBADDR;
    pub use super::HCSPLT0::PRTADDR;
    pub use super::HCSPLT0::SPLITEN;
    pub use super::HCSPLT0::XACTPOS;
}

/// OTG_HS host channel-5 split control register
pub mod HCSPLT5 {
    pub use super::HCSPLT0::COMPLSPLT;
    pub use super::HCSPLT0::HUBADDR;
    pub use super::HCSPLT0::PRTADDR;
    pub use super::HCSPLT0::SPLITEN;
    pub use super::HCSPLT0::XACTPOS;
}

/// OTG_HS host channel-6 split control register
pub mod HCSPLT6 {
    pub use super::HCSPLT0::COMPLSPLT;
    pub use super::HCSPLT0::HUBADDR;
    pub use super::HCSPLT0::PRTADDR;
    pub use super::HCSPLT0::SPLITEN;
    pub use super::HCSPLT0::XACTPOS;
}

/// OTG_HS host channel-7 split control register
pub mod HCSPLT7 {
    pub use super::HCSPLT0::COMPLSPLT;
    pub use super::HCSPLT0::HUBADDR;
    pub use super::HCSPLT0::PRTADDR;
    pub use super::HCSPLT0::SPLITEN;
    pub use super::HCSPLT0::XACTPOS;
}

/// OTG_HS host channel-8 split control register
pub mod HCSPLT8 {
    pub use super::HCSPLT0::COMPLSPLT;
    pub use super::HCSPLT0::HUBADDR;
    pub use super::HCSPLT0::PRTADDR;
    pub use super::HCSPLT0::SPLITEN;
    pub use super::HCSPLT0::XACTPOS;
}

/// OTG_HS host channel-9 split control register
pub mod HCSPLT9 {
    pub use super::HCSPLT0::COMPLSPLT;
    pub use super::HCSPLT0::HUBADDR;
    pub use super::HCSPLT0::PRTADDR;
    pub use super::HCSPLT0::SPLITEN;
    pub use super::HCSPLT0::XACTPOS;
}

/// OTG_HS host channel-10 split control register
pub mod HCSPLT10 {
    pub use super::HCSPLT0::COMPLSPLT;
    pub use super::HCSPLT0::HUBADDR;
    pub use super::HCSPLT0::PRTADDR;
    pub use super::HCSPLT0::SPLITEN;
    pub use super::HCSPLT0::XACTPOS;
}

/// OTG_HS host channel-11 split control register
pub mod HCSPLT11 {
    pub use super::HCSPLT0::COMPLSPLT;
    pub use super::HCSPLT0::HUBADDR;
    pub use super::HCSPLT0::PRTADDR;
    pub use super::HCSPLT0::SPLITEN;
    pub use super::HCSPLT0::XACTPOS;
}

/// OTG_HS host channel-11 interrupt register
pub mod HCINT0 {

    /// Transfer completed
    pub mod XFRC {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (1 bit: 1 << 0)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Channel halted
    pub mod CHH {
        /// Offset (1 bits)
        pub const offset: u32 = 1;
        /// Mask (1 bit: 1 << 1)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// AHB error
    pub mod AHBERR {
        /// Offset (2 bits)
        pub const offset: u32 = 2;
        /// Mask (1 bit: 1 << 2)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// STALL response received interrupt
    pub mod STALL {
        /// Offset (3 bits)
        pub const offset: u32 = 3;
        /// Mask (1 bit: 1 << 3)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// NAK response received interrupt
    pub mod NAK {
        /// Offset (4 bits)
        pub const offset: u32 = 4;
        /// Mask (1 bit: 1 << 4)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// ACK response received/transmitted interrupt
    pub mod ACK {
        /// Offset (5 bits)
        pub const offset: u32 = 5;
        /// Mask (1 bit: 1 << 5)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Response received interrupt
    pub mod NYET {
        /// Offset (6 bits)
        pub const offset: u32 = 6;
        /// Mask (1 bit: 1 << 6)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Transaction error
    pub mod TXERR {
        /// Offset (7 bits)
        pub const offset: u32 = 7;
        /// Mask (1 bit: 1 << 7)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Babble error
    pub mod BBERR {
        /// Offset (8 bits)
        pub const offset: u32 = 8;
        /// Mask (1 bit: 1 << 8)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Frame overrun
    pub mod FRMOR {
        /// Offset (9 bits)
        pub const offset: u32 = 9;
        /// Mask (1 bit: 1 << 9)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Data toggle error
    pub mod DTERR {
        /// Offset (10 bits)
        pub const offset: u32 = 10;
        /// Mask (1 bit: 1 << 10)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS host channel-1 interrupt register
pub mod HCINT1 {
    pub use super::HCINT0::ACK;
    pub use super::HCINT0::AHBERR;
    pub use super::HCINT0::BBERR;
    pub use super::HCINT0::CHH;
    pub use super::HCINT0::DTERR;
    pub use super::HCINT0::FRMOR;
    pub use super::HCINT0::NAK;
    pub use super::HCINT0::NYET;
    pub use super::HCINT0::STALL;
    pub use super::HCINT0::TXERR;
    pub use super::HCINT0::XFRC;
}

/// OTG_HS host channel-2 interrupt register
pub mod HCINT2 {
    pub use super::HCINT0::ACK;
    pub use super::HCINT0::AHBERR;
    pub use super::HCINT0::BBERR;
    pub use super::HCINT0::CHH;
    pub use super::HCINT0::DTERR;
    pub use super::HCINT0::FRMOR;
    pub use super::HCINT0::NAK;
    pub use super::HCINT0::NYET;
    pub use super::HCINT0::STALL;
    pub use super::HCINT0::TXERR;
    pub use super::HCINT0::XFRC;
}

/// OTG_HS host channel-3 interrupt register
pub mod HCINT3 {
    pub use super::HCINT0::ACK;
    pub use super::HCINT0::AHBERR;
    pub use super::HCINT0::BBERR;
    pub use super::HCINT0::CHH;
    pub use super::HCINT0::DTERR;
    pub use super::HCINT0::FRMOR;
    pub use super::HCINT0::NAK;
    pub use super::HCINT0::NYET;
    pub use super::HCINT0::STALL;
    pub use super::HCINT0::TXERR;
    pub use super::HCINT0::XFRC;
}

/// OTG_HS host channel-4 interrupt register
pub mod HCINT4 {
    pub use super::HCINT0::ACK;
    pub use super::HCINT0::AHBERR;
    pub use super::HCINT0::BBERR;
    pub use super::HCINT0::CHH;
    pub use super::HCINT0::DTERR;
    pub use super::HCINT0::FRMOR;
    pub use super::HCINT0::NAK;
    pub use super::HCINT0::NYET;
    pub use super::HCINT0::STALL;
    pub use super::HCINT0::TXERR;
    pub use super::HCINT0::XFRC;
}

/// OTG_HS host channel-5 interrupt register
pub mod HCINT5 {
    pub use super::HCINT0::ACK;
    pub use super::HCINT0::AHBERR;
    pub use super::HCINT0::BBERR;
    pub use super::HCINT0::CHH;
    pub use super::HCINT0::DTERR;
    pub use super::HCINT0::FRMOR;
    pub use super::HCINT0::NAK;
    pub use super::HCINT0::NYET;
    pub use super::HCINT0::STALL;
    pub use super::HCINT0::TXERR;
    pub use super::HCINT0::XFRC;
}

/// OTG_HS host channel-6 interrupt register
pub mod HCINT6 {
    pub use super::HCINT0::ACK;
    pub use super::HCINT0::AHBERR;
    pub use super::HCINT0::BBERR;
    pub use super::HCINT0::CHH;
    pub use super::HCINT0::DTERR;
    pub use super::HCINT0::FRMOR;
    pub use super::HCINT0::NAK;
    pub use super::HCINT0::NYET;
    pub use super::HCINT0::STALL;
    pub use super::HCINT0::TXERR;
    pub use super::HCINT0::XFRC;
}

/// OTG_HS host channel-7 interrupt register
pub mod HCINT7 {
    pub use super::HCINT0::ACK;
    pub use super::HCINT0::AHBERR;
    pub use super::HCINT0::BBERR;
    pub use super::HCINT0::CHH;
    pub use super::HCINT0::DTERR;
    pub use super::HCINT0::FRMOR;
    pub use super::HCINT0::NAK;
    pub use super::HCINT0::NYET;
    pub use super::HCINT0::STALL;
    pub use super::HCINT0::TXERR;
    pub use super::HCINT0::XFRC;
}

/// OTG_HS host channel-8 interrupt register
pub mod HCINT8 {
    pub use super::HCINT0::ACK;
    pub use super::HCINT0::AHBERR;
    pub use super::HCINT0::BBERR;
    pub use super::HCINT0::CHH;
    pub use super::HCINT0::DTERR;
    pub use super::HCINT0::FRMOR;
    pub use super::HCINT0::NAK;
    pub use super::HCINT0::NYET;
    pub use super::HCINT0::STALL;
    pub use super::HCINT0::TXERR;
    pub use super::HCINT0::XFRC;
}

/// OTG_HS host channel-9 interrupt register
pub mod HCINT9 {
    pub use super::HCINT0::ACK;
    pub use super::HCINT0::AHBERR;
    pub use super::HCINT0::BBERR;
    pub use super::HCINT0::CHH;
    pub use super::HCINT0::DTERR;
    pub use super::HCINT0::FRMOR;
    pub use super::HCINT0::NAK;
    pub use super::HCINT0::NYET;
    pub use super::HCINT0::STALL;
    pub use super::HCINT0::TXERR;
    pub use super::HCINT0::XFRC;
}

/// OTG_HS host channel-10 interrupt register
pub mod HCINT10 {
    pub use super::HCINT0::ACK;
    pub use super::HCINT0::AHBERR;
    pub use super::HCINT0::BBERR;
    pub use super::HCINT0::CHH;
    pub use super::HCINT0::DTERR;
    pub use super::HCINT0::FRMOR;
    pub use super::HCINT0::NAK;
    pub use super::HCINT0::NYET;
    pub use super::HCINT0::STALL;
    pub use super::HCINT0::TXERR;
    pub use super::HCINT0::XFRC;
}

/// OTG_HS host channel-11 interrupt register
pub mod HCINT11 {
    pub use super::HCINT0::ACK;
    pub use super::HCINT0::AHBERR;
    pub use super::HCINT0::BBERR;
    pub use super::HCINT0::CHH;
    pub use super::HCINT0::DTERR;
    pub use super::HCINT0::FRMOR;
    pub use super::HCINT0::NAK;
    pub use super::HCINT0::NYET;
    pub use super::HCINT0::STALL;
    pub use super::HCINT0::TXERR;
    pub use super::HCINT0::XFRC;
}

/// OTG_HS host channel-11 interrupt mask register
pub mod HCINTMSK0 {

    /// Transfer completed mask
    pub mod XFRCM {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (1 bit: 1 << 0)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Channel halted mask
    pub mod CHHM {
        /// Offset (1 bits)
        pub const offset: u32 = 1;
        /// Mask (1 bit: 1 << 1)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// AHB error
    pub mod AHBERR {
        /// Offset (2 bits)
        pub const offset: u32 = 2;
        /// Mask (1 bit: 1 << 2)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// STALL response received interrupt mask
    pub mod STALLM {
        /// Offset (3 bits)
        pub const offset: u32 = 3;
        /// Mask (1 bit: 1 << 3)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// NAK response received interrupt mask
    pub mod NAKM {
        /// Offset (4 bits)
        pub const offset: u32 = 4;
        /// Mask (1 bit: 1 << 4)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// ACK response received/transmitted interrupt mask
    pub mod ACKM {
        /// Offset (5 bits)
        pub const offset: u32 = 5;
        /// Mask (1 bit: 1 << 5)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// response received interrupt mask
    pub mod NYET {
        /// Offset (6 bits)
        pub const offset: u32 = 6;
        /// Mask (1 bit: 1 << 6)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Transaction error mask
    pub mod TXERRM {
        /// Offset (7 bits)
        pub const offset: u32 = 7;
        /// Mask (1 bit: 1 << 7)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Babble error mask
    pub mod BBERRM {
        /// Offset (8 bits)
        pub const offset: u32 = 8;
        /// Mask (1 bit: 1 << 8)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Frame overrun mask
    pub mod FRMORM {
        /// Offset (9 bits)
        pub const offset: u32 = 9;
        /// Mask (1 bit: 1 << 9)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Data toggle error mask
    pub mod DTERRM {
        /// Offset (10 bits)
        pub const offset: u32 = 10;
        /// Mask (1 bit: 1 << 10)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS host channel-1 interrupt mask register
pub mod HCINTMSK1 {
    pub use super::HCINTMSK0::ACKM;
    pub use super::HCINTMSK0::AHBERR;
    pub use super::HCINTMSK0::BBERRM;
    pub use super::HCINTMSK0::CHHM;
    pub use super::HCINTMSK0::DTERRM;
    pub use super::HCINTMSK0::FRMORM;
    pub use super::HCINTMSK0::NAKM;
    pub use super::HCINTMSK0::NYET;
    pub use super::HCINTMSK0::STALLM;
    pub use super::HCINTMSK0::TXERRM;
    pub use super::HCINTMSK0::XFRCM;
}

/// OTG_HS host channel-2 interrupt mask register
pub mod HCINTMSK2 {
    pub use super::HCINTMSK0::ACKM;
    pub use super::HCINTMSK0::AHBERR;
    pub use super::HCINTMSK0::BBERRM;
    pub use super::HCINTMSK0::CHHM;
    pub use super::HCINTMSK0::DTERRM;
    pub use super::HCINTMSK0::FRMORM;
    pub use super::HCINTMSK0::NAKM;
    pub use super::HCINTMSK0::NYET;
    pub use super::HCINTMSK0::STALLM;
    pub use super::HCINTMSK0::TXERRM;
    pub use super::HCINTMSK0::XFRCM;
}

/// OTG_HS host channel-3 interrupt mask register
pub mod HCINTMSK3 {
    pub use super::HCINTMSK0::ACKM;
    pub use super::HCINTMSK0::AHBERR;
    pub use super::HCINTMSK0::BBERRM;
    pub use super::HCINTMSK0::CHHM;
    pub use super::HCINTMSK0::DTERRM;
    pub use super::HCINTMSK0::FRMORM;
    pub use super::HCINTMSK0::NAKM;
    pub use super::HCINTMSK0::NYET;
    pub use super::HCINTMSK0::STALLM;
    pub use super::HCINTMSK0::TXERRM;
    pub use super::HCINTMSK0::XFRCM;
}

/// OTG_HS host channel-4 interrupt mask register
pub mod HCINTMSK4 {
    pub use super::HCINTMSK0::ACKM;
    pub use super::HCINTMSK0::AHBERR;
    pub use super::HCINTMSK0::BBERRM;
    pub use super::HCINTMSK0::CHHM;
    pub use super::HCINTMSK0::DTERRM;
    pub use super::HCINTMSK0::FRMORM;
    pub use super::HCINTMSK0::NAKM;
    pub use super::HCINTMSK0::NYET;
    pub use super::HCINTMSK0::STALLM;
    pub use super::HCINTMSK0::TXERRM;
    pub use super::HCINTMSK0::XFRCM;
}

/// OTG_HS host channel-5 interrupt mask register
pub mod HCINTMSK5 {
    pub use super::HCINTMSK0::ACKM;
    pub use super::HCINTMSK0::AHBERR;
    pub use super::HCINTMSK0::BBERRM;
    pub use super::HCINTMSK0::CHHM;
    pub use super::HCINTMSK0::DTERRM;
    pub use super::HCINTMSK0::FRMORM;
    pub use super::HCINTMSK0::NAKM;
    pub use super::HCINTMSK0::NYET;
    pub use super::HCINTMSK0::STALLM;
    pub use super::HCINTMSK0::TXERRM;
    pub use super::HCINTMSK0::XFRCM;
}

/// OTG_HS host channel-6 interrupt mask register
pub mod HCINTMSK6 {
    pub use super::HCINTMSK0::ACKM;
    pub use super::HCINTMSK0::AHBERR;
    pub use super::HCINTMSK0::BBERRM;
    pub use super::HCINTMSK0::CHHM;
    pub use super::HCINTMSK0::DTERRM;
    pub use super::HCINTMSK0::FRMORM;
    pub use super::HCINTMSK0::NAKM;
    pub use super::HCINTMSK0::NYET;
    pub use super::HCINTMSK0::STALLM;
    pub use super::HCINTMSK0::TXERRM;
    pub use super::HCINTMSK0::XFRCM;
}

/// OTG_HS host channel-7 interrupt mask register
pub mod HCINTMSK7 {
    pub use super::HCINTMSK0::ACKM;
    pub use super::HCINTMSK0::AHBERR;
    pub use super::HCINTMSK0::BBERRM;
    pub use super::HCINTMSK0::CHHM;
    pub use super::HCINTMSK0::DTERRM;
    pub use super::HCINTMSK0::FRMORM;
    pub use super::HCINTMSK0::NAKM;
    pub use super::HCINTMSK0::NYET;
    pub use super::HCINTMSK0::STALLM;
    pub use super::HCINTMSK0::TXERRM;
    pub use super::HCINTMSK0::XFRCM;
}

/// OTG_HS host channel-8 interrupt mask register
pub mod HCINTMSK8 {
    pub use super::HCINTMSK0::ACKM;
    pub use super::HCINTMSK0::AHBERR;
    pub use super::HCINTMSK0::BBERRM;
    pub use super::HCINTMSK0::CHHM;
    pub use super::HCINTMSK0::DTERRM;
    pub use super::HCINTMSK0::FRMORM;
    pub use super::HCINTMSK0::NAKM;
    pub use super::HCINTMSK0::NYET;
    pub use super::HCINTMSK0::STALLM;
    pub use super::HCINTMSK0::TXERRM;
    pub use super::HCINTMSK0::XFRCM;
}

/// OTG_HS host channel-9 interrupt mask register
pub mod HCINTMSK9 {
    pub use super::HCINTMSK0::ACKM;
    pub use super::HCINTMSK0::AHBERR;
    pub use super::HCINTMSK0::BBERRM;
    pub use super::HCINTMSK0::CHHM;
    pub use super::HCINTMSK0::DTERRM;
    pub use super::HCINTMSK0::FRMORM;
    pub use super::HCINTMSK0::NAKM;
    pub use super::HCINTMSK0::NYET;
    pub use super::HCINTMSK0::STALLM;
    pub use super::HCINTMSK0::TXERRM;
    pub use super::HCINTMSK0::XFRCM;
}

/// OTG_HS host channel-10 interrupt mask register
pub mod HCINTMSK10 {
    pub use super::HCINTMSK0::ACKM;
    pub use super::HCINTMSK0::AHBERR;
    pub use super::HCINTMSK0::BBERRM;
    pub use super::HCINTMSK0::CHHM;
    pub use super::HCINTMSK0::DTERRM;
    pub use super::HCINTMSK0::FRMORM;
    pub use super::HCINTMSK0::NAKM;
    pub use super::HCINTMSK0::NYET;
    pub use super::HCINTMSK0::STALLM;
    pub use super::HCINTMSK0::TXERRM;
    pub use super::HCINTMSK0::XFRCM;
}

/// OTG_HS host channel-11 interrupt mask register
pub mod HCINTMSK11 {
    pub use super::HCINTMSK0::ACKM;
    pub use super::HCINTMSK0::AHBERR;
    pub use super::HCINTMSK0::BBERRM;
    pub use super::HCINTMSK0::CHHM;
    pub use super::HCINTMSK0::DTERRM;
    pub use super::HCINTMSK0::FRMORM;
    pub use super::HCINTMSK0::NAKM;
    pub use super::HCINTMSK0::NYET;
    pub use super::HCINTMSK0::STALLM;
    pub use super::HCINTMSK0::TXERRM;
    pub use super::HCINTMSK0::XFRCM;
}

/// OTG_HS host channel-11 transfer size register
pub mod HCTSIZ0 {

    /// Transfer size
    pub mod XFRSIZ {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (19 bits: 0x7ffff << 0)
        pub const mask: u32 = 0x7ffff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Packet count
    pub mod PKTCNT {
        /// Offset (19 bits)
        pub const offset: u32 = 19;
        /// Mask (10 bits: 0x3ff << 19)
        pub const mask: u32 = 0x3ff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Data PID
    pub mod DPID {
        /// Offset (29 bits)
        pub const offset: u32 = 29;
        /// Mask (2 bits: 0b11 << 29)
        pub const mask: u32 = 0b11 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS host channel-1 transfer size register
pub mod HCTSIZ1 {
    pub use super::HCTSIZ0::DPID;
    pub use super::HCTSIZ0::PKTCNT;
    pub use super::HCTSIZ0::XFRSIZ;
}

/// OTG_HS host channel-2 transfer size register
pub mod HCTSIZ2 {
    pub use super::HCTSIZ0::DPID;
    pub use super::HCTSIZ0::PKTCNT;
    pub use super::HCTSIZ0::XFRSIZ;
}

/// OTG_HS host channel-3 transfer size register
pub mod HCTSIZ3 {
    pub use super::HCTSIZ0::DPID;
    pub use super::HCTSIZ0::PKTCNT;
    pub use super::HCTSIZ0::XFRSIZ;
}

/// OTG_HS host channel-4 transfer size register
pub mod HCTSIZ4 {
    pub use super::HCTSIZ0::DPID;
    pub use super::HCTSIZ0::PKTCNT;
    pub use super::HCTSIZ0::XFRSIZ;
}

/// OTG_HS host channel-5 transfer size register
pub mod HCTSIZ5 {
    pub use super::HCTSIZ0::DPID;
    pub use super::HCTSIZ0::PKTCNT;
    pub use super::HCTSIZ0::XFRSIZ;
}

/// OTG_HS host channel-6 transfer size register
pub mod HCTSIZ6 {
    pub use super::HCTSIZ0::DPID;
    pub use super::HCTSIZ0::PKTCNT;
    pub use super::HCTSIZ0::XFRSIZ;
}

/// OTG_HS host channel-7 transfer size register
pub mod HCTSIZ7 {
    pub use super::HCTSIZ0::DPID;
    pub use super::HCTSIZ0::PKTCNT;
    pub use super::HCTSIZ0::XFRSIZ;
}

/// OTG_HS host channel-8 transfer size register
pub mod HCTSIZ8 {
    pub use super::HCTSIZ0::DPID;
    pub use super::HCTSIZ0::PKTCNT;
    pub use super::HCTSIZ0::XFRSIZ;
}

/// OTG_HS host channel-9 transfer size register
pub mod HCTSIZ9 {
    pub use super::HCTSIZ0::DPID;
    pub use super::HCTSIZ0::PKTCNT;
    pub use super::HCTSIZ0::XFRSIZ;
}

/// OTG_HS host channel-10 transfer size register
pub mod HCTSIZ10 {
    pub use super::HCTSIZ0::DPID;
    pub use super::HCTSIZ0::PKTCNT;
    pub use super::HCTSIZ0::XFRSIZ;
}

/// OTG_HS host channel-11 transfer size register
pub mod HCTSIZ11 {
    pub use super::HCTSIZ0::DPID;
    pub use super::HCTSIZ0::PKTCNT;
    pub use super::HCTSIZ0::XFRSIZ;
}

/// OTG_HS host channel-0 DMA address register
pub mod HCDMA0 {

    /// DMA address
    pub mod DMAADDR {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (32 bits: 0xffffffff << 0)
        pub const mask: u32 = 0xffffffff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS host channel-1 DMA address register
pub mod HCDMA1 {
    pub use super::HCDMA0::DMAADDR;
}

/// OTG_HS host channel-2 DMA address register
pub mod HCDMA2 {
    pub use super::HCDMA0::DMAADDR;
}

/// OTG_HS host channel-3 DMA address register
pub mod HCDMA3 {
    pub use super::HCDMA0::DMAADDR;
}

/// OTG_HS host channel-4 DMA address register
pub mod HCDMA4 {
    pub use super::HCDMA0::DMAADDR;
}

/// OTG_HS host channel-5 DMA address register
pub mod HCDMA5 {
    pub use super::HCDMA0::DMAADDR;
}

/// OTG_HS host channel-6 DMA address register
pub mod HCDMA6 {
    pub use super::HCDMA0::DMAADDR;
}

/// OTG_HS host channel-7 DMA address register
pub mod HCDMA7 {
    pub use super::HCDMA0::DMAADDR;
}

/// OTG_HS host channel-8 DMA address register
pub mod HCDMA8 {
    pub use super::HCDMA0::DMAADDR;
}

/// OTG_HS host channel-9 DMA address register
pub mod HCDMA9 {
    pub use super::HCDMA0::DMAADDR;
}

/// OTG_HS host channel-10 DMA address register
pub mod HCDMA10 {
    pub use super::HCDMA0::DMAADDR;
}

/// OTG_HS host channel-11 DMA address register
pub mod HCDMA11 {
    pub use super::HCDMA0::DMAADDR;
}
#[repr(C)]
pub struct RegisterBlock {
    /// OTG_HS host configuration register
    pub HCFG: RWRegister<u32>,

    /// OTG_HS Host frame interval register
    pub HFIR: RWRegister<u32>,

    /// OTG_HS host frame number/frame time remaining register
    pub HFNUM: RORegister<u32>,

    _reserved1: [u32; 1],

    /// OTG_HS_Host periodic transmit FIFO/queue status register
    pub HPTXSTS: RWRegister<u32>,

    /// OTG_HS Host all channels interrupt register
    pub HAINT: RORegister<u32>,

    /// OTG_HS host all channels interrupt mask register
    pub HAINTMSK: RWRegister<u32>,

    _reserved2: [u32; 9],

    /// OTG_HS host port control and status register
    pub HPRT: RWRegister<u32>,

    _reserved3: [u32; 47],

    /// OTG_HS host channel-0 characteristics register
    pub HCCHAR0: RWRegister<u32>,

    /// OTG_HS host channel-0 split control register
    pub HCSPLT0: RWRegister<u32>,

    /// OTG_HS host channel-11 interrupt register
    pub HCINT0: RWRegister<u32>,

    /// OTG_HS host channel-11 interrupt mask register
    pub HCINTMSK0: RWRegister<u32>,

    /// OTG_HS host channel-11 transfer size register
    pub HCTSIZ0: RWRegister<u32>,

    /// OTG_HS host channel-0 DMA address register
    pub HCDMA0: UnsafeRWRegister<u32>,

    _reserved4: [u32; 2],

    /// OTG_HS host channel-1 characteristics register
    pub HCCHAR1: RWRegister<u32>,

    /// OTG_HS host channel-1 split control register
    pub HCSPLT1: RWRegister<u32>,

    /// OTG_HS host channel-1 interrupt register
    pub HCINT1: RWRegister<u32>,

    /// OTG_HS host channel-1 interrupt mask register
    pub HCINTMSK1: RWRegister<u32>,

    /// OTG_HS host channel-1 transfer size register
    pub HCTSIZ1: RWRegister<u32>,

    /// OTG_HS host channel-1 DMA address register
    pub HCDMA1: UnsafeRWRegister<u32>,

    _reserved5: [u32; 2],

    /// OTG_HS host channel-2 characteristics register
    pub HCCHAR2: RWRegister<u32>,

    /// OTG_HS host channel-2 split control register
    pub HCSPLT2: RWRegister<u32>,

    /// OTG_HS host channel-2 interrupt register
    pub HCINT2: RWRegister<u32>,

    /// OTG_HS host channel-2 interrupt mask register
    pub HCINTMSK2: RWRegister<u32>,

    /// OTG_HS host channel-2 transfer size register
    pub HCTSIZ2: RWRegister<u32>,

    /// OTG_HS host channel-2 DMA address register
    pub HCDMA2: UnsafeRWRegister<u32>,

    _reserved6: [u32; 2],

    /// OTG_HS host channel-3 characteristics register
    pub HCCHAR3: RWRegister<u32>,

    /// OTG_HS host channel-3 split control register
    pub HCSPLT3: RWRegister<u32>,

    /// OTG_HS host channel-3 interrupt register
    pub HCINT3: RWRegister<u32>,

    /// OTG_HS host channel-3 interrupt mask register
    pub HCINTMSK3: RWRegister<u32>,

    /// OTG_HS host channel-3 transfer size register
    pub HCTSIZ3: RWRegister<u32>,

    /// OTG_HS host channel-3 DMA address register
    pub HCDMA3: UnsafeRWRegister<u32>,

    _reserved7: [u32; 2],

    /// OTG_HS host channel-4 characteristics register
    pub HCCHAR4: RWRegister<u32>,

    /// OTG_HS host channel-4 split control register
    pub HCSPLT4: RWRegister<u32>,

    /// OTG_HS host channel-4 interrupt register
    pub HCINT4: RWRegister<u32>,

    /// OTG_HS host channel-4 interrupt mask register
    pub HCINTMSK4: RWRegister<u32>,

    /// OTG_HS host channel-4 transfer size register
    pub HCTSIZ4: RWRegister<u32>,

    /// OTG_HS host channel-4 DMA address register
    pub HCDMA4: UnsafeRWRegister<u32>,

    _reserved8: [u32; 2],

    /// OTG_HS host channel-5 characteristics register
    pub HCCHAR5: RWRegister<u32>,

    /// OTG_HS host channel-5 split control register
    pub HCSPLT5: RWRegister<u32>,

    /// OTG_HS host channel-5 interrupt register
    pub HCINT5: RWRegister<u32>,

    /// OTG_HS host channel-5 interrupt mask register
    pub HCINTMSK5: RWRegister<u32>,

    /// OTG_HS host channel-5 transfer size register
    pub HCTSIZ5: RWRegister<u32>,

    /// OTG_HS host channel-5 DMA address register
    pub HCDMA5: UnsafeRWRegister<u32>,

    _reserved9: [u32; 2],

    /// OTG_HS host channel-6 characteristics register
    pub HCCHAR6: RWRegister<u32>,

    /// OTG_HS host channel-6 split control register
    pub HCSPLT6: RWRegister<u32>,

    /// OTG_HS host channel-6 interrupt register
    pub HCINT6: RWRegister<u32>,

    /// OTG_HS host channel-6 interrupt mask register
    pub HCINTMSK6: RWRegister<u32>,

    /// OTG_HS host channel-6 transfer size register
    pub HCTSIZ6: RWRegister<u32>,

    /// OTG_HS host channel-6 DMA address register
    pub HCDMA6: UnsafeRWRegister<u32>,

    _reserved10: [u32; 2],

    /// OTG_HS host channel-7 characteristics register
    pub HCCHAR7: RWRegister<u32>,

    /// OTG_HS host channel-7 split control register
    pub HCSPLT7: RWRegister<u32>,

    /// OTG_HS host channel-7 interrupt register
    pub HCINT7: RWRegister<u32>,

    /// OTG_HS host channel-7 interrupt mask register
    pub HCINTMSK7: RWRegister<u32>,

    /// OTG_HS host channel-7 transfer size register
    pub HCTSIZ7: RWRegister<u32>,

    /// OTG_HS host channel-7 DMA address register
    pub HCDMA7: UnsafeRWRegister<u32>,

    _reserved11: [u32; 2],

    /// OTG_HS host channel-8 characteristics register
    pub HCCHAR8: RWRegister<u32>,

    /// OTG_HS host channel-8 split control register
    pub HCSPLT8: RWRegister<u32>,

    /// OTG_HS host channel-8 interrupt register
    pub HCINT8: RWRegister<u32>,

    /// OTG_HS host channel-8 interrupt mask register
    pub HCINTMSK8: RWRegister<u32>,

    /// OTG_HS host channel-8 transfer size register
    pub HCTSIZ8: RWRegister<u32>,

    /// OTG_HS host channel-8 DMA address register
    pub HCDMA8: UnsafeRWRegister<u32>,

    _reserved12: [u32; 2],

    /// OTG_HS host channel-9 characteristics register
    pub HCCHAR9: RWRegister<u32>,

    /// OTG_HS host channel-9 split control register
    pub HCSPLT9: RWRegister<u32>,

    /// OTG_HS host channel-9 interrupt register
    pub HCINT9: RWRegister<u32>,

    /// OTG_HS host channel-9 interrupt mask register
    pub HCINTMSK9: RWRegister<u32>,

    /// OTG_HS host channel-9 transfer size register
    pub HCTSIZ9: RWRegister<u32>,

    /// OTG_HS host channel-9 DMA address register
    pub HCDMA9: UnsafeRWRegister<u32>,

    _reserved13: [u32; 2],

    /// OTG_HS host channel-10 characteristics register
    pub HCCHAR10: RWRegister<u32>,

    /// OTG_HS host channel-10 split control register
    pub HCSPLT10: RWRegister<u32>,

    /// OTG_HS host channel-10 interrupt register
    pub HCINT10: RWRegister<u32>,

    /// OTG_HS host channel-10 interrupt mask register
    pub HCINTMSK10: RWRegister<u32>,

    /// OTG_HS host channel-10 transfer size register
    pub HCTSIZ10: RWRegister<u32>,

    /// OTG_HS host channel-10 DMA address register
    pub HCDMA10: UnsafeRWRegister<u32>,

    _reserved14: [u32; 2],

    /// OTG_HS host channel-11 characteristics register
    pub HCCHAR11: RWRegister<u32>,

    /// OTG_HS host channel-11 split control register
    pub HCSPLT11: RWRegister<u32>,

    /// OTG_HS host channel-11 interrupt register
    pub HCINT11: RWRegister<u32>,

    /// OTG_HS host channel-11 interrupt mask register
    pub HCINTMSK11: RWRegister<u32>,

    /// OTG_HS host channel-11 transfer size register
    pub HCTSIZ11: RWRegister<u32>,

    /// OTG_HS host channel-11 DMA address register
    pub HCDMA11: UnsafeRWRegister<u32>,
}
pub struct ResetValues {
    pub HCFG: u32,
    pub HFIR: u32,
    pub HFNUM: u32,
    pub HPTXSTS: u32,
    pub HAINT: u32,
    pub HAINTMSK: u32,
    pub HPRT: u32,
    pub HCCHAR0: u32,
    pub HCSPLT0: u32,
    pub HCINT0: u32,
    pub HCINTMSK0: u32,
    pub HCTSIZ0: u32,
    pub HCDMA0: u32,
    pub HCCHAR1: u32,
    pub HCSPLT1: u32,
    pub HCINT1: u32,
    pub HCINTMSK1: u32,
    pub HCTSIZ1: u32,
    pub HCDMA1: u32,
    pub HCCHAR2: u32,
    pub HCSPLT2: u32,
    pub HCINT2: u32,
    pub HCINTMSK2: u32,
    pub HCTSIZ2: u32,
    pub HCDMA2: u32,
    pub HCCHAR3: u32,
    pub HCSPLT3: u32,
    pub HCINT3: u32,
    pub HCINTMSK3: u32,
    pub HCTSIZ3: u32,
    pub HCDMA3: u32,
    pub HCCHAR4: u32,
    pub HCSPLT4: u32,
    pub HCINT4: u32,
    pub HCINTMSK4: u32,
    pub HCTSIZ4: u32,
    pub HCDMA4: u32,
    pub HCCHAR5: u32,
    pub HCSPLT5: u32,
    pub HCINT5: u32,
    pub HCINTMSK5: u32,
    pub HCTSIZ5: u32,
    pub HCDMA5: u32,
    pub HCCHAR6: u32,
    pub HCSPLT6: u32,
    pub HCINT6: u32,
    pub HCINTMSK6: u32,
    pub HCTSIZ6: u32,
    pub HCDMA6: u32,
    pub HCCHAR7: u32,
    pub HCSPLT7: u32,
    pub HCINT7: u32,
    pub HCINTMSK7: u32,
    pub HCTSIZ7: u32,
    pub HCDMA7: u32,
    pub HCCHAR8: u32,
    pub HCSPLT8: u32,
    pub HCINT8: u32,
    pub HCINTMSK8: u32,
    pub HCTSIZ8: u32,
    pub HCDMA8: u32,
    pub HCCHAR9: u32,
    pub HCSPLT9: u32,
    pub HCINT9: u32,
    pub HCINTMSK9: u32,
    pub HCTSIZ9: u32,
    pub HCDMA9: u32,
    pub HCCHAR10: u32,
    pub HCSPLT10: u32,
    pub HCINT10: u32,
    pub HCINTMSK10: u32,
    pub HCTSIZ10: u32,
    pub HCDMA10: u32,
    pub HCCHAR11: u32,
    pub HCSPLT11: u32,
    pub HCINT11: u32,
    pub HCINTMSK11: u32,
    pub HCTSIZ11: u32,
    pub HCDMA11: u32,
}
#[cfg(not(feature = "nosync"))]
pub struct Instance {
    pub(crate) addr: u32,
    pub(crate) _marker: PhantomData<*const RegisterBlock>,
}
#[cfg(not(feature = "nosync"))]
impl ::core::ops::Deref for Instance {
    type Target = RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*(self.addr as *const _) }
    }
}
#[cfg(feature = "rtfm")]
unsafe impl Send for Instance {}
