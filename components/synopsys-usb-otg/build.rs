fn main() {
    let profile = std::env::var("PROFILE").unwrap();
    let target = std::env::var("TARGET").unwrap();

    if profile == "debug" && !target.starts_with("x86_64") {
        println!("cargo:warning=synopsys-usb-otg is being compiled in debug mode. This driver works reliably only in release mode!");
    }
}
