use cipher::{
    generic_array::GenericArray,
    typenum::{U16, U32},
    BlockDecryptMut, BlockEncryptMut, BlockSizeUser,
};

pub const SECTOR_SIZE: usize = super::BLOCK_SIZE;
const BLOCK_SIZE: usize = 16;
const BLOCKS_PER_SECTOR: usize = {
    assert!(SECTOR_SIZE.is_multiple_of(BLOCK_SIZE));
    SECTOR_SIZE / BLOCK_SIZE
};

fn blocks(sector: &mut [u8; SECTOR_SIZE]) -> &mut [[u8; BLOCK_SIZE]; BLOCKS_PER_SECTOR] {
    let (blocks, rest) = sector.as_chunks_mut::<BLOCK_SIZE>();
    let blocks: &mut [[u8; BLOCK_SIZE]; BLOCKS_PER_SECTOR] = blocks.try_into().unwrap();
    assert!(rest.is_empty());
    blocks
}

fn cast_slice(data: &mut [[u8; 16]; 32]) -> &mut GenericArray<GenericArray<u8, U16>, U32> {
    // see GenericArray::from_slice_mut
    unsafe { &mut *(data.as_mut_ptr() as *mut GenericArray<GenericArray<u8, U16>, U32>) }
}

fn encrypt_sector_impl<C1, C2>(sector: &mut [u8; SECTOR_SIZE], n: u32, c1: &mut C1, c2: &mut C2)
where
    C1: BlockSizeUser<BlockSize = U16> + BlockEncryptMut,
    C2: BlockSizeUser<BlockSize = U16> + BlockEncryptMut,
{
    let mut tweak = u128::from(n).to_le_bytes();
    c2.encrypt_block_mut(GenericArray::from_mut_slice(&mut tweak));

    let blocks = blocks(sector);
    xor_blocks(blocks, tweak);
    c1.encrypt_blocks_mut(cast_slice(blocks));
    xor_blocks(blocks, tweak);
}

fn decrypt_sector_impl<C1, C2>(sector: &mut [u8; SECTOR_SIZE], n: u32, c1: &mut C1, c2: &mut C2)
where
    C1: BlockSizeUser<BlockSize = U16> + BlockDecryptMut,
    C2: BlockSizeUser<BlockSize = U16> + BlockEncryptMut,
{
    let mut tweak = u128::from(n).to_le_bytes();
    c2.encrypt_block_mut(GenericArray::from_mut_slice(&mut tweak));

    let blocks = blocks(sector);
    xor_blocks(blocks, tweak);
    c1.decrypt_blocks_mut(cast_slice(blocks));
    xor_blocks(blocks, tweak);
}

pub trait Xts128 {
    type C1Dec<'a>: BlockSizeUser<BlockSize = U16> + BlockDecryptMut;
    type C1Enc<'a>: BlockSizeUser<BlockSize = U16> + BlockEncryptMut;
    type C2<'a>: BlockSizeUser<BlockSize = U16> + BlockEncryptMut;

    fn with_dec<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self::C1Dec<'_>, &mut Self::C2<'_>);

    fn with_enc<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self::C1Enc<'_>, &mut Self::C2<'_>);

    fn encrypt_sector(&mut self, sector: &mut [u8; SECTOR_SIZE], n: u32) {
        self.with_enc(|c1, c2| encrypt_sector_impl(sector, n, c1, c2))
    }

    fn encrypt_sectors(&mut self, sectors: &mut [[u8; SECTOR_SIZE]], n: u32) {
        self.with_enc(|c1, c2| {
            for (i, sector) in sectors.iter_mut().enumerate() {
                let i = u32::try_from(i).unwrap();
                encrypt_sector_impl(sector, n + i, c1, c2);
            }
        });
    }

    fn decrypt_sector(&mut self, sector: &mut [u8; SECTOR_SIZE], n: u32) {
        self.with_dec(|c1, c2| decrypt_sector_impl(sector, n, c1, c2))
    }

    fn decrypt_sectors(&mut self, sectors: &mut [[u8; SECTOR_SIZE]], n: u32) {
        self.with_dec(|c1, c2| {
            for (i, sector) in sectors.iter_mut().enumerate() {
                let i = u32::try_from(i).unwrap();
                decrypt_sector_impl(sector, n + i, c1, c2);
            }
        });
    }
}

#[inline(always)]
fn xor_block(block: &mut [u8; BLOCK_SIZE], tweak: &[u8; BLOCK_SIZE]) {
    for i in 0..BLOCK_SIZE {
        block[i] ^= tweak[i];
    }
}

