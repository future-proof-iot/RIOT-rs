#![cfg_attr(not(test), no_std)]
#![cfg_attr(test, no_main)]

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

    pub use rtt_target::{rprint as print, rprintln as println};

    pub fn init() {
        #[cfg(not(feature = "defmt"))]
        {
            use rtt_target::ChannelMode::NoBlockTrim;

            rtt_target::rtt_init_print!(NoBlockTrim);
        }

        #[cfg(feature = "defmt")]
        {
            use rtt_target::ChannelMode::{NoBlockSkip, NoBlockTrim};
            let channels = rtt_target::rtt_init! {
                up: {
                    0: {
                        size: 1024,
                        mode: NoBlockTrim,
                        // probe-run autodetects whether defmt is in use based on this channel name
                        name: "Terminal"
                    }
                    1: {
                        size: 1024,
                        mode: NoBlockSkip,
                        // probe-run autodetects whether defmt is in use based on this channel name
                        name: "defmt"
                    }
                }
            };

            rtt_target::set_print_channel(channels.up.0);
            defmt_rtt_target::init(channels.up.1);
        }
    }
}

#[cfg(all(feature = "debug-console", context = "esp"))]
mod backend {
    pub use esp_println::{print, println};
    pub const EXIT_SUCCESS: Result<(), ()> = Ok(());
    pub const EXIT_FAILURE: Result<(), ()> = Err(());
    pub fn exit(_code: Result<(), ()>) {
        loop {}
    }
    pub fn init() {
        // TODO: unify logging config.
        // Until then, `ESP_LOGLEVEL` can be used.
        // See https://github.com/esp-rs/esp-println#logging.
        esp_println::logger::init_logger_from_env();
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

#[cfg(feature = "defmt")]
pub mod log {
    pub use defmt;

    #[macro_export]
    macro_rules! __trace {
        ($($arg:tt)*) => {{
            use $crate::log::defmt;
            defmt::trace!($($arg)*);
        }};
    }

    #[macro_export]
    macro_rules! __debug {
        ($($arg:tt)*) => {{
            use $crate::log::defmt;
            defmt::debug!($($arg)*);
        }};
    }

    #[macro_export]
    macro_rules! __info {
        ($($arg:tt)*) => {{
            use $crate::log::defmt;
            defmt::info!($($arg)*);
        }};
    }

    #[macro_export]
    macro_rules! __warn {
        ($($arg:tt)*) => {{
            use $crate::log::defmt;
            defmt::warn!($($arg)*);
        }};
    }

    #[macro_export]
    macro_rules! __error {
        ($($arg:tt)*) => {{
            use $crate::log::defmt;
            defmt::error!($($arg)*);
        }};
    }

    pub use __debug as debug;
    pub use __error as error;
    pub use __info as info;
    pub use __trace as trace;
    pub use __warn as warn;
}

#[cfg(not(feature = "defmt"))]
pub mod log {
    #[macro_export]
    macro_rules! __stub {
        ($($arg:tt)*) => {{
            let _ = ($($arg)*); // Do nothing
        }};
    }

    pub use __stub as debug;
    pub use __stub as error;
    pub use __stub as info;
    pub use __stub as trace;
    pub use __stub as warn;
}
