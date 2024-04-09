pub trait Multicore {
    const CORES: u32;

    fn cpuid() -> u32;

    fn startup_cores();
}

cfg_if::cfg_if! {
    if #[cfg(context = "rp2040")] {
        mod rp2040;
        pub use rp2040::Chip;
    }
    else {
        pub struct Chip;
        impl Multicore for Chip {
            const CORES: u32 = 1;

            fn cpuid() -> u32 {
                0
            }

            fn startup_cores() { }
        }
    }
}
