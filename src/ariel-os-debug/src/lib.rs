//! Provides debug interface facilities.

#![cfg_attr(not(test), no_std)]
#![cfg_attr(test, no_main)]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]

#[cfg(all(feature = "rtt-target", feature = "esp-println"))]
compile_error!(
    r#"feature "rtt-target" and feature "esp-println" cannot be enabled at the same time"#
);

#[cfg(all(
    feature = "debug-console",
    not(any(feature = "rtt-target", feature = "esp-println"))
))]
compile_error!(
    r#"feature "debug-console" enabled but no backend. Select feature "rtt-target" or feature "esp-println"."#
);

/// Represents the exit code of a debug output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExitCode {
    #[doc(hidden)]
    Success,
    #[doc(hidden)]
    Failure,
}

impl ExitCode {
    /// The [`ExitCode`] for success.
    pub const SUCCESS: Self = Self::Success;
    /// The [`ExitCode`] for failure.
    pub const FAILURE: Self = Self::Failure;

    #[allow(dead_code, reason = "not always used due to conditional compilation")]
    fn to_semihosting_code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::Failure => 1,
        }
    }
}

/// Terminates the debug output session.
pub fn exit(code: ExitCode) {
    loop {
        #[cfg(feature = "semihosting")]
        semihosting::process::exit(code.to_semihosting_code());

        #[cfg(not(feature = "semihosting"))]
        {
            let _ = code;
            core::hint::spin_loop();
        }
    }
}

#[cfg(all(feature = "debug-console", feature = "rtt-target"))]
mod backend {
    pub use rtt_target::{rprint as print, rprintln as println};

    #[doc(hidden)]
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
            rtt_target::set_defmt_channel(channels.up.1);
        }
    }
}

#[cfg(all(feature = "debug-console", feature = "esp-println"))]
mod backend {
    pub use esp_println::{print, println};

    #[doc(hidden)]
    pub fn init() {
        // TODO: unify logging config.
        // Until then, `ESP_LOGLEVEL` can be used.
        // See https://github.com/esp-rs/esp-println#logging.
        esp_println::logger::init_logger_from_env();
    }
}

#[cfg(not(feature = "debug-console"))]
mod backend {
    #[doc(hidden)]
    pub fn init() {}

    /// Prints to the debug output, with a newline.
    #[macro_export]
    macro_rules! println {
        ($($arg:tt)*) => {{
            let _ = ($($arg)*);
            // Do nothing
        }};
    }

    /// Prints to the debug output.
    ///
    /// Equivalent to the [`println!`] macro except that a newline is not printed at the end of the message.
    #[macro_export]
    macro_rules! print {
        ($($arg:tt)*) => {{
            let _ = ($($arg)*);
            // Do nothing
        }};
    }
}

pub use backend::*;

#[cfg(feature = "defmt")]
pub mod log {
    //! Provides debug logging, powered by [`defmt`].

    pub use defmt::{debug, error, info, trace, warn, Debug2Format, Display2Format};
    // We would rather not re-export the whole crate, but the macros do rely on it being public.
    pub use defmt;
}

#[cfg(not(feature = "defmt"))]
pub mod log {
    //! Stub module for when the `defmt` Cargo feature is not enabled.

    #[doc(hidden)]
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
