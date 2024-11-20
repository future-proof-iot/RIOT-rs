//! Access to unique identifiers provided by the device.
//!
//! This module provides [`device_id_bytes()`], which returns an identifier for the
//! concrete piece of hardware that the software is running on in byte serialized form.
//!
//! Concrete properties of a device identity are:
//!
//! * Identifiers are reasonably unique: They are either unique by construction (serial number, MAC
//!   address) or random identifiers (>= 64 bit).
//!
//! * The scope of the identifier is within an Ariel OS board. Their scope may be broader, eg. when
//!   a identifier is unique per MCU family, or even globally.
//!
//! * Identifiers do not change during regular development with a device, which includes the use of
//!   a programmer. Identifiers may change under deliberate conditions, eg. when a device has a
//!   one-time programmable identity, or when there is a custom functionality to overwrite the
//!   built-in identifier that is not triggered by the device erase that is performed as part of
//!   programming the device.
//!
//! Constructing an identifier fails rather than produce a dummy identifier.
//!
//! It is considered a breaking change in Ariel OS if a device's identifier changes or becomes an
//! error. Errors changing to valid identifiers is a compatible change.
#![no_std]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]

/// Obtains a unique identifier of the device in its byte serialized form.
///
/// See module level documentation for that identifier's properties.
///
/// # Errors
///
/// This function's errors are device dependent, and range from the function being infallibly typed
/// (where a device ID is accessible through memory mapped access) over bus errors (where a chip in
/// a faulty state might impede communication with an identity EEPROM) to erring unconditionally
/// (where no device ID is present or implemented).
pub fn device_id_bytes() -> Result<impl AsRef<[u8]>, impl core::error::Error> {
    use riot_rs_embassy_common::identity::DeviceId;

    riot_rs_embassy::hal::identity::DeviceId::get().map(|d| d.bytes())
}
