use std::env;
use std::path::PathBuf;

fn main() {
    // Put the linker scripts somewhere the linker can find them
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    std::fs::copy("isr_stack.ld.in", out.join("isr_stack.x")).unwrap();
    std::fs::copy("linkme.x", out.join("linkme.x")).unwrap();

    println!("cargo:rustc-link-search={}", out.display());
}
