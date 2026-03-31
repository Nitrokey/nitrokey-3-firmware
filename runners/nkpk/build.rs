use std::{env, error};

use memory_regions::MemoryRegions;
use utils::Soc;

const MEMORY_REGIONS: &MemoryRegions = &MemoryRegions::NKPK;

fn main() -> Result<(), Box<dyn error::Error>> {
    println!(
        "cargo:rustc-env=NKPK_FIRMWARE_VERSION={}",
        utils::version_string("nitrokey-passkey-firmware", env!("CARGO_PKG_VERSION"))
    );

    utils::setup_linker_script(Soc::Nrf52, MEMORY_REGIONS);

    Ok(())
}
