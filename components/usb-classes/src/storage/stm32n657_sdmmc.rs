use stm32n657_hal::{
    mmc::{MmcMaster, MmcPins},
    sdmmc::{Enabled, Error, SdMmc},
};

use super::BlockDevice;

pub struct MmcStorage<P, Pins> {
    mmc: MmcMaster<P, Pins, Enabled>,
}

impl<P: SdMmc, Pins: MmcPins<Peripheral = P>> MmcStorage<P, Pins> {
    pub fn new(mmc: MmcMaster<P, Pins, Enabled>) -> Self {
        Self { mmc }
    }
}

impl<P: SdMmc, Pins: MmcPins<Peripheral = P>> BlockDevice for MmcStorage<P, Pins> {
    type Error = Error;
    fn blocks(&self) -> u32 {
        self.mmc.block_count()
    }

    fn read_block(
        &mut self,
        lba: u32,
        buf: &mut [u8; super::BLOCK_SIZE],
    ) -> Result<(), Self::Error> {
        self.mmc.read_blocks(core::slice::from_mut(buf), lba)
    }

    fn write_block(
        &mut self,
        lba: u32,
        buf: &mut [u8; super::BLOCK_SIZE],
    ) -> Result<(), Self::Error> {
        self.mmc.write_blocks(core::slice::from_ref(buf), lba)
    }
}
