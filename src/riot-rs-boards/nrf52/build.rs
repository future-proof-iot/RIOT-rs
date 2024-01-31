use ld_memory::{Memory, MemorySection};

fn main() {
    let (ram, rom) = if std::env::var_os("CARGO_FEATURE_NRF52832").is_some() {
        (64, 256)
    } else if std::env::var_os("CARGO_FEATURE_NRF52840").is_some() {
        (256, 1024)
    } else {
        panic!("nrf52: please set MCU feature");
    };

    // generate linker script
    let memory = Memory::new()
        .add_section(MemorySection::new("RAM", 0x20000000, ram * 1024))
        .add_section(
            MemorySection::new("FLASH", 0x0, rom * 1024)
                .pagesize(4096)
                .from_env_with_prefix("NRF52_FLASH"),
        );

    memory.to_cargo_outdir("memory.x").expect("wrote memory.x");

    println!("cargo:rerun-if-changed=build.rs");
}
