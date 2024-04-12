//! Provides a seeded random number generator depending on RIOT-rs's configuration.
//!
//! The module provides functions for use by applications, [`fast_rng()`] and [`crypto_rng()`],
//! which produce owned types that provide the [`rand_core::RngCore`] and
//! [`rand_core::CryptoRng`'] traits, respectively.
//!
//! The crate abstracts over multiple aspects of RNGs:
//! * Where do we take a valid seed for the RNG from?
//! * What's the type of RNG that we take along?
//! * Is RNG state shared across cores, threads, tasks or not at all?
//!
//! No matter the choices taken (eventually through the application's setup), all is hidden behind
//! the [`FastRng`] and [`CryptoRng`] types.
//!
//! Before accessing the RNG, it needs to be initialized through the [`construct_rng()`'] function.
//! This is taken care of by the `riot-rs-embassy` initialization functions. Applications can
//! ensure that this has happened by depending on the laze feature `random`.
//!
//! ---
//!
//! Currently, this provides very little choice, and little fanciness: It (more or less
//! arbitrarily) uses the [rand_chacha::ChaCha20Rng] generator as a shared global RNG, and
//! [rand_pcg::Pcg32] is decided yet for the fast one. Neither the algorithm nor the size of
//! FastRng or CryptoRng is guaranteed.
#![no_std]

use rand_core::{RngCore, SeedableRng};

/// A global RNG.
// The Mutex<RefCell> can probably be simplified
static RNG: embassy_sync::blocking_mutex::Mutex<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    core::cell::RefCell<Option<SelectedRng>>,
> = embassy_sync::blocking_mutex::Mutex::new(core::cell::RefCell::new(None));

/// Type of the global RNG when needing the ability to produce cryptographially secure random
/// numbers.
///
/// If calls to [`rng()`] are rare, it may even make sense to move the HWRNG in here to get a
/// ZST global.
#[cfg(feature = "csprng")]
pub(crate) type SelectedRng = rand_chacha::ChaCha20Rng;

/// Type of the global RNG when cryptographically secure random numbers are not needed.
#[cfg(not(feature = "csprng"))]
pub(crate) type SelectedRng = rand_pcg::Pcg32;

/// Locks the global RNG for a single operation.
///
/// ## Panics
///
/// … if initialization did not happen.
///
/// ## Deadlocks
///
/// … if the action attempts to lock RNG.
fn with_global<R>(action: impl FnOnce(&mut SelectedRng) -> R) -> R {
    RNG.lock(|i| {
        action(
            i.borrow_mut()
                .as_mut()
                .expect("Initialization should have populated RNG"),
        )
    })
}

/// The OS provided fast random number generator.
///
/// This will generally be faster to produce random numbers than [`CryptoRng`].
///
/// Such an RNG can be requested by any component, and will always be seeded appropriately.
pub struct FastRng {
    inner: rand_pcg::Pcg32,
    // Make the type not Send to later allow using thread-locals
    _private: core::marker::PhantomData<*const ()>,
}

// Re-implementing the trait rather than Deref'ing into inner: This avoids leaking implementation
// details to users who might otherwise come to depend on platform specifics of the FastRng.
impl RngCore for FastRng {
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

/// The OS provided cryptographically secure random number generator.
///
/// Such an RNG can be requested by any component, and will always be seeded appropriately.
#[cfg(feature = "csprng")]
pub struct CryptoRng {
    // Make the type not Send to later allow using thread-locals
    pub(crate) _private: core::marker::PhantomData<*const ()>,
}

#[cfg(feature = "csprng")]
mod csprng {
    use super::*;

    // Re-implementing the trait rather than Deref'ing into inner: This avoids leaking implementation
    // details to users who might otherwise come to depend on platform specifics of the CryptoRng.
    impl RngCore for CryptoRng {
        fn next_u32(&mut self) -> u32 {
            with_global(|i| i.next_u32())
        }
        fn next_u64(&mut self) -> u64 {
            with_global(|i| i.next_u64())
        }
        fn fill_bytes(&mut self, buf: &mut [u8]) {
            with_global(|i| i.fill_bytes(buf))
        }
        fn try_fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), rand_core::Error> {
            with_global(|i| i.try_fill_bytes(buf))
        }
    }

    impl rand_core::CryptoRng for super::CryptoRng {}

    /// Asserts that SelectedRng is CryptoRng, justifying the implementation above.
    fn static_assert_is_cryptorng() -> impl rand_core::CryptoRng {
        let result: super::SelectedRng = unreachable!("This function is for type checking only");
        result
    }
}

/// Populates the global RNG from a seed value.
///
/// This is called by RIOT-rs's initialization functions.
pub fn construct_rng(hwrng: impl RngCore) {
    RNG.lock(|r| {
        r.replace(Some(
            SelectedRng::from_rng(hwrng).expect("Hardware RNG failed to provide entropy"),
        ))
    });
}

/// Obtains a suitably initialized fast random number generator.
#[inline]
pub fn fast_rng() -> FastRng {
    FastRng {
        inner: with_global(|i| rand_pcg::Pcg32::from_rng(i).expect("Global RNG is infallible")),
        _private: Default::default(),
    }
}

/// Obtains a suitably initialized cryptographically secure random number generator.
#[inline]
#[cfg(feature = "csprng")]
pub fn crypto_rng() -> CryptoRng {
    CryptoRng {
        _private: Default::default(),
    }
}
