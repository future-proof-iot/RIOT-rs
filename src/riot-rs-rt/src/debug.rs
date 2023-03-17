#[cfg(all(feature = "rtt-target", feature = "cortex-m-semihosting"))]
compile_error!("feature \"rtt-target\" and feature \"cortex-m-semihosting\" cannot be enabled at the same time");

#[cfg(all(feature = "debug-console", feature = "cortex-m-semihosting"))]
mod backend {
    pub use cortex_m_semihosting::debug::{exit, EXIT_FAILURE, EXIT_SUCCESS};
    pub use cortex_m_semihosting::hprint as print;
    pub use cortex_m_semihosting::hprintln as println;
}

#[cfg(all(feature = "debug-console", feature = "rtt-target"))]
mod backend {
    pub const EXIT_SUCCESS: u32 = 0;
    pub const EXIT_FAILURE: u32 = 1;
    pub fn exit(_code: u32) {
        loop {
            cortex_m::asm::bkpt();
        }
    }
    pub use rtt_target::rprint as print;
    pub use rtt_target::rprintln as println;
}

#[cfg(not(feature = "debug-console"))]
mod backend {
    pub const EXIT_SUCCESS: u32 = 0;
    pub const EXIT_FAILURE: u32 = 1;
    pub fn exit(_code: u32) {
        loop {}
    }

    #[macro_export]
    macro_rules! nop_println {
        ($($arg:tt)*) => {{
            // Do nothing
        }};
    }

    #[macro_export]
    macro_rules! nop_print {
        ($($arg:tt)*) => {{
            // Do nothing
        }};
    }

    pub use nop_print as print;
    pub use nop_println as println;
}

pub use backend::*;
