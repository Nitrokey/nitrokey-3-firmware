#![allow(non_snake_case, non_upper_case_globals)]
#![allow(non_camel_case_types)]
//! USB on the go full speed
//!
//! Used by: stm32f401, stm32f405, stm32f407, stm32f411, stm32f412, stm32f413, stm32f427, stm32f429, stm32f446, stm32f469

use super::super::register::{RORegister, RWRegister};
#[cfg(not(feature = "nosync"))]
use core::marker::PhantomData;

/// OTG_FS host configuration register (OTG_FS_HCFG)
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

/// OTG_FS Host frame interval register
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

/// OTG_FS host frame number/frame time remaining register (OTG_FS_HFNUM)
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

/// OTG_FS_Host periodic transmit FIFO/queue status register (OTG_FS_HPTXSTS)
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

/// OTG_FS Host all channels interrupt register
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

/// OTG_FS host all channels interrupt mask register
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

/// OTG_FS host port control and status register (OTG_FS_HPRT)
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

/// OTG_FS host channel-0 characteristics register (OTG_FS_HCCHAR0)
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

    /// Multicount
    pub mod MCNT {
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

/// OTG_FS host channel-1 characteristics register (OTG_FS_HCCHAR1)
pub mod HCCHAR1 {
    pub use super::HCCHAR0::CHDIS;
    pub use super::HCCHAR0::CHENA;
    pub use super::HCCHAR0::DAD;
    pub use super::HCCHAR0::EPDIR;
    pub use super::HCCHAR0::EPNUM;
    pub use super::HCCHAR0::EPTYP;
    pub use super::HCCHAR0::LSDEV;
    pub use super::HCCHAR0::MCNT;
    pub use super::HCCHAR0::MPSIZ;
    pub use super::HCCHAR0::ODDFRM;
}

/// OTG_FS host channel-2 characteristics register (OTG_FS_HCCHAR2)
pub mod HCCHAR2 {
    pub use super::HCCHAR0::CHDIS;
    pub use super::HCCHAR0::CHENA;
    pub use super::HCCHAR0::DAD;
    pub use super::HCCHAR0::EPDIR;
    pub use super::HCCHAR0::EPNUM;
    pub use super::HCCHAR0::EPTYP;
    pub use super::HCCHAR0::LSDEV;
    pub use super::HCCHAR0::MCNT;
    pub use super::HCCHAR0::MPSIZ;
    pub use super::HCCHAR0::ODDFRM;
}

/// OTG_FS host channel-3 characteristics register (OTG_FS_HCCHAR3)
pub mod HCCHAR3 {
    pub use super::HCCHAR0::CHDIS;
    pub use super::HCCHAR0::CHENA;
    pub use super::HCCHAR0::DAD;
    pub use super::HCCHAR0::EPDIR;
    pub use super::HCCHAR0::EPNUM;
    pub use super::HCCHAR0::EPTYP;
    pub use super::HCCHAR0::LSDEV;
    pub use super::HCCHAR0::MCNT;
    pub use super::HCCHAR0::MPSIZ;
    pub use super::HCCHAR0::ODDFRM;
}

/// OTG_FS host channel-4 characteristics register (OTG_FS_HCCHAR4)
pub mod HCCHAR4 {
    pub use super::HCCHAR0::CHDIS;
    pub use super::HCCHAR0::CHENA;
    pub use super::HCCHAR0::DAD;
    pub use super::HCCHAR0::EPDIR;
    pub use super::HCCHAR0::EPNUM;
    pub use super::HCCHAR0::EPTYP;
    pub use super::HCCHAR0::LSDEV;
    pub use super::HCCHAR0::MCNT;
    pub use super::HCCHAR0::MPSIZ;
    pub use super::HCCHAR0::ODDFRM;
}

/// OTG_FS host channel-5 characteristics register (OTG_FS_HCCHAR5)
pub mod HCCHAR5 {
    pub use super::HCCHAR0::CHDIS;
    pub use super::HCCHAR0::CHENA;
    pub use super::HCCHAR0::DAD;
    pub use super::HCCHAR0::EPDIR;
    pub use super::HCCHAR0::EPNUM;
    pub use super::HCCHAR0::EPTYP;
    pub use super::HCCHAR0::LSDEV;
    pub use super::HCCHAR0::MCNT;
    pub use super::HCCHAR0::MPSIZ;
    pub use super::HCCHAR0::ODDFRM;
}

/// OTG_FS host channel-6 characteristics register (OTG_FS_HCCHAR6)
pub mod HCCHAR6 {
    pub use super::HCCHAR0::CHDIS;
    pub use super::HCCHAR0::CHENA;
    pub use super::HCCHAR0::DAD;
    pub use super::HCCHAR0::EPDIR;
    pub use super::HCCHAR0::EPNUM;
    pub use super::HCCHAR0::EPTYP;
    pub use super::HCCHAR0::LSDEV;
    pub use super::HCCHAR0::MCNT;
    pub use super::HCCHAR0::MPSIZ;
    pub use super::HCCHAR0::ODDFRM;
}

/// OTG_FS host channel-7 characteristics register (OTG_FS_HCCHAR7)
pub mod HCCHAR7 {
    pub use super::HCCHAR0::CHDIS;
    pub use super::HCCHAR0::CHENA;
    pub use super::HCCHAR0::DAD;
    pub use super::HCCHAR0::EPDIR;
    pub use super::HCCHAR0::EPNUM;
    pub use super::HCCHAR0::EPTYP;
    pub use super::HCCHAR0::LSDEV;
    pub use super::HCCHAR0::MCNT;
    pub use super::HCCHAR0::MPSIZ;
    pub use super::HCCHAR0::ODDFRM;
}

/// OTG_FS host channel-0 interrupt register (OTG_FS_HCINT0)
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

/// OTG_FS host channel-1 interrupt register (OTG_FS_HCINT1)
pub mod HCINT1 {
    pub use super::HCINT0::ACK;
    pub use super::HCINT0::BBERR;
    pub use super::HCINT0::CHH;
    pub use super::HCINT0::DTERR;
    pub use super::HCINT0::FRMOR;
    pub use super::HCINT0::NAK;
    pub use super::HCINT0::STALL;
    pub use super::HCINT0::TXERR;
    pub use super::HCINT0::XFRC;
}

/// OTG_FS host channel-2 interrupt register (OTG_FS_HCINT2)
pub mod HCINT2 {
    pub use super::HCINT0::ACK;
    pub use super::HCINT0::BBERR;
    pub use super::HCINT0::CHH;
    pub use super::HCINT0::DTERR;
    pub use super::HCINT0::FRMOR;
    pub use super::HCINT0::NAK;
    pub use super::HCINT0::STALL;
    pub use super::HCINT0::TXERR;
    pub use super::HCINT0::XFRC;
}

/// OTG_FS host channel-3 interrupt register (OTG_FS_HCINT3)
pub mod HCINT3 {
    pub use super::HCINT0::ACK;
    pub use super::HCINT0::BBERR;
    pub use super::HCINT0::CHH;
    pub use super::HCINT0::DTERR;
    pub use super::HCINT0::FRMOR;
    pub use super::HCINT0::NAK;
    pub use super::HCINT0::STALL;
    pub use super::HCINT0::TXERR;
    pub use super::HCINT0::XFRC;
}

/// OTG_FS host channel-4 interrupt register (OTG_FS_HCINT4)
pub mod HCINT4 {
    pub use super::HCINT0::ACK;
    pub use super::HCINT0::BBERR;
    pub use super::HCINT0::CHH;
    pub use super::HCINT0::DTERR;
    pub use super::HCINT0::FRMOR;
    pub use super::HCINT0::NAK;
    pub use super::HCINT0::STALL;
    pub use super::HCINT0::TXERR;
    pub use super::HCINT0::XFRC;
}

/// OTG_FS host channel-5 interrupt register (OTG_FS_HCINT5)
pub mod HCINT5 {
    pub use super::HCINT0::ACK;
    pub use super::HCINT0::BBERR;
    pub use super::HCINT0::CHH;
    pub use super::HCINT0::DTERR;
    pub use super::HCINT0::FRMOR;
    pub use super::HCINT0::NAK;
    pub use super::HCINT0::STALL;
    pub use super::HCINT0::TXERR;
    pub use super::HCINT0::XFRC;
}

/// OTG_FS host channel-6 interrupt register (OTG_FS_HCINT6)
pub mod HCINT6 {
    pub use super::HCINT0::ACK;
    pub use super::HCINT0::BBERR;
    pub use super::HCINT0::CHH;
    pub use super::HCINT0::DTERR;
    pub use super::HCINT0::FRMOR;
    pub use super::HCINT0::NAK;
    pub use super::HCINT0::STALL;
    pub use super::HCINT0::TXERR;
    pub use super::HCINT0::XFRC;
}

/// OTG_FS host channel-7 interrupt register (OTG_FS_HCINT7)
pub mod HCINT7 {
    pub use super::HCINT0::ACK;
    pub use super::HCINT0::BBERR;
    pub use super::HCINT0::CHH;
    pub use super::HCINT0::DTERR;
    pub use super::HCINT0::FRMOR;
    pub use super::HCINT0::NAK;
    pub use super::HCINT0::STALL;
    pub use super::HCINT0::TXERR;
    pub use super::HCINT0::XFRC;
}

/// OTG_FS host channel-0 mask register (OTG_FS_HCINTMSK0)
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

/// OTG_FS host channel-1 mask register (OTG_FS_HCINTMSK1)
pub mod HCINTMSK1 {
    pub use super::HCINTMSK0::ACKM;
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

/// OTG_FS host channel-2 mask register (OTG_FS_HCINTMSK2)
pub mod HCINTMSK2 {
    pub use super::HCINTMSK0::ACKM;
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

/// OTG_FS host channel-3 mask register (OTG_FS_HCINTMSK3)
pub mod HCINTMSK3 {
    pub use super::HCINTMSK0::ACKM;
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

/// OTG_FS host channel-4 mask register (OTG_FS_HCINTMSK4)
pub mod HCINTMSK4 {
    pub use super::HCINTMSK0::ACKM;
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

/// OTG_FS host channel-5 mask register (OTG_FS_HCINTMSK5)
pub mod HCINTMSK5 {
    pub use super::HCINTMSK0::ACKM;
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

/// OTG_FS host channel-6 mask register (OTG_FS_HCINTMSK6)
pub mod HCINTMSK6 {
    pub use super::HCINTMSK0::ACKM;
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

/// OTG_FS host channel-7 mask register (OTG_FS_HCINTMSK7)
pub mod HCINTMSK7 {
    pub use super::HCINTMSK0::ACKM;
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

/// OTG_FS host channel-0 transfer size register
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

/// OTG_FS host channel-1 transfer size register
pub mod HCTSIZ1 {
    pub use super::HCTSIZ0::DPID;
    pub use super::HCTSIZ0::PKTCNT;
    pub use super::HCTSIZ0::XFRSIZ;
}

/// OTG_FS host channel-2 transfer size register
pub mod HCTSIZ2 {
    pub use super::HCTSIZ0::DPID;
    pub use super::HCTSIZ0::PKTCNT;
    pub use super::HCTSIZ0::XFRSIZ;
}

/// OTG_FS host channel-3 transfer size register
pub mod HCTSIZ3 {
    pub use super::HCTSIZ0::DPID;
    pub use super::HCTSIZ0::PKTCNT;
    pub use super::HCTSIZ0::XFRSIZ;
}

/// OTG_FS host channel-x transfer size register
pub mod HCTSIZ4 {
    pub use super::HCTSIZ0::DPID;
    pub use super::HCTSIZ0::PKTCNT;
    pub use super::HCTSIZ0::XFRSIZ;
}

/// OTG_FS host channel-5 transfer size register
pub mod HCTSIZ5 {
    pub use super::HCTSIZ0::DPID;
    pub use super::HCTSIZ0::PKTCNT;
    pub use super::HCTSIZ0::XFRSIZ;
}

/// OTG_FS host channel-6 transfer size register
pub mod HCTSIZ6 {
    pub use super::HCTSIZ0::DPID;
    pub use super::HCTSIZ0::PKTCNT;
    pub use super::HCTSIZ0::XFRSIZ;
}

/// OTG_FS host channel-7 transfer size register
pub mod HCTSIZ7 {
    pub use super::HCTSIZ0::DPID;
    pub use super::HCTSIZ0::PKTCNT;
    pub use super::HCTSIZ0::XFRSIZ;
}
#[repr(C)]
pub struct RegisterBlock {
    /// OTG_FS host configuration register (OTG_FS_HCFG)
    pub HCFG: RWRegister<u32>,

    /// OTG_FS Host frame interval register
    pub HFIR: RWRegister<u32>,

    /// OTG_FS host frame number/frame time remaining register (OTG_FS_HFNUM)
    pub HFNUM: RORegister<u32>,

    _reserved1: [u32; 1],

    /// OTG_FS_Host periodic transmit FIFO/queue status register (OTG_FS_HPTXSTS)
    pub HPTXSTS: RWRegister<u32>,

    /// OTG_FS Host all channels interrupt register
    pub HAINT: RORegister<u32>,

    /// OTG_FS host all channels interrupt mask register
    pub HAINTMSK: RWRegister<u32>,

    _reserved2: [u32; 9],

    /// OTG_FS host port control and status register (OTG_FS_HPRT)
    pub HPRT: RWRegister<u32>,

    _reserved3: [u32; 47],

    /// OTG_FS host channel-0 characteristics register (OTG_FS_HCCHAR0)
    pub HCCHAR0: RWRegister<u32>,

    _reserved4: [u32; 1],

    /// OTG_FS host channel-0 interrupt register (OTG_FS_HCINT0)
    pub HCINT0: RWRegister<u32>,

    /// OTG_FS host channel-0 mask register (OTG_FS_HCINTMSK0)
    pub HCINTMSK0: RWRegister<u32>,

    /// OTG_FS host channel-0 transfer size register
    pub HCTSIZ0: RWRegister<u32>,

    _reserved5: [u32; 3],

    /// OTG_FS host channel-1 characteristics register (OTG_FS_HCCHAR1)
    pub HCCHAR1: RWRegister<u32>,

    _reserved6: [u32; 1],

    /// OTG_FS host channel-1 interrupt register (OTG_FS_HCINT1)
    pub HCINT1: RWRegister<u32>,

    /// OTG_FS host channel-1 mask register (OTG_FS_HCINTMSK1)
    pub HCINTMSK1: RWRegister<u32>,

    /// OTG_FS host channel-1 transfer size register
    pub HCTSIZ1: RWRegister<u32>,

    _reserved7: [u32; 3],

    /// OTG_FS host channel-2 characteristics register (OTG_FS_HCCHAR2)
    pub HCCHAR2: RWRegister<u32>,

    _reserved8: [u32; 1],

    /// OTG_FS host channel-2 interrupt register (OTG_FS_HCINT2)
    pub HCINT2: RWRegister<u32>,

    /// OTG_FS host channel-2 mask register (OTG_FS_HCINTMSK2)
    pub HCINTMSK2: RWRegister<u32>,

    /// OTG_FS host channel-2 transfer size register
    pub HCTSIZ2: RWRegister<u32>,

    _reserved9: [u32; 3],

    /// OTG_FS host channel-3 characteristics register (OTG_FS_HCCHAR3)
    pub HCCHAR3: RWRegister<u32>,

    _reserved10: [u32; 1],

    /// OTG_FS host channel-3 interrupt register (OTG_FS_HCINT3)
    pub HCINT3: RWRegister<u32>,

    /// OTG_FS host channel-3 mask register (OTG_FS_HCINTMSK3)
    pub HCINTMSK3: RWRegister<u32>,

    /// OTG_FS host channel-3 transfer size register
    pub HCTSIZ3: RWRegister<u32>,

    _reserved11: [u32; 3],

    /// OTG_FS host channel-4 characteristics register (OTG_FS_HCCHAR4)
    pub HCCHAR4: RWRegister<u32>,

    _reserved12: [u32; 1],

    /// OTG_FS host channel-4 interrupt register (OTG_FS_HCINT4)
    pub HCINT4: RWRegister<u32>,

    /// OTG_FS host channel-4 mask register (OTG_FS_HCINTMSK4)
    pub HCINTMSK4: RWRegister<u32>,

    /// OTG_FS host channel-x transfer size register
    pub HCTSIZ4: RWRegister<u32>,

    _reserved13: [u32; 3],

    /// OTG_FS host channel-5 characteristics register (OTG_FS_HCCHAR5)
    pub HCCHAR5: RWRegister<u32>,

    _reserved14: [u32; 1],

    /// OTG_FS host channel-5 interrupt register (OTG_FS_HCINT5)
    pub HCINT5: RWRegister<u32>,

    /// OTG_FS host channel-5 mask register (OTG_FS_HCINTMSK5)
    pub HCINTMSK5: RWRegister<u32>,

    /// OTG_FS host channel-5 transfer size register
    pub HCTSIZ5: RWRegister<u32>,

    _reserved15: [u32; 3],

    /// OTG_FS host channel-6 characteristics register (OTG_FS_HCCHAR6)
    pub HCCHAR6: RWRegister<u32>,

    _reserved16: [u32; 1],

    /// OTG_FS host channel-6 interrupt register (OTG_FS_HCINT6)
    pub HCINT6: RWRegister<u32>,

    /// OTG_FS host channel-6 mask register (OTG_FS_HCINTMSK6)
    pub HCINTMSK6: RWRegister<u32>,

    /// OTG_FS host channel-6 transfer size register
    pub HCTSIZ6: RWRegister<u32>,

    _reserved17: [u32; 3],

    /// OTG_FS host channel-7 characteristics register (OTG_FS_HCCHAR7)
    pub HCCHAR7: RWRegister<u32>,

    _reserved18: [u32; 1],

    /// OTG_FS host channel-7 interrupt register (OTG_FS_HCINT7)
    pub HCINT7: RWRegister<u32>,

    /// OTG_FS host channel-7 mask register (OTG_FS_HCINTMSK7)
    pub HCINTMSK7: RWRegister<u32>,

    /// OTG_FS host channel-7 transfer size register
    pub HCTSIZ7: RWRegister<u32>,
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
    pub HCINT0: u32,
    pub HCINTMSK0: u32,
    pub HCTSIZ0: u32,
    pub HCCHAR1: u32,
    pub HCINT1: u32,
    pub HCINTMSK1: u32,
    pub HCTSIZ1: u32,
    pub HCCHAR2: u32,
    pub HCINT2: u32,
    pub HCINTMSK2: u32,
    pub HCTSIZ2: u32,
    pub HCCHAR3: u32,
    pub HCINT3: u32,
    pub HCINTMSK3: u32,
    pub HCTSIZ3: u32,
    pub HCCHAR4: u32,
    pub HCINT4: u32,
    pub HCINTMSK4: u32,
    pub HCTSIZ4: u32,
    pub HCCHAR5: u32,
    pub HCINT5: u32,
    pub HCINTMSK5: u32,
    pub HCTSIZ5: u32,
    pub HCCHAR6: u32,
    pub HCINT6: u32,
    pub HCINTMSK6: u32,
    pub HCTSIZ6: u32,
    pub HCCHAR7: u32,
    pub HCINT7: u32,
    pub HCINTMSK7: u32,
    pub HCTSIZ7: u32,
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
