//! Tools and traits for describing device identities.
//!
//! See `riot_rs::identity` for general documentation.

/// Describes how a board produces unique identifiers.
pub trait DeviceId {
    /// Opaque type representing a unique-ish identifier of a board.
    ///
    /// See [`.get()`](Self::get) for precise semantics.
    ///
    /// # Open questions
    ///
    /// Do we have any concrete serializers we want to prescribe? Or should we switch to a const
    /// size ([u8; N]) anyway?
    type DeviceId: core::fmt::Debug + defmt::Format;

    /// Error type indicating that no identifier is available.
    ///
    /// This is encouraged to be [`core::convert::Infallible`] where possible.
    ///
    /// # Open questions
    ///
    /// Some architectures will have to read this (eg. at QSPI initialization time); is there guidance
    /// on how to report "Not yet available"?
    type Error: core::error::Error + defmt::Format;

    /// Obtains a unique identifier of the device.
    ///
    /// A successful return value is, within the scope of the active board, reasonably unique. (The
    /// "reasonably" part is what allows this to be used with devices whose vendors do not assign
    /// serial numbers but large (>= 64 bits) random identifiers).
    ///
    /// This value may be obtained from the processor itself, or from a peripheral that is connected to
    /// the processor as part of the board.
    ///
    /// For the type implementing this trait at its conventional position
    /// `riot_rs::arch::identity::DeviceId`, a convenience function to call it exists at
    /// `riot_rs::identity::device_identity()`.
    ///
    /// # Open questions
    ///
    /// Uniqueness is currently described in the scope of the board, but this is implemented as part of
    /// the `arch`. Implementation experience will show what is more realistic.
    ///
    /// Ideally, those should be coordinated even outside of the project, so that a hypothetical
    /// extension to probe-rs that connects to devices by their ID can reuse the identifiers. This will
    /// require some more agreement on their structure as well as their scope. (probe-rs does not work
    /// on boards but just on chips).
    ///
    /// # Errors
    ///
    /// This prodcues an error if no device ID is available on this board, or is not implemented.
    fn get() -> Result<Self::DeviceId, Self::Error>;
}

/// A type implementing [`DeviceId`] that always errs.
///
/// This can be used both on architectures that do not have a unique identifier on their boards,
/// and when it has not yet been implemented.
pub struct NoDeviceId<E: core::error::Error + defmt::Format + Default>(
    core::marker::PhantomData<E>,
);

impl<E: core::error::Error + defmt::Format + Default> DeviceId for NoDeviceId<E> {
    type DeviceId = core::convert::Infallible;

    type Error = E;

    fn get() -> Result<Self::DeviceId, Self::Error> {
        Err(Default::default())
    }
}

/// Error indicating that a [`DeviceId`] may be available on this platform, but is not implemented.
#[derive(Debug, Default, defmt::Format)]
pub struct NotImplemented;

impl core::fmt::Display for NotImplemented {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Device ID not implemented on this platform")
    }
}

impl core::error::Error for NotImplemented {}

/// Error indicating that a [`DeviceId`] is not available on this platform.
#[derive(Debug, Default, defmt::Format)]
pub struct NotAvailable;

impl core::fmt::Display for NotAvailable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Device ID not available on this platform")
    }
}

impl core::error::Error for NotAvailable {}
