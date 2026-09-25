use core::fmt;

use stm32n657_hal::{
    mmc::{MmcMaster, MmcPins, BLOCK_SIZE},
    sdmmc::{Enabled, Error, SdMmc},
};

const BLOCK: usize = BLOCK_SIZE as usize;
type Block = [u8; BLOCK];

const MAX_BLOCKS: usize = 8;
const EMPTY_BYTE: u8 = 0xA5;

const SINGLE_ADDR: u32 = 8;
const MULTI_ADDR: u32 = 16;
const CROSS_ADDR: u32 = 24;
const NEIGHBOUR_A_ADDR: u32 = 32;
const NEIGHBOUR_B_ADDR: u32 = 40;
const TEST_REGION_END: u32 = 48;

const SWEEP_BLOCKS: u32 = 1024;
const REPEAT_READS: u32 = 64;

pub struct Buffers {
    pub write: [Block; MAX_BLOCKS],
    pub read: [Block; MAX_BLOCKS],
}

pub enum Failure {
    Driver {
        op: &'static str,
        block: u32,
        err: Error,
    },
    Mismatch {
        block: u32,
        offset: usize,
        expected: u8,
        got: u8,
    },
    WrongResult {
        what: &'static str,
        got: Result<(), Error>,
    },
    Check(&'static str),
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Driver { op, block, err } => write!(f, "{op} at block {block}: {err:?}"),
            Self::Mismatch {
                block,
                offset,
                expected,
                got,
            } => write!(
                f,
                "block {block} byte {offset}: expected {expected:#04x}, got {got:#04x}"
            ),
            Self::WrongResult { what, got } => write!(f, "{what}: got {got:?}"),
            Self::Check(what) => write!(f, "check failed: {what}"),
        }
    }
}

