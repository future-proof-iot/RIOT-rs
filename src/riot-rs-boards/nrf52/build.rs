use ld_memory::{Memory, MemorySection};

enum Partition {
    FullDevice,
    Riotboot {
        bootloader: u64,
        slots: u64,
        slot_used: u64,
    },
}

impl Partition {
    fn from_env() -> Self {
        let slot_flags = [
            std::env::var_os("CARGO_FEATURE_RIOTBOOT_SLOT_0").is_some(),
            std::env::var_os("CARGO_FEATURE_RIOTBOOT_SLOT_1").is_some(),
        ];

        let slot = match slot_flags {
            [false, false] => None,
            [true, false] => Some(0),
            [false, true] => Some(1),
            _ => panic!("Conflicting riotboot-slot- flags."),
        };

        if std::env::var_os("CARGO_FEATURE_RIOTBOOT_PARTITIONING_16K_2SLOTS").is_some() {
            Partition::Riotboot {
                bootloader: 16 * 1024,
                slots: 2,
                slot_used: slot.expect("riotboot-partitioning set but no slot set"),
            }
        } else {
            if slot.is_some() {
                panic!("riotboot-slot selected but no riotboot-partitioning set");
            }
            // FIXME: We should make this explicit
            Partition::FullDevice
        }
    }

    /// Given the chip's total flash size, return flash start and end of the usable flash region in
    /// the given partition
    fn get_rom(&self, total_rom: u64) -> (u64, u64) {
        let hdr_len = 256; // see genhdr.c: needed for alignment purposes

        match self {
            Partition::FullDevice => (0, total_rom),
            Partition::Riotboot { bootloader, slots, slot_used } => {
                let after_bootloader = total_rom - bootloader;
                let per_slot = after_bootloader / slots;
                let start = bootloader + per_slot * slot_used + hdr_len;
                let end = bootloader + per_slot * (slot_used + 1);
                (start, end)
            }
        }
    }
}

fn main() {
    let (ram, rom) = if std::env::var_os("CARGO_FEATURE_NRF52832").is_some() {
        (64, 256)
    } else if std::env::var_os("CARGO_FEATURE_NRF52840").is_some() {
        (256, 1024)
    } else {
        panic!("nrf52: please set MCU feature");
    };

    let partition = Partition::from_env();
    let (rom_start, rom_end) = partition.get_rom(rom * 1024);

    // generate linker script
    let memory = Memory::new()
        .add_section(MemorySection::new("RAM", 0x20000000, ram * 1024))
        .add_section(
            MemorySection::new("FLASH", rom_start, rom_end)
                .pagesize(4096)
                .from_env_with_prefix("NRF52_FLASH"),
        );

    memory.to_cargo_outdir("memory.x").expect("wrote memory.x");

    println!("cargo:rerun-if-changed=build.rs");
}
