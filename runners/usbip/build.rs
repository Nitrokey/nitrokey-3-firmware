fn main() {
    println!(
        "cargo:rustc-env=USBIP_FIRMWARE_VERSION={}",
        utils::version_string()
    );
}
