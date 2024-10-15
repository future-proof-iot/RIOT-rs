use std::env;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // Put the linker script somewhere the linker can find them
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let mut storage_template = std::fs::read_to_string("storage.ld.in").unwrap();
    storage_template = storage_template.replace("${ALIGNMENT}", "0x1000");
    storage_template = storage_template.replace("${SIZE}", "0x2000");

    std::fs::File::create(out.join("storage.x"))
        .unwrap()
        .write_all(storage_template.as_bytes())
        .unwrap();

    println!("cargo:rustc-link-search={}", out.display());
}
