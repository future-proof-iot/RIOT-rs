//! Exposes information about the build.

/// The board name.
///
/// The board name is read from the `CONFIG_BOARD` environment variable, which is expected to be
/// provided by the build system.
pub const BOARD: &str = riot_rs_utils::str_from_env_or!(
    "CONFIG_BOARD",
    "unknown",
    "board name provided by the build system"
);
