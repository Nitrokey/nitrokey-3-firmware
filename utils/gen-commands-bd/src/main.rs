use cargo_metadata::MetadataCommand;
use memory_regions::MemoryRegions;
use std::env;
use utils::Version;

fn main() {
    let mut args = env::args();
    args.next();
    let manifest_path = args.next().expect("missing argument: manifest file");
    let metadata = MetadataCommand::new()
        .manifest_path(manifest_path)
        .exec()
        .expect("failed to parse package metadata");
    let package = metadata
        .root_package()
        .expect("missing root package in manifest");

    let version = Version::new(
        package
            .version
            .major
            .try_into()
            .expect("major version too high"),
        package
            .version
            .minor
            .try_into()
            .expect("minor version too high"),
        package
            .version
            .patch
            .try_into()
            .expect("patch version too high"),
    );
    let version_to_check = version.encode();
    let version_string = package.version.to_string();

    match args.next().as_deref() {
        None => {}
        Some("-O") => {
            println!("{version_to_check}");
            return;
        }
        Some("-o") => {
            println!("{version_string}");
            return;
        }
        Some(s) => {
            panic!("Cannot parse arguments: {s}");
        }
    }

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
	erase 0x0..{filesystem_start:#x};
	load inputFile > 0x0;
}}",
        major = version.major(),
        minor = version.minor(),
        patch = version.patch(),
        filesystem_start = MemoryRegions::NK3XN.filesystem.start,
    );
}
