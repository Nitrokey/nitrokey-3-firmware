use bitfield::bitfield;

#[derive(Debug)]
pub struct TryFromU8Error;

macro_rules! enum_u8 {
    (
        $(#[$outer:meta])*
        $vis:vis enum $name:ident {

            $($(#[doc = $doc:literal])* $var:ident = $num:expr $(; $more_num:expr)*),+
            $(,)*
        }
    ) => {
        $(#[$outer])*
        #[repr(u8)]
        $vis enum $name {
            $(
                $(#[doc = $doc])*
                $var = $num,
            )*
        }

        impl TryFrom<u8> for $name {
            type Error = TryFromU8Error;
            fn try_from(val: u8) -> ::core::result::Result<Self, TryFromU8Error> {
                match val {
                    $(
                        $num $(| $more_num)* => Ok($name::$var),
                    )*
                    _ => Err(TryFromU8Error)
                }
            }
        }

        impl From<$name> for u8 {
            fn from(val: $name) -> u8 {
                match val {
                    $(
                        $name::$var => $num,
                    )*
                }
            }
        }

        impl From<$name> for U8Wrapper {
            fn from(val: $name) -> U8Wrapper {
                match val {
                    $(
                        $name::$var => U8Wrapper($num),
                    )*
                }
            }
        }

        impl From<U8Wrapper> for $name {
            fn from(val: U8Wrapper) -> $name {
                // We know this can't fail because the from is only used with the proper bitmasking in the bitfield macro
                val.0.try_into().unwrap()
            }
        }

    }
}

pub trait Register: From<u8> + Into<u8> {
    const ADDRESS: u16;
}

macro_rules! register {
    ($reg:path, $addr:expr) => {
        impl From<u8> for $reg {
            fn from(value: u8) -> $reg {
                $reg(value)
            }
        }
        impl From<$reg> for u8 {
            fn from($reg(value): $reg) -> u8 {
                value
            }
        }
        impl Register for $reg {
            const ADDRESS: u16 = $addr;
        }
    };
}

macro_rules! register_bitfield {
    (
        $(#[$attribute:meta])*
        struct $name:ident(u8): $addr:expr;
        $($rest:tt)*
    ) => {
        register!($name, $addr);

        bitfield! {
            $(#[$attribute])*
            pub struct $name(u8);
            $($rest)*
        }
    };
}

// For bitfield compatibility
// Some struct implement From<U8Wrapper> even though they
// should implement TryFrom, but since we know that the bytes obtained from the register
// are only ever obtained from the correct bits, we know the conversion will never fail,
// but we still don't want to implement a panicking From<u8>
//
// U8Wrapper should not be used outside of the bitfield implementations for registers
struct U8Wrapper(u8);

impl bitfield::BitRange<U8Wrapper> for u8 {
    fn bit_range(&self, msb: usize, lsb: usize) -> U8Wrapper {
        U8Wrapper(self.bit_range(msb, lsb))
    }
}

impl bitfield::BitRangeMut<U8Wrapper> for u8 {
    fn set_bit_range(&mut self, msb: usize, lsb: usize, value: U8Wrapper) {
        self.set_bit_range(msb, lsb, value.0);
    }
}

enum_u8! {
    #[derive(Debug, Clone, Copy)]
    pub enum VoutMode {
        PinDisable = 0b00 ; 0b01,
        EnabledAfterPowerOn = 0b10,
        ControledByVoutEnCfg = 0b11,
    }
}

enum_u8! {
    #[derive(Debug, Clone, Copy)]
    pub enum Fdel {
        /// 2/fc
        Fc2 = 0b00,
        /// 4/fc
        Fc4 = 0b01,
        /// 6/fc
        Fc6 = 0b10,
        /// 8/fc
        Fc8 = 0b11,
    }
}

enum_u8! {
    #[derive(Debug, Clone, Copy)]
    pub enum NfcMode {
        Iso14443_4 = 0b00,
        Iso14443_3 = 0b01,
        RFU1 = 0b10,
        RFU2 = 0b11,
    }
}

register_bitfield! {
    #[derive(Clone)]
    struct UserCfg0(u8): 0xFFE0;
    impl Debug;
    pub U8Wrapper, from into VoutMode, vout_mode, set_vout_mode: 7,6;
    pub demodu_enhancement, set_demodu_enhancement: 5;
    pub irq_mode, set_irq_mode: 4;
    rfu1, _: 3;
    rfu2, _: 2;
    rfu3, _: 1;
    pub op_mode_select, set_op_mode_select: 0;
}

register_bitfield! {
    #[derive(Clone)]
    struct UserCfg1(u8): 0xFFE1;
    impl Debug;
    pub fast_inventory_en, set_fast_inventory_en: 7;
    pub fdt_comp_en, set_fdt_comp_en: 6;
    pub U8Wrapper, from into Fdel, fdel, set_fdel: 5,4;
    pub U8Wrapper, from into NfcMode, nfc_mode, set_nfc_mode: 3,2;
    rfu1, _: 1;
    pub rfu2, set_rfu2: 0;

}

enum_u8! {
    #[derive(Debug, Clone, Copy)]
    pub enum ArbitrationCfg {
        NoPriority = 0b00,
        FirstCome = 0b01,
        ContactFirst = 0b10,
        ContactLessFirst = 0b11,
    }
}

enum_u8! {
    #[derive(Debug, Clone, Copy)]
    pub enum CtSleepCfg {
        /// 0.5ms
        Point5Ms = 0x0,
        /// 1ms
        Ms1 = 0x1,
        /// 2ms
        Ms2 = 0x2,
        /// 4ms
        Ms4 = 0x3,
        /// 8ms
        Ms8 = 0x4,
        /// 16ms
        Ms16 = 0x5,
        /// 32ms
        Ms32 = 0x6,
        /// 64ms
        Ms64 = 0x7,
        /// always keep the power switch in open status, never enter zero power consumption mode
        AlwaysOpen = 0x8;0x9;0xA;0xB;0xC;0xD;0xE;0xF,
    }
}

register_bitfield! {
    #[derive(Clone)]
    struct UserCfg2(u8): 0xFFE2;
    impl Debug;
    rfu1, _: 7;
    rfu2, _: 6;
    pub U8Wrapper, from into ArbitrationCfg, arbitration_cfg, set_arbitration_cfg: 5,4;
    pub U8Wrapper, from into CtSleepCfg, ct_sleep_cfg, set_ct_sleep_cfg: 3,0;

}

register_bitfield! {
    #[derive(Clone)]
    struct ResetSilence(u8): 0xFFE6;
    impl Debug;
    pub soft_reset, set_soft_reset: 1;
    pub nfc_silence, set_nfc_silence: 0;
}

register_bitfield! {
    #[derive(Clone)]
    struct Status(u8): 0xFFE7;
    impl Debug;
    pub user_cfg_chk_flag, _: 0;
}

register_bitfield! {
    #[derive(Clone)]
    struct VoutEnCfg(u8): 0xFFE9;
    impl Debug;
    pub vout_en, set_vout_en: 7;
}

register_bitfield! {
    #[derive(Clone)]
    struct VoutResCfg(u8): 0xFFEA;
    impl Debug;
    pub vout_res_cfg, set_vout_res_cfg: 7,4;
}

register_bitfield! {
    #[derive(Clone)]
    struct FifoAccess(u8): 0xFFF0;
    impl Debug;
    pub fifo_data, set_fifo_data: 7,0;
}

register_bitfield! {
    #[derive(Clone)]
    struct FifoClear(u8): 0xFFF1;
    impl Debug;
    pub fifo_clear, set_fifo_clear: 7,0;
}

register_bitfield! {
    #[derive(Clone)]
    struct FifoWordCnt(u8): 0xFFF2;
    impl Debug;
    pub fifo_wordcnt, _: 5,0;
}

enum_u8! {
    #[derive(Debug, Clone, Copy)]
    pub enum NfcStatusValue {
        Idle = 0b0000,
        Ready1 = 0b0001,
        Ready2 = 0b0010,
        IdleInventoryProcessing = 0b0011,
        Halt = 0b0100,
        Quiet = 0b0111,
        /// NC mode: in ISO14443-3, or in ISO14443-4 but in front of RATS command
        Iso14443_3OrFrontOfRats = 0b1000,
        Iso14443_4BewtweenRatsAndPPS = 0b1001,
        Iso14443_4AfterPPS = 0b1010,
        NotAuthenticated = 0b1100,
        Authenticated = 0b1101,
    }
}

register_bitfield! {
    #[derive(Clone)]
    struct NfcStatus(u8): 0xFFF3;
    impl Debug;
    pub U8Wrapper, from into NfcStatusValue, nfc_status, set_nfc_status: 7,4;
    pub nfc_dir, set_nfc_dir: 2;
    pub nfc_rx, set_nfc_rx: 1;
    pub nfc_tx, set_nfc_tx: 0;
}

enum_u8! {
    #[derive(Debug, Clone, Copy)]
    pub enum NfcTxenValue {
        SendBackData = 0x55,
        StartPostbackData = 0xAA,
        SwitchToReceiveState = 0x88,
        SwitchToReceiveStateAndIdle = 0x77
    }
}

register_bitfield! {
    #[derive(Clone)]
    struct NfcTxen(u8): 0xFFF4;
    impl Debug;
    pub from NfcTxenValue, _, set_nfc_txn: 7,0;
}

impl NfcTxen {
    pub fn new(value: NfcTxenValue) -> Self {
        let mut this = NfcTxen(0);
        this.set_nfc_txn(value);
        this
    }
}

// Also known as HALT_CTRL
register_bitfield! {
    #[derive(Clone)]
    struct NfcCfg(u8): 0xFFF5;
    impl Debug;
    pub halt_control, set_halt_control: 1;
    pub halt_mode, set_halt_mode: 0;
}

register_bitfield! {
    #[derive(Clone)]
    struct NfcRats(u8): 0xFFF6;
    impl Debug;
    pub fsdi, set_fsdi: 7,4;
    pub cid, set_cid: 3,0;
}

register_bitfield! {
    #[derive(Clone)]
    struct MainIrq(u8): 0xFFF7;
    impl Debug;
    pub power_on_flag, set_power_on_flag: 7;
    pub active_flag, set_active_flag: 6;
    pub rx_start, set_rx_start: 5;
    pub rx_done, set_rx_done: 4;
    pub tx_done, set_tx_done: 3;
    pub arbitration_flag, set_arbitration_flag: 2;
    pub fifo_flag, set_fifo_flag: 1;
    pub aux_flag, set_aux_flag: 0;
}

register_bitfield! {
    #[derive(Clone)]
    struct FifoIrq(u8): 0xFFF8;
    impl Debug;
    pub water_level, set_water_level: 3;
    pub overflow, set_overflow: 2;
    pub full, set_full: 1;
    pub empty, set_empty: 0;
}

register_bitfield! {
    #[derive(Clone)]
    struct AuxIrq(u8): 0xFFF9;
    impl Debug;
    pub write_done, set_write_done: 7;
    pub write_error, set_write_error: 6;
    pub parity_error, set_parity_error: 5;
    pub crc_error, set_crc_error: 4;
    pub framing_error, set_framing_error: 3;
    pub halt_flag, set_halt_flag: 2;
}

register_bitfield! {
    #[derive(Clone)]
    struct MainIrqMask(u8): 0xFFFA;
    impl Debug;
    pub rx_start_mask, set_rx_start_mask: 5;
    pub rx_done_mask, set_rx_done_mask: 4;
    pub tx_done_mask, set_tx_done_mask: 3;
    pub arbitration_flag_mask, set_arbitration_flag_mask: 2;
    pub fifo_flag_mask, set_fifo_flag_mask: 1;
    pub aux_mask, set_aux_mask: 0;
}

register_bitfield! {
    #[derive(Clone)]
    struct FifoIrqMask(u8): 0xFFFB;
    impl Debug;
    pub water_level_mask, set_water_level_mask: 3;
    pub overflow_mask, set_overflow_mask: 2;
    pub full_mask, set_full_mask: 1;
    pub empty_mask, set_empty_mask: 0;
}

register_bitfield! {
    #[derive(Clone)]
    struct AuxIrqMask(u8): 0xFFFC;
    impl Debug;
    pub halt_flag_mask, set_halt_flag_mask: 5;
    pub ee_write_done_mask, set_ee_write_done_mask: 4;
    pub write_error_mask, set_write_error_mask: 3;
    pub parity_error_mask, set_parity_error_mask: 2;
    pub crc_error_mask, set_crc_error_mask: 1;
    pub framing_error_mask, set_framing_error_mask: 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range() {
        let mut fifo_data = FifoAccess(0);
        fifo_data.set_fifo_data(0xFF);
        assert_eq!(fifo_data.0, 0xFF);
        let mut rats = NfcRats(0);
        rats.set_fsdi(0xF);
        assert_eq!(rats.0, 0xF0);
        let mut rats = NfcRats(0);
        rats.set_cid(0xF);
        assert_eq!(rats.0, 0x0F);
    }
}
