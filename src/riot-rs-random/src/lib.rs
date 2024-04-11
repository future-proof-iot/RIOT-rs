//! Provides a seeded random number generator depending on RIOT-rs's configuration.
//!
//! The module provides a single function for use by applications, [`get_rng()`], produces an owned
//! RNG of type [`Rng`] to the application.
//!
//! The crate abstracts over multiple aspects of RNGs:
//! * Where do we take a valid seed for the RNG from?
//! * What's the type of RNG that we take along?
//! * Can some of those be shared?
//!   (This may need an API change, but non-Send tasks could very well use an executor-local RNG
//!   behind a shared reference to not all carry a full RNG state for each user)
//!
//! No matter the choices taken (eventually through the application's setup), all is hidden behind
//! a main [`Rng`] type, which depending on the feature `main-is-csprng` also implements
//! [`rand_core::CryptoRng`]
//!
//! Before accessing the RNG, it needs to be initialized through the [`construct_rng()`'] function.
//! This is taken care of by the `riot-rs-embassy` initialization functions. Applications can
//! ensure that this has happened by depending on the laze feature `rng`.
//!
//! ---
//!
//! Currently, this provides very little choice, and little fanciness: It (more or less
//! arbitrarily) uses the [rand_chacha::ChaCha20Rng] generator, seeds uses different copies of that
//! RNG for all callers fo [`get_rng()`]. Later, it may offer a wider range of choices and
//! trade-offs between flash size, speed and security. (See [Rng] for its properties).
#![no_std]

use rand_core::{RngCore, SeedableRng};

// The Mutex<RefCell> can probably be simplified
static RNG: embassy_sync::blocking_mutex::Mutex<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    core::cell::RefCell<Option<Rng>>,
> = embassy_sync::blocking_mutex::Mutex::new(core::cell::RefCell::new(None));

// This is one of the points where we can tune easily. If calls to `get_rng()` are rare, it may
// even make sense to move the HWRNG in here to get a global ZST.
// #[cfg(feature = "main-is-csprng")]
pub(crate) type SelectedRng = rand_chacha::ChaCha20Rng;

/// The OS provided random number generator.
///
/// This may be small, fast and/or secure, but will sacrifice the earlier properties if the later
/// are needed by any system component.
///
/// Such an RNG can be requested by any component, and will always be seeded appropriately.
///
/// There is no point in requesting a "small" RNG (if a fast RNG is built in, that will do just as
/// well). There may be a point in requesting a "fast" RNG, which it may make sense to introduce a
/// FastRng that particular components can request. (Until that is available, an application can
/// depend on a known fast RNG and seed it from the OS's RNG, but preferably there should be a
/// single configured implementation used across applications for ROM optimization, even if it
/// turns out to be beneficial to have a thread or task local RNG around in RAM for speed).
pub struct Rng {
    inner: SelectedRng,
}

impl Rng {
    // Create a per-thread/per-task copy of the RNG. Not sure whether we'll need that all the time,
    // and/or whether these should all have the same types, but for the first iteration
    // everything-is-of-type-Rng is good enough
    fn split(&mut self) -> Self {
        Self { inner: SelectedRng::from_rng(self).expect("Main RNG is infallible") }
    }
}

// Re-implementing the trait rather than Deref'ing into inner: This avoids leaking implementation
// details to users who might otherwise come to depend on platform specifics of the Rng.
impl RngCore for Rng {
    fn next_u32(&mut self) -> u32 {
        self.inner.next_u32()
    }
    fn next_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }
    fn fill_bytes(&mut self, buf: &mut [u8]) {
        self.inner.fill_bytes(buf)
    }
    fn try_fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), rand_core::Error> {
        self.inner.try_fill_bytes(buf)
    }
}

#[cfg(feature = "main-is-csprng")]
mod main_is_csprng {
    impl rand_core::CryptoRng for super::Rng {}

    /// Asserts that SelectedRng is CryptoRng, justifying the implementation above.
    fn static_assert_is_cryptorng() -> impl rand_core::CryptoRng {
        let result: super::SelectedRng = unreachable!("This function is for type checking only");
        result
    }
}

/// Populates the RNG from a seed value.
///
/// This is called by RIOT-rs's initialization functions.
pub fn construct_rng(hwrng: impl RngCore) {
    RNG.lock(|r| {
        r.replace(Some(Rng {
            inner: SelectedRng::from_rng(hwrng).expect("Hardware RNG failed to provide entropy"),
        }))
    });
}

/// Obtains a suitably initialized random number generator.
///
/// This may be used by threads or tasks. To avoid synchronizion overhead, in the future,
/// dependency injection for task and thread generation might be provided through the riot-rs
/// macros.
#[inline]
pub fn get_rng() -> Rng {
    RNG.lock(|r| r.borrow_mut().as_mut().map(|main| main.split()))
        .expect("Initialization should have populated the RNG")
}
