//! Access to unique identifiers provided by the device.
//!
//! The main way to use this module is [`device_identity()`], which returns an identifier for the
//! concrete piece of hardware that the software is running on.
//!
//! Concrete properties of a device identity are:
//!
//! * Identifiers are reasonably unique: They are either unique by construction (serial number, MAC
//!   address) or random identifiers (>= 64 bit).
//!
//! * The scope of the identifier is within a RIOT-rs board. Their scope may be broader, eg. when
//!   a identifier is unique per MCU family, or even globally.
//!
//! * Identifiers do not change during regular development with a board, which includes the use of
//!   programmer. Identifiers may change under deliberate conditions, eg. when a device has a
//!   one-time programmable identity, or when there is a custom functionality to overwrite the
//!   built-in identifier that is not triggered by the device erase that is performed as part of
//!   programming the device.
//!
//! Constructing an identifier fails rather than producing a dummy identifier.
//!
//! It is considered a breaking change in a board or this module if a board's identifier changes or
//! becomes an error as result of an update to RIOT-rs; errors may change to valid identifiers.

#[doc(inline)]
pub use riot_rs_embassy_common::identity::DeviceId;

use crate::arch::identity::DeviceId as ArchDeviceId;

/// Obtains a unique identifier of the device.
pub fn device_identity() -> Result<ArchDeviceId, <ArchDeviceId as DeviceId>::Error> {
    riot_rs_embassy::arch::identity::DeviceId::get()
}
