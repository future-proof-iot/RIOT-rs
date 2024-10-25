use std::{env, path::PathBuf};

fn main() {
    // TODO: These should be configurable. Like this, it works for MCUs with
    // a flash page size <= 4KiB.
    const FLASH_PAGE_SIZE: u32 = 0x1000;
    const STORAGE_SIZE_TOTAL: u32 = 0x2000;

    // need at least two flash pages
    // TODO: uncomment once this is not always true
    //assert!(STORAGE_SIZE_TOTAL / FLASH_PAGE_SIZE >= 2);

    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let mut storage_template = std::fs::read_to_string("storage.ld.in").unwrap();
    storage_template = storage_template.replace("${ALIGNMENT}", &format!("{FLASH_PAGE_SIZE}"));
    storage_template = storage_template.replace("${SIZE}", &format!("{STORAGE_SIZE_TOTAL}"));

    std::fs::write(out.join("storage.x"), &storage_template).unwrap();

    println!("cargo:rerun-if-changed=storage.ld.in");
    println!("cargo:rustc-link-search={}", out.display());
}