type Mmc<'a, P, Pins> = &'a mut MmcMaster<P, Pins, Enabled>;
type Case<P, Pins> = fn(Mmc<'_, P, Pins>, &mut Buffers) -> Result<(), Failure>;

/// run all
pub fn run<P: SdMmc, Pins: MmcPins<Peripheral = P>>(mmc: Mmc<'_, P, Pins>) {
    let cases: &[(&str, Case<P, Pins>)] = &[
        ("card_info", card_info),
        ("cold_read", cold_read),
        ("read_sweep", read_sweep),
        ("read_scatter", read_scatter),
        ("single_block", single_block),
        ("multi_block", multi_block),
        ("cross_paths", cross_paths),
        ("neighbour_isolation", neighbour_isolation),
        ("last_block", last_block),
        ("out_of_range", out_of_range),
    ];
    let mut bufs = Buffers {
        write: [[0; BLOCK]; MAX_BLOCKS],
        read: [[0; BLOCK]; MAX_BLOCKS],
    };

    info_now!(
        "sdmmc tests: {:?} card, {} cases",
        mmc.card_kind(),
        cases.len()
    );
    let mut failed = 0;
    for (i, (name, case)) in cases.iter().enumerate() {
        info_now!("sdmmc test {}/{} {}: start", i + 1, cases.len(), name);
        match case(mmc, &mut bufs) {
            Ok(()) => {
                info_now!("sdmmc test {}/{} {}: PASS", i + 1, cases.len(), name);
            }
            Err(failure) => {
                failed += 1;
                error_now!(
                    "sdmmc test {}/{} {}: FAIL: {}",
                    i + 1,
                    cases.len(),
                    name,
                    failure
                );
            }
        }
    }
    info_now!(
        "sdmmc tests: {} passed, {} failed",
        cases.len() - failed,
        failed
    );
    if failed > 0 {
        panic!("{failed} sdmmc tests failed");
    }
}

/// fills a block with a xorshift32 stream unique to (block address, seed)
fn fill(block: &mut Block, addr: u32, seed: u32) {
    let mut x = (addr.wrapping_mul(0x9E37_79B9) ^ seed) | 1;
    for word in block.chunks_exact_mut(4) {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        word.copy_from_slice(&x.to_le_bytes());
    }
}

fn verify(block: &Block, addr: u32, seed: u32) -> Result<(), Failure> {
    let mut expected = [0; BLOCK];
    fill(&mut expected, addr, seed);
    match block.iter().zip(expected.iter()).position(|(a, b)| a != b) {
        None => Ok(()),
        Some(offset) => Err(Failure::Mismatch {
            block: addr,
            offset,
            expected: expected[offset],
            got: block[offset],
        }),
    }
}

fn fill_blocks(blocks: &mut [Block], addr: u32, seed: u32) {
    for (i, block) in blocks.iter_mut().enumerate() {
        fill(block, addr + i as u32, seed);
    }
}

fn verify_blocks(blocks: &[Block], addr: u32, seed: u32) -> Result<(), Failure> {
    for (i, block) in blocks.iter().enumerate() {
        verify(block, addr + i as u32, seed)?;
    }
    Ok(())
}

fn write<P: SdMmc, Pins: MmcPins<Peripheral = P>>(
    mmc: Mmc<'_, P, Pins>,
    blocks: &[Block],
    addr: u32,
) -> Result<(), Failure> {
    mmc.write_blocks(blocks, addr)
        .map_err(|err| Failure::Driver {
            op: "write",
            block: addr,
            err,
        })
}

/// reads into a "empty-byte" buffer; avoid accidental match
fn read<P: SdMmc, Pins: MmcPins<Peripheral = P>>(
    mmc: Mmc<'_, P, Pins>,
    blocks: &mut [Block],
    addr: u32,
) -> Result<(), Failure> {
    blocks.as_flattened_mut().fill(EMPTY_BYTE);
    mmc.read_blocks(blocks, addr)
        .map_err(|err| Failure::Driver {
            op: "read",
            block: addr,
            err,
        })
}

/// write/read/verify `n` blocks at `addr` twice with different seeds
fn round_trip<P: SdMmc, Pins: MmcPins<Peripheral = P>>(
    mmc: Mmc<'_, P, Pins>,
    bufs: &mut Buffers,
    addr: u32,
    n: usize,
    seed: u32,
) -> Result<(), Failure> {
    for seed in [seed, !seed] {
        // No logging between write and read: the read must cope with a card still busy.
        info_now!("  round trip {} blocks at {}", n, addr);
        fill_blocks(&mut bufs.write[..n], addr, seed);
        write(mmc, &bufs.write[..n], addr)?;
        read(mmc, &mut bufs.read[..n], addr)?;
        verify_blocks(&bufs.read[..n], addr, seed)?;
    }
    Ok(())
}

fn expect_err(what: &'static str, got: Result<(), Error>, expected: Error) -> Result<(), Failure> {
    if got == Err(expected) {
        Ok(())
    } else {
        Err(Failure::WrongResult { what, got })
    }
}

fn card_info<P: SdMmc, Pins: MmcPins<Peripheral = P>>(
    mmc: Mmc<'_, P, Pins>,
    _bufs: &mut Buffers,
) -> Result<(), Failure> {
    let blocks = mmc.block_count();
    info_now!(
        "card: {:?}, {} blocks of {} bytes ({} MiB)",
        mmc.card_kind(),
        blocks,
        mmc.log_block_size(),
        blocks / (1024 * 1024 / BLOCK_SIZE)
    );
    if mmc.log_block_size() != BLOCK_SIZE {
        return Err(Failure::Check("logical block size is not 512"));
    }
    if blocks < TEST_REGION_END + MAX_BLOCKS as u32 {
        return Err(Failure::Check("card too small for the test region"));
    }
    Ok(())
}

/// reads `addr` twice into both buffers; must be stable
fn read_stable<P: SdMmc, Pins: MmcPins<Peripheral = P>>(
    mmc: Mmc<'_, P, Pins>,
    bufs: &mut Buffers,
    addr: u32,
    n: usize,
) -> Result<(), Failure> {
    read(mmc, &mut bufs.read[..n], addr)?;
    read(mmc, &mut bufs.write[..n], addr)?;
    compare(&bufs.read[..n], &bufs.write[..n], addr)
}

fn compare(a: &[Block], b: &[Block], addr: u32) -> Result<(), Failure> {
    for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
        if let Some(offset) = x.iter().zip(y.iter()).position(|(p, q)| p != q) {
            return Err(Failure::Mismatch {
                block: addr + i as u32,
                offset,
                expected: x[offset],
                got: y[offset],
            });
        }
    }
    Ok(())
}

