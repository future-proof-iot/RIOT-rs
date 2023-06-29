use std::env;
use std::path::PathBuf;

use ld_memory::{Memory, MemorySection};

fn main() {
    // generate linker script
    let memory = Memory::new()
        .add_section(MemorySection::new("RAM", 0x20000000, 256 * 1024))
        .add_section(
            MemorySection::new("FLASH", 0x0, 1024 * 1024)
                .pagesize(4096)
                .from_env_with_prefix("NRF52840_FLASH"),
        );

    memory.to_cargo_outdir("memory.x").expect("wrote memory.x");

    println!("cargo:rerun-if-changed=build.rs");
}
