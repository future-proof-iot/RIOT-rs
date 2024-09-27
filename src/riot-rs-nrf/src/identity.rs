/// Opaque type representing a unique-ish identifier of a board.
///
/// This should be primitive enough that it implements Debug, defmt::Format and common
/// serializers/deserializers.
///
/// See [`device_id()`] for precise semantics.
pub type DeviceId = u64;

/// Error type indicating that no identifier is available.
///
/// This should implement [core::error::Error], and is encouraged to be
/// [`core::convert::Infallible`] where possible.
///
/// ## Open questions
///
/// Some architectures will have to read this (eg. at QSPI initialization time); is there guidance
/// on how to report "Not yet available"?
pub type DeviceIdError = core::convert::Infallible;

/// Obtains a unique identifier of the device.
///
/// A successful return value is, within the scope of the active board, reasonably unique. (The
/// "reasonably" part is what allows this to be used with devices whose vendors do not assign
/// serial numbers but large (>= 64 bits) random identifiers).
///
/// This value may be obtained from the processor itself, or from a peripheral that is connected to
/// the processor as part of the board.
///
/// ## Open questions
///
/// Uniqueness is currently described in the scope of the board, but this is implemented as part of
/// the `arch`. Implementation experience will show what is more realistic.
///
/// Ideally, those should be coordinated even outside of the project, so that a hypothetical
/// extension to probe-rs that connects to devices by their ID can reuse the identifiers. This will
/// require some more agreement on their structure as well as their scope. (probe-rs does not work
/// on boards but just on chips).
pub fn device_id() -> Result<DeviceId, DeviceIdError> {
    // Embassy does not wrap the FICR register, and given that all we need from there is a register
    // read that is perfectly fine to do through a stolen register, let's do that rather than
    // thread the access through several layers.

    // SAFETY: The register is used for read-only operations on constant values.
    #[cfg(context = "nrf52840")]
    let ficr = unsafe { nrf52840_pac::Peripherals::steal().FICR };
    #[cfg(context = "nrf52832")]
    let ficr = unsafe { nrf52832_pac::Peripherals::steal().FICR };
    #[cfg(context = "nrf5340")]
    let ficr = unsafe { nrf5340_app_pac::Peripherals::steal().FICR_S }.info;

    let low = ficr.deviceid[0].read().bits();
    let high = ficr.deviceid[1].read().bits();
    Ok((u64::from(high) << u32::BITS) | u64::from(low))
}
