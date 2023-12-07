use memory_regions::MemoryRegions;
use utils::VERSION;

fn main() {
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
        major = VERSION.major,
        minor = VERSION.minor,
        patch = VERSION.patch,
        version_to_check = VERSION.encode(),
        filesystem_start = MemoryRegions::LPC55.filesystem.start,
    );
}