/// first accesses after power-up, no write before: mount-like single reads
fn cold_read<P: SdMmc, Pins: MmcPins<Peripheral = P>>(
    mmc: Mmc<'_, P, Pins>,
    bufs: &mut Buffers,
) -> Result<(), Failure> {
    let blocks = mmc.block_count();
    // some often used addresses: MBR, boot sector candidates, FAT start, mid card, last block
    let addrs = [0, 1, 2, 8, 63, 64, 2048, 8192, blocks / 2, blocks - 1];
    for addr in addrs {
        if addr >= blocks {
            continue;
        }
        info_now!("  cold single read at {}", addr);
        read_stable(mmc, bufs, addr, 1)?;
    }
    info_now!("  cold multi read at 0");
    read_stable(mmc, bufs, 0, MAX_BLOCKS)?;
    // multi read must agree with the single
    for i in 0..MAX_BLOCKS {
        read(mmc, &mut bufs.write[..1], i as u32)?;
        compare(&bufs.read[i..=i], &bufs.write[..1], i as u32)?;
    }
    Ok(())
}

/// many single reads + same range as multi reads
fn read_sweep<P: SdMmc, Pins: MmcPins<Peripheral = P>>(
    mmc: Mmc<'_, P, Pins>,
    bufs: &mut Buffers,
) -> Result<(), Failure> {
    let end = SWEEP_BLOCKS.min(mmc.block_count());
    for addr in 0..end {
        if addr % 256 == 0 {
            info_now!("  single sweep at {}/{}", addr, end);
        }
        read(mmc, &mut bufs.read[..1], addr)?;
    }
    let mut addr = 0;
    while addr + MAX_BLOCKS as u32 <= end {
        if addr % 256 == 0 {
            info_now!("  multi sweep at {}/{}", addr, end);
        }
        read(mmc, &mut bufs.read, addr)?;
        addr += MAX_BLOCKS as u32;
    }
    // block 0 must still read the same after the whole sweep
    read_stable(mmc, bufs, 0, 1)
}

/// far-apart addresses and re-read many times
fn read_scatter<P: SdMmc, Pins: MmcPins<Peripheral = P>>(
    mmc: Mmc<'_, P, Pins>,
    bufs: &mut Buffers,
) -> Result<(), Failure> {
    let blocks = mmc.block_count();
    let far = [0, blocks - 1, blocks / 2, 1, blocks - MAX_BLOCKS as u32, 2];
    info_now!("  {} rounds over {:?}", REPEAT_READS / 8, far);
    for _ in 0..REPEAT_READS / 8 {
        for addr in far {
            read(mmc, &mut bufs.read[..1], addr)?;
        }
    }
    info_now!("  {} re-reads of block 0", REPEAT_READS);
    read(mmc, &mut bufs.write[..1], 0)?;
    for _ in 0..REPEAT_READS {
        read(mmc, &mut bufs.read[..1], 0)?;
        compare(&bufs.read[..1], &bufs.write[..1], 0)?;
    }
    Ok(())
}

fn single_block<P: SdMmc, Pins: MmcPins<Peripheral = P>>(
    mmc: Mmc<'_, P, Pins>,
    bufs: &mut Buffers,
) -> Result<(), Failure> {
    round_trip(mmc, bufs, SINGLE_ADDR, 1, 0x0001)
}

fn multi_block<P: SdMmc, Pins: MmcPins<Peripheral = P>>(
    mmc: Mmc<'_, P, Pins>,
    bufs: &mut Buffers,
) -> Result<(), Failure> {
    round_trip(mmc, bufs, MULTI_ADDR, MAX_BLOCKS, 0x0002)
}

