use std::env;
use std::path::PathBuf;

fn main() {
    // Put the linker scripts somewhere the linker can find them
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    std::fs::copy("isr_stack.ld.in", out.join("isr_stack.x")).unwrap();
    std::fs::copy("linkme.x", out.join("linkme.x")).unwrap();

    if env::var_os("CARGO_FEATURE__ESP32C3").is_some() {
        std::fs::copy("linkme-esp32c3-fixup.x", out.join("linkme-esp-fixup.x")).unwrap();
    }

    if env::var_os("CARGO_FEATURE__ESP32C6").is_some() {
        std::fs::copy("linkme-esp32c6-fixup.x", out.join("linkme-esp-fixup.x")).unwrap();
    }

    println!("cargo:rustc-link-search={}", out.display());
}
