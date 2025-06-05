// use bitfield::bitfield;

use array_tuple_concat::concat_arrays;

#[derive(Debug)]
pub struct TryFromU8Error;

// For bitfield compatibility
// pub struct U8Wrapper(u8);

fn extract_array<const A: usize, const B: usize>(block: [u8; 16]) -> [u8; B] {
    const {
        assert!(A <= 16);
        assert!(A + B <= 16);
    }
    block[A..][..B].try_into().unwrap()
}

macro_rules! subarray {
    ($block:ident[$start:literal..$length:literal]) => {
        extract_array::<$start, const {$length - $start }>($block)
    };
    ($block:ident[$start:literal..][..$length:literal]) => {
        extract_array::<$start, $length>($block)
    };
    ($block:expr, $start:literal, $length:literal) => {
        extract_array::<$start, $length>($block)
    };
}

pub trait Block {
    const ADDRESS: u8;
    fn parse(block: [u8; 16]) -> Self;
    fn serialize(self) -> [u8; 16];
}

#[derive(Debug, Clone, Copy)]
pub struct Metadata {
    pub addr: u8,
    pub serial_number: [u8; 6],
    pub internal: [u8; 3],
    pub static_lock_bytes: [u8; 2],
    pub capacibility_container: [u8; 4],
}

impl Block for Metadata {
    const ADDRESS: u8 = 0;
    fn parse(block: [u8; 16]) -> Self {
        Self {
            addr: block[0],
            serial_number: subarray!(block, 1, 6),
            internal: subarray!(block, 7, 3),
            static_lock_bytes: subarray!(block, 10, 2),
            capacibility_container: subarray!(block, 12, 4),
        }
    }

    fn serialize(self) -> [u8; 16] {
        concat_arrays((
            [self.addr],
            self.serial_number,
            self.internal,
            self.static_lock_bytes,
            self.capacibility_container,
        ))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DynamicLockBytes {
    pub protected_user_memory: [u8; 4],
    pub dynamic_lock_bytes: [u8; 3],
    pub zero_byte: u8,
    pub rfu: [u8; 3],
    pub auth0: u8,
    pub unspecified: [u8; 4],
}

impl Block for DynamicLockBytes {
    const ADDRESS: u8 = 0x38;
    fn parse(block: [u8; 16]) -> Self {
        Self {
            protected_user_memory: subarray!(block, 0, 4),
            dynamic_lock_bytes: subarray!(block, 4, 3),
            zero_byte: block[7],
            rfu: subarray!(block, 8, 3),
            auth0: block[11],
            unspecified: subarray!(block, 12, 4),
        }
    }

    fn serialize(self) -> [u8; 16] {
        concat_arrays((
            self.protected_user_memory,
            self.dynamic_lock_bytes,
            [self.zero_byte],
            self.rfu,
            [self.auth0],
            self.unspecified,
        ))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Access {
    pub access: u8,
    pub rfu: [u8; 3],
    pub pwd: [u8; 4],
    pub pack: [u8; 2],
    pub rfu2: [u8; 2],
    pub pt_i2c: u8,
    pub rfu3: [u8; 3],
}

impl Block for Access {
    const ADDRESS: u8 = 0x39;
    fn parse(block: [u8; 16]) -> Self {
        Self {
            access: block[0],
            rfu: subarray!(block, 1, 3),
            pwd: subarray!(block, 4, 4),
            pack: subarray!(block, 8, 2),
            rfu2: subarray!(block, 10, 2),
            pt_i2c: block[12],
            rfu3: subarray!(block, 13, 3),
        }
    }
    fn serialize(self) -> [u8; 16] {
        concat_arrays((
            [self.access],
            self.rfu,
            self.pwd,
            self.pack,
            self.rfu2,
            [self.pt_i2c],
            self.rfu3,
        ))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConfigurationRegisters {
    pub nc_reg: u8,
    pub last_ndef_block: u8,
    pub sram_mirror_block: u8,
    pub wdt_ls: u8,
    pub wdt_ms: u8,
    pub i2c_clock_str: u8,
    pub reg_lock: u8,
    pub rfu: u8,
    pub zeros: [u8; 8],
}

impl Block for ConfigurationRegisters {
    const ADDRESS: u8 = 0x3A;
    fn parse(block: [u8; 16]) -> Self {
        let [nc_reg, last_ndef_block, sram_mirror_block, wdt_ls, wdt_ms, i2c_clock_str, reg_lock, rfu, ..] =
            block;
        Self {
            nc_reg,
            last_ndef_block,
            sram_mirror_block,
            wdt_ls,
            wdt_ms,
            i2c_clock_str,
            reg_lock,
            rfu,
            zeros: subarray!(block, 8, 8),
        }
    }

    fn serialize(self) -> [u8; 16] {
        concat_arrays((
            [
                self.nc_reg,
                self.last_ndef_block,
                self.sram_mirror_block,
                self.wdt_ls,
                self.wdt_ms,
                self.i2c_clock_str,
                self.reg_lock,
                self.rfu,
            ],
            self.zeros,
        ))
    }
}

pub struct SessionRegisters {
    pub nc_reg: u8,
    pub last_ndef_block: u8,
    pub sram_mirror_block: u8,
    pub wdt_ls: u8,
    pub wdt_ms: u8,
    pub i2c_clock_str: u8,
    pub reg_lock: u8,
    pub rfu: u8,
    pub zeros: [u8; 8],
}

impl Block for SessionRegisters {
    const ADDRESS: u8 = 0xFE;
    fn parse(block: [u8; 16]) -> Self {
        let [nc_reg, last_ndef_block, sram_mirror_block, wdt_ls, wdt_ms, i2c_clock_str, reg_lock, rfu, ..] =
            block;
        Self {
            nc_reg,
            last_ndef_block,
            sram_mirror_block,
            wdt_ls,
            wdt_ms,
            i2c_clock_str,
            reg_lock,
            rfu,
            zeros: subarray!(block, 8, 8),
        }
    }

    fn serialize(self) -> [u8; 16] {
        concat_arrays((
            [
                self.nc_reg,
                self.last_ndef_block,
                self.sram_mirror_block,
                self.wdt_ls,
                self.wdt_ms,
                self.i2c_clock_str,
                self.reg_lock,
                self.rfu,
            ],
            self.zeros,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::proptest;
    use std::collections::HashSet;

    #[test]
    fn duplicate_address() {
        let all = [
            Metadata::ADDRESS,
            DynamicLockBytes::ADDRESS,
            Access::ADDRESS,
            ConfigurationRegisters::ADDRESS,
            SessionRegisters::ADDRESS,
        ];

        let mut uniq = HashSet::new();
        assert!(all.into_iter().all(move |addr| uniq.insert(addr)));
    }

    proptest! {
        #[test]
        fn test_rountrip(block in [const {0..u8::MAX}; 16]) {
            fn block_roundtrip_check<T: Block>(block: [u8; 16]) {
                assert_eq!(
                    T::parse(block).serialize(),
                    block,
                    "Block address: {}",
                    T::ADDRESS
                );
            }

            block_roundtrip_check::<Metadata>(block);
            block_roundtrip_check::<DynamicLockBytes>(block);
            block_roundtrip_check::<Access>(block);
            block_roundtrip_check::<ConfigurationRegisters>(block);
            block_roundtrip_check::<SessionRegisters>(block);
        }
    }
}
