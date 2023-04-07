use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};

use crate::types::build_constants::{
    CONFIG_FILESYSTEM_BOUNDARY as FS_BASE, CONFIG_FILESYSTEM_END as FS_CEIL,
};

pub const FLASH_BASE: *mut u8 = FS_BASE as *mut u8;
pub const FLASH_SIZE: usize = FS_CEIL - FS_BASE;

const REAL_BLOCK_SIZE: usize = 4 * 1024;

const FTL_BLOCK_SIZE: usize = 256;
const FTL_JOURNAL_BLOCKS: usize = 2;
const FTL_BLOCKS_IN_REAL: usize = REAL_BLOCK_SIZE / FTL_BLOCK_SIZE;

// relative to FLASH_BASE
const FTL_JOURNAL_START: u32 = 0;

#[derive(Clone)]
pub struct JournalInfo {
    pub idx: u32,
    pub cnt: u32,
}

pub struct FlashStorage {
    pub nvmc: nrf52840_hal::nvmc::Nvmc<nrf52840_pac::NVMC>,
    next_journal: Option<JournalInfo>,
}

impl FlashStorage {
    pub fn format_journal_blocks(&mut self) {
        // erase entire journal region
        self.nvmc
            .erase(
                FTL_JOURNAL_START,
                (FTL_JOURNAL_BLOCKS * REAL_BLOCK_SIZE) as u32,
            )
            .unwrap();

        // format, for each block:
        // * write counter, which equals the block idx
        // * write 0x00 into remaining block
        let buf: [u8; FTL_BLOCK_SIZE - 4] = [0x00; (FTL_BLOCK_SIZE - 4)];
        for idx in 0..FTL_JOURNAL_BLOCKS as u32 {
            let addr = FTL_JOURNAL_START + (REAL_BLOCK_SIZE * idx as usize) as u32;
            self.nvmc.write(addr, &idx.to_be_bytes()).unwrap();
            self.nvmc.write(addr + 4, &buf).unwrap();
        }
    }

    pub fn get_next_journal(&mut self) -> JournalInfo {
        // search journal blocks for max(cnt)
        if self.next_journal.is_none() {
            let (min_idx, min_cnt) = (0..FTL_JOURNAL_BLOCKS as u32)
                .map(|idx| (idx, self.get_journal_cnt(idx)))
                .min_by_key(|item| item.1)
                .unwrap();
            // ... and set `next_journal` accordingly
            self.next_journal = Some(JournalInfo {
                idx: min_idx,
                cnt: min_cnt,
            });
        }
        // common case: just return `next_journal`
        trace!("cnt: {}", self.next_journal.clone().unwrap().cnt);
        self.next_journal.clone().unwrap()
    }

    pub fn get_last_journal(&mut self) -> JournalInfo {
        // for recovery the latest written (max counter) block is needed
        let (max_idx, max_cnt) = (0..FTL_JOURNAL_BLOCKS as u32)
            .map(|idx| (idx, self.get_journal_cnt(idx)))
            .max_by_key(|item| item.1)
            .unwrap();

        JournalInfo {
            idx: max_idx,
            cnt: max_cnt,
        }
    }

    pub fn get_journal_cnt(&mut self, block_idx: u32) -> u32 {
        // read counter for given `block_idx` from nvm
        let mut buf: [u8; 4] = [0x00; 4];
        let block_addr = FTL_JOURNAL_START + (REAL_BLOCK_SIZE * block_idx as usize) as u32;
        self.nvmc.read(block_addr, &mut buf).unwrap();

        // construct counter
        let mut jrnl_cnt_raw: [u8; 4] = [0u8; 4];
        jrnl_cnt_raw.copy_from_slice(&buf);
        let jrnl_cnt = u32::from_be_bytes(jrnl_cnt_raw);

        // freshly erased block? 0xff == u32::MAX
        // if so, format all journal blocks
        if jrnl_cnt == u32::MAX {
            self.format_journal_blocks();
            trace!("FORMATTING JOURNAL BLOCKS ????");
            // default cnt after formatting == block idx
            block_idx
        } else {
            jrnl_cnt
        }
    }

