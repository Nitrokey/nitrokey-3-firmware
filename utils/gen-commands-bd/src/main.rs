use std::{
    fs,
    path::{Path, PathBuf},
};

use cargo_metadata::{CargoOpt, MetadataCommand};
use gumdrop::Options;
use semver::Version;
use serde::Deserialize;

/// Generates a commands.bd file.
#[derive(Debug, Options)]
struct Args {
    /// Show this help message.
    help: bool,
    /// Only print out the firmware version
    only_version: bool,
    /// The path of the Cargo manifest to use.
    #[options(free)]
    manifest: PathBuf,
    /// The path of the build profile to use.
    #[options(free)]
    profile: PathBuf,
}

fn main() {
    let args = Args::parse_args_default_or_exit();
    let firmware_version = firmware_version(args.manifest);

    if args.only_version {
        println!("{firmware_version}");
        return;
    }

    let version_to_check = version_to_check(&firmware_version);
    let filesystem_boundary = filesystem_boundary(&args.profile);
    println!(
        "\
options {{
	flags = 0x8;
	buildNumber = 0x1;
	productVersion = \"{major}.{minor}.{patch}\";
	componentVersion = \"{major}.{minor}.{patch}\";
	secureBinaryVersion = \"2.1\";
}}

sources {{
	inputFile = extern(0);
}}

section (0) {{
	version_check sec {version_to_check};
	version_check nsec {version_to_check};
	erase 0x0..{filesystem_boundary:#x};
	load inputFile > 0x0;
}}",
        major = firmware_version.major,
        minor = firmware_version.minor,
        patch = firmware_version.patch,
    );
}

fn firmware_version(path: PathBuf) -> Version {
    let mut cmd = MetadataCommand::new();
    cmd.features(CargoOpt::AllFeatures);
    cmd.manifest_path(&path);
    let metadata = cmd.exec().expect("failed to parse manifest");
    metadata
        .root_package()
        .expect("missing root package")
        .version
        .clone()
}

fn version_to_check(version: &Version) -> u32 {
    let major = u32::try_from(version.major).expect("major version too high");
    let minor = u32::try_from(version.minor).expect("major version too high");
    let patch = u32::try_from(version.patch).expect("major version too high");
    if major >= 1024 || minor > 9999 || patch >= 64 {
        panic!("firmware version can at most be 1023.9999.63");
    }
    (major << 22) | (minor << 6) | patch
}

fn filesystem_boundary(path: &Path) -> u32 {
    let config = fs::read(path).expect("failed to read build profile");
    let config: Config = toml::from_slice(&config).expect("failed to parse build profile");
    config.parameters.filesystem_boundary
}

#[derive(Deserialize)]
struct Config {
    parameters: Parameters,
}

#[derive(Deserialize)]
struct Parameters {
    filesystem_boundary: u32,
}
