use std::{env, error};

use memory_regions::MemoryRegions;
use utils::Soc;

#[cfg(feature = "board-nk3xn")]
const MEMORY_REGIONS: &MemoryRegions = &MemoryRegions::NK3XN;
#[cfg(feature = "board-nk3am")]
const MEMORY_REGIONS: &MemoryRegions = &MemoryRegions::NK3AM;

fn check_build_triplet() -> Soc {
    let target = env::var("TARGET").expect("$TARGET unset");
    let soc_is_lpc55 = env::var_os("CARGO_FEATURE_SOC_LPC55").is_some();
    let soc_is_nrf52840 = env::var_os("CARGO_FEATURE_SOC_NRF52").is_some();

    if soc_is_lpc55 && !soc_is_nrf52840 {
        if target != "thumbv8m.main-none-eabi" {
            panic!(
                "Wrong build triplet for LPC55, expecting thumbv8m.main-none-eabi, got {target}"
            );
        }
        Soc::Lpc55
    } else if soc_is_nrf52840 && !soc_is_lpc55 {
        if target != "thumbv7em-none-eabihf" {
            panic!(
                "Wrong build triplet for NRF52840, expecting thumbv7em-none-eabihf, got {target}",
            );
        }
        Soc::Nrf52
    } else {
        panic!("Multiple or no SOC features set.");
    }
}

fn main() -> Result<(), Box<dyn error::Error>> {
    println!(
        "cargo:rustc-env=NK3_FIRMWARE_VERSION={}",
        utils::version_string("nitrokey-3-firmware", env!("CARGO_PKG_VERSION"))
    );

    let soc = check_build_triplet();
    utils::setup_linker_script(soc, MEMORY_REGIONS);

    Ok(())
}
