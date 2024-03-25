const fn sequence<const N: usize>(base: u16, end: u16) -> [u16; N] {
    assert!(N as u16 == end - base + 1);
    let mut arr = [0; N];
    let mut i = 0;
    while i < N {
        arr[i] = base + i as u16;
        i += 1;
    }

    arr
}

pub const ORIGINALITY_SIGNATURE_BLOCKS: [u16; 8] = sequence(0x1000, 0x1007);
pub const CH: u16 = 0x1008;
pub const CID: u16 = 0x1009;
pub const NFC_GCH: u16 = 0x100C;
pub const NFC_CCH: u16 = 0x100D;
pub const NFC_AUTH_LIMIT: u16 = 0x100E;
pub const NFC_KH0: u16 = 0x1010;
pub const NFC_KP0: u16 = 0x1011;
pub const NFC_KH1: u16 = 0x1012;
pub const NFC_KP1: u16 = 0x1013;
pub const NFC_KH2: u16 = 0x1014;
pub const NFC_KP2: u16 = 0x1015;
pub const NFC_KH3: u16 = 0x1016;
pub const NFC_KP3: u16 = 0x1017;
pub const AES_KEY_0: [u16; 4] = sequence(0x1020, 0x1023);
pub const AES_KEY_1: [u16; 4] = sequence(0x1024, 0x1027);
pub const AES_KEY_2: [u16; 4] = sequence(0x1028, 0x102B);
pub const AES_KEY_3: [u16; 4] = sequence(0x102C, 0x102F);
pub const I2C_KH: u16 = 0x1030;
pub const I2C_PP_PPC: u16 = 0x1031;
pub const I2C_AUTH_LIMIT: u16 = 0x1032;
pub const I2C_PWD_0: u16 = 0x1033;
pub const I2C_PWD_1: u16 = 0x1034;
pub const I2C_PWD_2: u16 = 0x1035;
pub const I2C_PWD_3: u16 = 0x1036;
pub const CONFIG: u16 = 0x1037;
pub const SYNC_DATA_BLOCK: u16 = 0x1038;
