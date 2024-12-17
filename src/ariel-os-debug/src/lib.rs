#![cfg_attr(not(test), no_std)]
#![cfg_attr(test, no_main)]

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

#[doc(inline)]
pub use ariel_os_log as log;

pub const EXIT_SUCCESS: Result<(), ()> = Ok(());
pub const EXIT_FAILURE: Result<(), ()> = Err(());
pub fn exit(code: Result<(), ()>) {
    let code = match code {
        EXIT_FAILURE => 1,
        EXIT_SUCCESS => 0,
    };

    loop {
        #[cfg(feature = "semihosting")]
        semihosting::process::exit(code);
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
    pub fn init() {
        // TODO: unify logging config.
        // Until then, `ESP_LOGLEVEL` can be used.
        // See https://github.com/esp-rs/esp-println#logging.
        esp_println::logger::init_logger_from_env();
    }
}

#[cfg(not(feature = "debug-console"))]
mod backend {
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
