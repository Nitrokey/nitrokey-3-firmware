fn main() {
    println!(
        "cargo:rustc-env=USBIP_FIRMWARE_VERSION={}",
        utils::version_string("nitrokey-3-firmware", env!("CARGO_PKG_VERSION"))
    );
}
