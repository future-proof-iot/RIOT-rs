//! Tools and traits for describing device identities.
//!
//! See `riot_rs::identity` for general documentation.
#![deny(missing_docs)]

/// Trait desribing the unique identifier available on a board.
///
/// See the module level documentation on the characteristics of the identifier.
///
/// # Evolution
///
/// In its current state, this type is mainly a wrapper around a binary identifier with a
/// length constant at build time.
///
/// As it is used more, additional methods can be provided for concrete types of identifiers, such
/// as MAC addresses. By default, those would be generated in some way from what is available in
/// the identifier -- but boards where the identifier already *is* a MAC address (or possibly a
/// range thereof) can provide their official addresses.
pub trait DeviceId: Sized + core::fmt::Debug + defmt::Format {
    /// Error type indicating that no identifier is available.
    ///
    /// This is part of the return type of the [`::get()`][Self::get] constructor.
    ///
    /// It is encouraged to be [`core::convert::Infallible`] where possible.
    ///
    /// # Open questions
    ///
    /// Some architectures will have to read this (eg. at QSPI initialization time); is there guidance
    /// on how to report "Not yet available"?
    type Error: core::error::Error + defmt::Format;

    /// Some `[u8; N]` type, returned by [`.bytes()`][Self::bytes].
    ///
    /// This may not represent all the identifying information available on the board, but can
    /// represent a unique portion thereof.
    ///
    /// (For example, if a device has two consecutive MAC addresses assigned, the type as a whole
    /// may represent both, but the conventional serialized identity of the board may just be one
    /// of them).
    ///
    /// # Evolution
    ///
    /// On the long run, it will be preferable to add a `const BYTES_LEN: usize;` and enforce the
    /// type `[u8; Self::BYTES_LEN]` as the return value of [`.bytes(_)]`][Self::bytes]. This can
    /// not be done yet as it depends on the `generic_const_exprs` featureVg
    type Bytes: AsRef<[u8]>;

    /// Obtains a unique identifier of the device.
    ///
    /// For callers, there is the convenience function `riot_rs::identity::device_identity()`
    /// available, which just calls this trait method on `riot_rs::arch::identity::DeviceId`.
    ///
    /// # Errors
    ///
    /// This produces an error if no device ID is available on this board, or is not implemented.
    fn get() -> Result<Self, Self::Error>;

    /// The device identifier in serialized bytes format.
    fn bytes(&self) -> Self::Bytes;
}

/// An uninhabited type implementing [`DeviceId`] that always errs.
///
/// This can be used both on architectures that do not have a unique identifier on their boards,
/// and when it has not yet been implemented.
///
/// Typical types for `E` are [`NotImplemented`] or [`NotAvailable`].
#[derive(Debug, defmt::Format)]
pub struct NoDeviceId<E: core::error::Error + defmt::Format + Default>(
    core::convert::Infallible,
    core::marker::PhantomData<E>,
);

impl<E: core::error::Error + defmt::Format + Default> DeviceId for NoDeviceId<E> {
    type Error = E;

    // We could also come up with a custom never type that AsRef's into [u8], but that won't fly
    // once there is a BYTES_LEN.
    type Bytes = [u8; 0];

    fn get() -> Result<Self, Self::Error> {
        Err(Default::default())
    }

    fn bytes(&self) -> [u8; 0] {
        match self.0 {}
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
