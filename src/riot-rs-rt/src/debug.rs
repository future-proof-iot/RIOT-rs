#[cfg(all(feature = "rtt-target", feature = "cortex-m-semihosting"))]
compile_error!("feature \"rtt-target\" and feature \"cortex-m-semihosting\" cannot be enabled at the same time");

#[cfg(all(feature = "debug-console", feature = "cortex-m-semihosting"))]
mod backend {
    pub use cortex_m_semihosting::debug::{exit, EXIT_FAILURE, EXIT_SUCCESS};
    pub use cortex_m_semihosting::hprint as print;
    pub use cortex_m_semihosting::hprintln as println;
    pub fn init() {}
}

#[cfg(all(feature = "debug-console", feature = "rtt-target"))]
mod backend {
    const SYS_EXIT: u32 = 0x18;
    pub const EXIT_SUCCESS: Result<(), ()> = Ok(());
    pub const EXIT_FAILURE: Result<(), ()> = Err(());
    pub fn exit(code: Result<(), ()>) {
        let semihosting_exit_code = match code {
            EXIT_FAILURE => 1,
            EXIT_SUCCESS => 0x20026,
        };

        loop {
            unsafe { cortex_m::asm::semihosting_syscall(SYS_EXIT, semihosting_exit_code) };
        }
    }
    pub use rtt_target::rprint as print;
    pub use rtt_target::rprintln as println;
    pub fn init() {
        rtt_target::rtt_init_print!();
    }
}

#[cfg(not(feature = "debug-console"))]
mod backend {
    pub const EXIT_SUCCESS: Result<(), ()> = Ok(());
    pub const EXIT_FAILURE: Result<(), ()> = Err(());
    pub fn exit(_code: Result<(), ()>) {
        #[allow(clippy::empty_loop)]
        loop {}
    }
    pub fn init() {}

    #[macro_export]
    macro_rules! nop_println {
        ($($arg:tt)*) => {{
            let _ = ($($arg)*);
            // Do nothing
        }};
    }

    #[macro_export]
    macro_rules! nop_print {
        ($($arg:tt)*) => {{
            let _ = ($($arg)*);
            // Do nothing
        }};
    }

    pub use nop_print as print;
    pub use nop_println as println;
}

pub use backend::*;