    pub fn write_journal(&mut self, src_addr: usize, data: &[u8], lhs_len: usize, rhs_off: usize) {
        // Writing the journal, contents
        // * counter     => overall journal pages written, smallest used for next
        // * source addr => where to write-back to in case of recovery
        // * lhs len     => length of left part
        // * rhs off     => offset of right part (relative to block)
        // * lhs data    => lhs journal data
        // * rhs data    => rhs journal data

        // get journal block & calc its address
        let jinfo = self.get_next_journal();
        let addr = FTL_JOURNAL_START + (jinfo.idx * REAL_BLOCK_SIZE as u32);

        // erase journal block
        self.nvmc
            .erase(addr as u32, addr + REAL_BLOCK_SIZE as u32)
            .unwrap();

        // write meta-data
        let write_vals = [jinfo.cnt, src_addr as u32, lhs_len as u32, rhs_off as u32];
        for (idx, data) in (0..4).zip(write_vals) {
            self.nvmc
                .write(addr + idx * 4, &data.to_be_bytes())
                .unwrap();
        }

        // write lhs + rhs for journaled block
        if lhs_len > 0 {
            self.nvmc
                .write((addr + 16) as u32, &data[..lhs_len])
                .unwrap();
        }

        let rhs_len = (REAL_BLOCK_SIZE - rhs_off) as u32;
        if rhs_len > 0 {
            self.nvmc
                .write(addr + 16 + lhs_len as u32, &data[rhs_off..])
                .unwrap();
        }

        trace!(
            "wrote jrnl {FTL_JOURNAL_START:x} {src_addr:x} {lhs_len:x} {rhs_off:x} {:x}",
            data.len()
        );

        // move next journal info to next slot + inc counter
        self.next_journal = Some(JournalInfo {
            idx: (jinfo.idx + 1) % (FTL_JOURNAL_BLOCKS as u32),
            cnt: jinfo.cnt + 1,
        });
    }

    pub fn recover_from_journal(&mut self) -> bool {
        trace!("RECOVERING FROM JOURNAL");

        // get last written journal block & calc its address
        let jinfo = self.get_last_journal();
        let addr = FTL_JOURNAL_START + jinfo.idx * REAL_BLOCK_SIZE as u32;

        // read entire block at once
        let mut buf: [u8; REAL_BLOCK_SIZE] = [0x00; REAL_BLOCK_SIZE];
        self.nvmc.read(addr, &mut buf).unwrap();

        // read meta-data
        let mut cnt_raw: [u8; 4] = [0u8; 4];
        cnt_raw.copy_from_slice(&buf[..4]);
        let _cnt = u32::from_be_bytes(cnt_raw);

        let mut src_addr_raw: [u8; 4] = [0u8; 4];
        src_addr_raw.copy_from_slice(&buf[4..8]);
        let src_addr = u32::from_be_bytes(src_addr_raw);

        let mut lhs_len_raw: [u8; 4] = [0u8; 4];
        lhs_len_raw.copy_from_slice(&buf[8..12]);
        let lhs_len = u32::from_be_bytes(lhs_len_raw);

        let mut rhs_off_raw: [u8; 4] = [0u8; 4];
        rhs_off_raw.copy_from_slice(&buf[12..16]);
        let rhs_off = u32::from_be_bytes(rhs_off_raw);
        let rhs_len = REAL_BLOCK_SIZE - rhs_off as usize;

        // erase target block inside littlefs2 space
        self.nvmc
            .erase(src_addr as u32, (src_addr + REAL_BLOCK_SIZE as u32) as u32)
            .unwrap();

        // write back journaled lhs + rhs
        if lhs_len > 0 {
            self.nvmc
                .write(src_addr, &buf[16..((lhs_len + 16) as usize)])
                .unwrap();
        }
        if rhs_len > 0 {
            self.nvmc
                .write(
                    src_addr + rhs_off,
                    &buf[((lhs_len + 16) as usize)..((rhs_len + 16 + lhs_len as usize) as usize)],
                )
                .unwrap();
        }
        trace!("RECOVER complete count: {_cnt} idx: {}", jinfo.idx);

        true
    }
}

