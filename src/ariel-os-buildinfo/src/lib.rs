//! Exposes information about the build.
#![no_std]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]

/// The board name.
///
/// The board name is read from the `CONFIG_BOARD` environment variable, which is expected to be
/// provided by the build system.
pub const BOARD: &str = ariel_os_utils::str_from_env_or!(
    "CONFIG_BOARD",
    "unknown",
    "board name provided by the build system"
);

/// The operating system's name.
pub const OS_NAME: &str = "Ariel OS";
