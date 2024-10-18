//! Provides a sensor driver instance registry, allowing to register sensor driver instances and
//! access them in a centralized location.

use crate::Sensor;

/// Stores references to registered sensor driver instances.
///
/// To register a sensor driver instance, insert a `&'static` into this [distributed
/// slice](linkme).
/// The sensor driver will therefore need to be statically allocated, to be able to obtain a
/// `&'static`.
// Exclude this from the users' documentation, to force users to use `Registry::sensors()` instead,
// for easier forward compatibility with possibly non-static references.
#[doc(hidden)]
#[linkme::distributed_slice]
pub static SENSOR_REFS: [&'static dyn Sensor] = [..];

/// The global registry instance.
pub static REGISTRY: Registry = Registry::new();

/// The sensor driver instance registry.
///
/// This is exposed as [`REGISTRY`].
pub struct Registry {
    // Prevents instantiation from outside this module.
    _private: (),
}

impl Registry {
    // The constructor is private to make the registry a singleton.
    const fn new() -> Self {
        Self { _private: () }
    }

    /// Returns an iterator over registered sensor driver instances.
    pub fn sensors(&self) -> impl Iterator<Item = &'static dyn Sensor> {
        // Returning an iterator instead of the distributed slice directly would allow us to chain
        // another source of sensor driver instances in the future, if we decided to support
        // dynamically-allocated sensor driver instances.
        SENSOR_REFS.iter().copied()
    }
}