impl littlefs2::driver::Storage for FlashStorage {
    const BLOCK_SIZE: usize = FTL_BLOCK_SIZE;
    const READ_SIZE: usize = 4;
    const WRITE_SIZE: usize = FTL_BLOCK_SIZE;
    const BLOCK_COUNT: usize =
        (FLASH_SIZE / Self::BLOCK_SIZE) - (FTL_BLOCKS_IN_REAL * FTL_JOURNAL_BLOCKS);

    type CACHE_SIZE = generic_array::typenum::U256;
    type LOOKAHEADWORDS_SIZE = generic_array::typenum::U2;

    fn read(&mut self, off: usize, buf: &mut [u8]) -> Result<usize, littlefs2::io::Error> {
        // skip journal blocks
        let off = off + (REAL_BLOCK_SIZE * FTL_JOURNAL_BLOCKS);

        let res = self.nvmc.read(off as u32, buf);
        nvmc_to_lfs_return(res, buf.len())
    }

    fn write(&mut self, off: usize, buf: &[u8]) -> Result<usize, littlefs2::io::Error> {
        // skip journal blocks
        let off = off + (REAL_BLOCK_SIZE * FTL_JOURNAL_BLOCKS);

        trace!("IFw {:x} {:x}", off, buf.len());
        let res = self.nvmc.write(off as u32, buf);
        nvmc_to_lfs_return(res, buf.len())
    }

    fn erase(&mut self, off: usize, len: usize) -> Result<usize, littlefs2::io::Error> {
        trace!("IFe {:x} {:x}", off, len);

        // skip journal blocks
        let off = off + (REAL_BLOCK_SIZE * FTL_JOURNAL_BLOCKS);

        let block_off: usize = off - (off % REAL_BLOCK_SIZE);
        let left_end: usize = off - block_off;
        let right_off: usize = left_end + len;

        let mut buf: [u8; REAL_BLOCK_SIZE] = [0x00; REAL_BLOCK_SIZE];

        self.nvmc.read(block_off as u32, &mut buf).unwrap();

        self.write_journal(block_off, &buf, left_end, right_off);

        let erase_res = self
            .nvmc
            .erase(block_off as u32, (block_off + REAL_BLOCK_SIZE) as u32);

        if left_end > 0 {
            self.nvmc.write(block_off as u32, &buf[..left_end]).unwrap();
        }

        if REAL_BLOCK_SIZE - right_off > 0 {
            self.nvmc
                .write((off + len) as u32, &buf[right_off..])
                .unwrap();
        }

        nvmc_to_lfs_return(erase_res, len)
    }
}

/**
 * Source Result type does not provide a useful Ok value, and Destination Result type
 * does not contain a meaningful low-level error code we could return; so here goes
 * the most stupid result conversion routine ever
 */
fn nvmc_to_lfs_return(
    r: Result<(), nrf52840_hal::nvmc::NvmcError>,
    len: usize,
) -> Result<usize, littlefs2::io::Error> {
    r.map(|_| len)
        .map_err(|_| littlefs2::io::Error::Unknown(0x4e56_4d43)) // 'NVMC'
}

impl FlashStorage {
    pub fn new(nvmc_pac: nrf52840_hal::pac::NVMC) -> Self {
        let buf = unsafe { core::slice::from_raw_parts_mut(FLASH_BASE, FLASH_SIZE) };
        let nvmc = nrf52840_hal::nvmc::Nvmc::new(nvmc_pac, buf);

        Self {
            nvmc,
            next_journal: None,
        }
    }
}