/// multi write vs single read, then single write vs multi read
fn cross_paths<P: SdMmc, Pins: MmcPins<Peripheral = P>>(
    mmc: Mmc<'_, P, Pins>,
    bufs: &mut Buffers,
) -> Result<(), Failure> {
    const MULTI_SEED: u32 = 0x0003;
    const SINGLE_SEED: u32 = 0x0004;

    fill_blocks(&mut bufs.write, CROSS_ADDR, MULTI_SEED);
    write(mmc, &bufs.write, CROSS_ADDR)?;
    read(mmc, &mut bufs.read[..1], CROSS_ADDR)?;
    verify(&bufs.read[0], CROSS_ADDR, MULTI_SEED)?;

    fill(&mut bufs.write[0], CROSS_ADDR, SINGLE_SEED);
    write(mmc, &bufs.write[..1], CROSS_ADDR)?;
    read(mmc, &mut bufs.read, CROSS_ADDR)?;
    verify(&bufs.read[0], CROSS_ADDR, SINGLE_SEED)?;
    verify_blocks(&bufs.read[1..], CROSS_ADDR + 1, MULTI_SEED)
}

/// write one block -> must not change another one
fn neighbour_isolation<P: SdMmc, Pins: MmcPins<Peripheral = P>>(
    mmc: Mmc<'_, P, Pins>,
    bufs: &mut Buffers,
) -> Result<(), Failure> {
    const SEED_A: u32 = 0x0005;
    const SEED_B: u32 = 0x0006;
    const SEED_A2: u32 = 0x0007;

    fill(&mut bufs.write[0], NEIGHBOUR_A_ADDR, SEED_A);
    write(mmc, &bufs.write[..1], NEIGHBOUR_A_ADDR)?;
    fill(&mut bufs.write[0], NEIGHBOUR_B_ADDR, SEED_B);
    write(mmc, &bufs.write[..1], NEIGHBOUR_B_ADDR)?;

    read(mmc, &mut bufs.read[..1], NEIGHBOUR_A_ADDR)?;
    verify(&bufs.read[0], NEIGHBOUR_A_ADDR, SEED_A)?;
    read(mmc, &mut bufs.read[..1], NEIGHBOUR_B_ADDR)?;
    verify(&bufs.read[0], NEIGHBOUR_B_ADDR, SEED_B)?;

    fill(&mut bufs.write[0], NEIGHBOUR_A_ADDR, SEED_A2);
    write(mmc, &bufs.write[..1], NEIGHBOUR_A_ADDR)?;
    read(mmc, &mut bufs.read[..1], NEIGHBOUR_B_ADDR)?;
    verify(&bufs.read[0], NEIGHBOUR_B_ADDR, SEED_B)?;
    read(mmc, &mut bufs.read[..1], NEIGHBOUR_A_ADDR)?;
    verify(&bufs.read[0], NEIGHBOUR_A_ADDR, SEED_A2)
}

/// highest 8-aligned block; wrong byte/block address scaling fails
fn last_block<P: SdMmc, Pins: MmcPins<Peripheral = P>>(
    mmc: Mmc<'_, P, Pins>,
    bufs: &mut Buffers,
) -> Result<(), Failure> {
    let blocks = mmc.block_count();
    let last = (blocks / 8 - 1) * 8;
    info_now!("last block region starts at {}", last);
    round_trip(mmc, bufs, last, 1, 0x0008)?;
    if last + MAX_BLOCKS as u32 <= blocks {
        round_trip(mmc, bufs, last, MAX_BLOCKS, 0x0009)?;
    }
    Ok(())
}

/// accesses past the end are rejected and the driver stays usabled
fn out_of_range<P: SdMmc, Pins: MmcPins<Peripheral = P>>(
    mmc: Mmc<'_, P, Pins>,
    bufs: &mut Buffers,
) -> Result<(), Failure> {
    let addr = mmc.block_count().next_multiple_of(8);
    expect_err(
        "read past end",
        mmc.read_blocks(&mut bufs.read[..1], addr),
        Error::ADDR_OUTOF_RANGE,
    )?;
    expect_err(
        "write past end",
        mmc.write_blocks(&bufs.write[..1], addr),
        Error::ADDR_OUTOF_RANGE,
    )?;
    round_trip(mmc, bufs, SINGLE_ADDR, 1, 0x000A)
}
