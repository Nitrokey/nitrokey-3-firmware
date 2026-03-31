use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use cargo_lock::Lockfile;
use chrono::{Datelike as _, Local};
use memory_regions::MemoryRegions;
use regex::Regex;
use semver::{BuildMetadata, Prerelease, Version};

// We have a crate version, as defined in the Cargo.toml, and a firmware version, as reported by
// the device e. g. in the admin app.  The firmware version is determined using these steps:
//
// 1. The crate version is either <major>.<minor>.<patch> or <major>.<minor>.<patch>-rc.<n>.
// 2. If the test feature is activated, the prerelease component for the firmware version is set to
//    test.<yyyy><mm><dd>.
// 3. If built from a tag in the CI, the tag name must be a semver version with the v prefix.  The
//    major, minor and patch components must match the crate version, the prerelease component must
//    match the crate version or the prerelease component computed in step (2), and the build
//    metadata must be empty.
// 4. Unless the firmware is built in the release CI, the build metadata for the firmware version
//    is set to git-<commit> or git-<commit>-dirty.

const PATTERN_PRE: &str = r"rc\.\d+";

pub fn version_string(project: &str, cargo_pkg_version: &str) -> String {
    let mut version = crate_version(cargo_pkg_version);
    let test_prerelease = test_prerelease();

    if let Some(tag_version) = tag_version(project) {
        assert_eq!(
            tag_version.major, version.major,
            "bad major component in tag"
        );
        assert_eq!(
            tag_version.minor, version.minor,
            "bad minor component in tag"
        );
        assert_eq!(
            tag_version.patch, version.patch,
            "bad patch component in tag"
        );
        assert!(
            tag_version.pre == version.pre || tag_version.pre == test_prerelease,
            "prerelease component {} in tag must be one of: {}, {}",
            tag_version.pre,
            version.pre,
            test_prerelease,
        );
        assert_eq!(
            tag_version.build,
            BuildMetadata::EMPTY,
            "build metadata in tag must be empty"
        );
    }

    if cfg!(feature = "test") {
        // We intentionally overwrite the rc.<n> prerelease component if it is set
        version.pre = test_prerelease;
    }
    if !is_release() {
        if let Some(build_metadata) = build_metadata() {
            version.build = build_metadata;
        }
    }

    version.to_string()
}

fn crate_version(cargo_pkg_version: &str) -> Version {
    let version = Version::parse(cargo_pkg_version).expect("failed to parse crate version");
    assert!(
        version.build.is_empty(),
        "crate version may not have build metadata: {version}"
    );
    if !version.pre.is_empty() {
        let r = Regex::new(PATTERN_PRE).unwrap();
        assert!(
            r.is_match(version.pre.as_str()),
            "unexpected pre version: {}",
            version.pre
        );
    }
    version
}

fn tag_version(project: &str) -> Option<Version> {
    if option_env!("CI_PROJECT_NAME")? != project {
        return None;
    }
    option_env!("CI_COMMIT_TAG")
        .map(|s| s.strip_prefix('v').expect("tag must start with v"))
        .map(|s| Version::parse(s).expect("failed to parse version from tag"))
}

fn test_prerelease() -> Prerelease {
    let now = Local::now();
    let pre = format!("test.{:04}{:02}{:02}", now.year(), now.month(), now.day());
    pre.parse().unwrap()
}

fn build_metadata() -> Option<BuildMetadata> {
    // We want to get the latest commit and whether the working tree is dirty.  Apparently, there
    // is no easy machine-readble way to do the latter so we use git-describe with --exclude * so
    // that we don’t get tag names, only commit shas.
    Command::new("git")
        .args(["describe", "--always", "--dirty", "--exclude", "*"])
        .output()
        .ok()
        .map(|output| String::from_utf8(output.stdout).expect("invalid output from git describe"))
        .map(|s| format!("git.{}", s.trim().replace('-', ".")))
        .map(|s| s.parse().unwrap())
}

fn is_release() -> bool {
    option_env!("CI_PIPELINE_SOURCE") == Some("web")
}

#[derive(Eq, PartialEq)]
pub enum Soc {
    Lpc55,
    Nrf52,
}

pub fn setup_linker_script(soc: Soc, regions: &MemoryRegions) {
    assert!(regions.filesystem.start.is_multiple_of(1024));

    let soc = match soc {
        Soc::Lpc55 => "lpc55",
        Soc::Nrf52 => "nrf52",
    };

    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let linker_script_dir = root.join("../ld");
    let template = linker_script_dir.join(format!("{soc}-memory-template.x"));

    let memory_x_dir = root.join("ld").join(soc);
    std::fs::create_dir_all(&memory_x_dir).ok();
    let memory_x = memory_x_dir.join("custom_memory.x");

    generate_memory_x(&memory_x, &template, regions);

    let lockfile = Lockfile::load(root.join("../../Cargo.lock")).expect("failed to parse lockfile");
    let cortex_m_rt = lockfile
        .packages
        .iter()
        .find(|p| p.name.as_str() == "cortex-m-rt")
        .expect("missing cortex-m-rt dependency");
    let linker_script = format!("cortex-m-rt_{}_link.x", cortex_m_rt.version);

    println!("cargo:rerun-if-changed={}", template.display());
    println!(
        "cargo:rerun-if-changed={}",
        linker_script_dir.join(&linker_script).display()
    );
    println!("cargo:rustc-link-search={}", memory_x_dir.display());
    println!("cargo:rustc-link-search={}", root.join("../ld").display());
    println!("cargo:rustc-link-arg=-T{linker_script}");
}

fn generate_memory_x(outpath: &Path, template: &Path, regions: &MemoryRegions) {
    let buildrs_caveat = r#"/* DO NOT EDIT THIS FILE */
/* This file was generated by build.rs */
"#;

    let template = std::fs::read_to_string(template).expect("cannot read memory.x template file");

    let fw_len = regions.firmware.len();
    let template = template.replace("##FLASH_LENGTH##", &format!("{fw_len:#X}"));

    let fs_len = regions.filesystem.len();
    let template = template.replace("##FS_LENGTH##", &format!("{fs_len:#X}"));

    let template = template.replace("##FS_BASE##", &format!("{:#X}", regions.filesystem.start));
    let template = template.replace("##FLASH_BASE##", &format!("{:#X}", regions.firmware.start));

    std::fs::write(outpath, [buildrs_caveat, &template].join("")).expect("cannot write memory.x");
}
