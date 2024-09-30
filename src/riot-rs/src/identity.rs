//! Access to unique identifiers provided by the device.

#[doc(inline)]
pub use riot_rs_embassy_common::identity::DeviceId;

use crate::arch::identity::DeviceId as ArchDeviceId;

/// Obtains a unique identifier of the device.
///
/// See also [`DeviceId`].
pub fn device_identity(
) -> Result<<ArchDeviceId as DeviceId>::DeviceId, <ArchDeviceId as DeviceId>::Error> {
    riot_rs_embassy::arch::identity::DeviceId::get()
}
