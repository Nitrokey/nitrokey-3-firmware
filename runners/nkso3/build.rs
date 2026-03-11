fn main() {
    println!("cargo:rustc-link-search={}", env!("CARGO_MANIFEST_DIR"));
}
