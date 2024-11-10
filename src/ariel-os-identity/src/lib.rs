//! Access to unique identifiers provided by the device.
//!
//! This module provides [`device_id_bytes()`] and related functions, which returns an identifier for the
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
//!
//! Other identifiers, such as the EUI-48 addresses provided by [`interface_eui48()`], are usually
//! derived from the main identity, but have different properties.
#![no_std]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]
// required for tests:
#![cfg_attr(test, no_main)]
#![cfg_attr(test, feature(impl_trait_in_assoc_type))]
#![cfg_attr(test, feature(used_with_arg))]

pub use ariel_os_embassy_common::identity::Eui48;

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
    use ariel_os_embassy_common::identity::DeviceId;

    ariel_os_embassy::hal::identity::DeviceId::get().map(|d| d.bytes())
}

/// Generates an EUI-48 identifier ("6-byte MAC address") based on the device identity.
///
/// The argument `if_index` allows the system to generate addresses for consecutive interfaces.
///
/// The default implementation creates a static random identifier based on the board name, the
/// device ID bytes, incrementing by the interface index (following the common scheme of
/// sequential MAC addresses being assigned to multi-interface hardware). Those take the shape
/// `?2-??-??-??-??-??`: Their bits set to individual (I/G, unicast), administratively
/// locally administered (U/L, not indicating any particular manufacturer), and following the
/// SLAP (Structured Local Address Plan) semantics, they fall into the AII (Administratively
/// Assigned Identifier) quadrant. Wikipedia has a [good description of those address
/// details](https://en.wikipedia.org/wiki/MAC_address#Address_details).
///
/// The randomly generated identifiers aim to appear random, but can
/// be traced back to the device ID it is calculated from.
///
/// On devices that have access to globally unique EUI-48 identifiers, those are returned
/// for interface indices up to the number of available identifiers.
///
/// # Errors
///
/// Same as in [`device_id_bytes()`].
pub fn interface_eui48(if_index: u32) -> Result<Eui48, impl core::error::Error> {
    use ariel_os_embassy_common::identity::DeviceId;

    ariel_os_embassy::hal::identity::DeviceId::get().map(|d| d.interface_eui48(if_index))
}

#[cfg(test)]
#[embedded_test::tests]
mod tests {
    #[test]
    async fn has_device_id() {
        // TODO: make this confirm what we know about the device under test
        assert!(ariel_os::identity::device_id_bytes().is_ok());
    }
}
