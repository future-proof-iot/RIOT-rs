//! Access to unique identifiers provided by the device.

use crate::arch::identity::DeviceId as ArchDeviceId;
use riot_rs_embassy_common::identity::DeviceId as CommonDeviceId;

/// Obtains a unique identifier of the device.
///
/// See also [`riot_rs_embassy_common::identity`].
pub fn device_identity(
) -> Result<<ArchDeviceId as CommonDeviceId>::DeviceId, <ArchDeviceId as CommonDeviceId>::Error> {
    riot_rs_embassy::arch::identity::DeviceId::get()
}
