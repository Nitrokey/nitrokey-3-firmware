use std::process::Command;

fn main() {
	Command::new("./conv.py")
		.args(&["src/types.rs", "src/types_convs.rs"])
		.status().unwrap();
	Command::new("./conv.py")
		.args(&["src/se050.rs", "src/se050_convs.rs"])
		.status().unwrap();

	println!("cargo:rerun-if-changed=src/types.rs");
	println!("cargo:rerun-if-changed=src/se050.rs");
	println!("cargo:rerun-if-changed=build.rs");
}