fn xor_blocks(blocks: &mut [[u8; BLOCK_SIZE]], mut tweak: [u8; BLOCK_SIZE]) {
    for block in blocks.iter_mut() {
        xor_block(block, &tweak);
        tweak = galois_field_128_mul_le(tweak);
    }
}

fn galois_field_128_mul_le(tweak: [u8; 16]) -> [u8; 16] {
    let low_bytes = u64::from_le_bytes(tweak[0..8].try_into().unwrap());
    let high_bytes = u64::from_le_bytes(tweak[8..16].try_into().unwrap());
    let new_low_bytes = (low_bytes << 1) ^ if (high_bytes >> 63) != 0 { 0x87 } else { 0x00 };
    let new_high_bytes = (low_bytes >> 63) | (high_bytes << 1);

    let mut tweak = [0; 16];
    tweak[0..8].copy_from_slice(&new_low_bytes.to_le_bytes());
    tweak[8..16].copy_from_slice(&new_high_bytes.to_le_bytes());
    tweak
}

pub struct Ciphers<C1, C2> {
    pub cipher1: C1,
    pub cipher2: C2,
}

impl<C1, C2> Xts128 for Ciphers<C1, C2>
where
    C1: BlockSizeUser<BlockSize = U16> + BlockDecryptMut + BlockEncryptMut,
    C2: BlockSizeUser<BlockSize = U16> + BlockEncryptMut,
{
    type C1Dec<'a> = C1;
    type C1Enc<'a> = C1;
    type C2<'a> = C2;

    fn with_dec<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self::C1Dec<'_>, &mut Self::C2<'_>),
    {
        f(&mut self.cipher1, &mut self.cipher2)
    }

    fn with_enc<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self::C1Enc<'_>, &mut Self::C2<'_>),
    {
        f(&mut self.cipher1, &mut self.cipher2)
    }
}

#[cfg(test)]
mod tests {
    use super::{Ciphers, Xts128 as _, SECTOR_SIZE};

    use aes::{cipher::KeyInit as _, Aes128};

    #[test]
    fn test_xts_sector() {
        let key1 = b"deadbeefdeadbeef";
        let key2 = b"01234567abcdefgh";
        let mut ciphers = Ciphers {
            cipher1: Aes128::new_from_slice(key1).unwrap(),
            cipher2: Aes128::new_from_slice(key2).unwrap(),
        };
        let xts = xts_mode::Xts128::new(
            Aes128::new_from_slice(key1).unwrap(),
            Aes128::new_from_slice(key2).unwrap(),
        );

        let mut sector1 = [0xff; SECTOR_SIZE];
        let mut sector2 = sector1;
        let n = 42;

        ciphers.encrypt_sector(&mut sector1, n);
        xts.encrypt_sector(&mut sector2, xts_mode::get_tweak_default(n.into()));
        assert_eq!(sector1, sector2);

        ciphers.decrypt_sector(&mut sector1, n);
        xts.decrypt_sector(&mut sector2, xts_mode::get_tweak_default(n.into()));
        assert_eq!(sector1, sector2);
    }

    #[test]
    fn test_xts_sectors() {
        let key1 = b"deadbeefdeadbeef";
        let key2 = b"01234567abcdefgh";
        let mut ciphers = Ciphers {
            cipher1: Aes128::new_from_slice(key1).unwrap(),
            cipher2: Aes128::new_from_slice(key2).unwrap(),
        };
        let xts = xts_mode::Xts128::new(
            Aes128::new_from_slice(key1).unwrap(),
            Aes128::new_from_slice(key2).unwrap(),
        );

        let mut sectors1 = [
            [0xff; SECTOR_SIZE],
            [0x00; SECTOR_SIZE],
            [0xab; SECTOR_SIZE],
        ];
        let mut sectors2 = sectors1;
        let n = 42;

        ciphers.encrypt_sectors(&mut sectors1, n);
        xts.encrypt_area(
            sectors2.as_flattened_mut(),
            SECTOR_SIZE,
            n.into(),
            xts_mode::get_tweak_default,
        );
        assert_eq!(sectors1, sectors2);

        ciphers.decrypt_sectors(&mut sectors1, n);
        xts.decrypt_area(
            sectors2.as_flattened_mut(),
            SECTOR_SIZE,
            n.into(),
            xts_mode::get_tweak_default,
        );
        assert_eq!(sectors1, sectors2);
    }
}
