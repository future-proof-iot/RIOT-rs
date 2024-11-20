use ld_memory::{Memory, MemorySection};

// TODO: deduplicate with all "simple" cortex-m SoCs

fn main() {
    let (ram, rom) = (512, 1024);

    // generate linker script
    let memory = Memory::new()
        .add_section(MemorySection::new("RAM", 0x20000000, ram * 1024))
        .add_section(
            MemorySection::new("FLASH", 0x0, rom * 1024)
                .pagesize(4096)
                .from_env_with_prefix("NRF5340_FLASH"),
        );

    memory.to_cargo_outdir("memory.x").expect("wrote memory.x");

    println!("cargo:rerun-if-changed=build.rs");
}
