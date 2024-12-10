//! Delegate or lend an object to another task.

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use portable_atomic::{AtomicBool, Ordering};

use crate::sendcell::SendCell;

/// [`Delegate`]s or lends an object to another task.
///
/// This struct can be used to lend a `&mut T` to another task on the same executor.
/// The other task can then call a closure on it. After that, the `&mut T` is returned
/// to the original task.
///
/// Under the hood, [`Delegate`] leverages [`SendCell`] to ensure the delegated
/// object stays on the same executor.
///
/// # Example
///
/// ```
/// # use ariel_os_embassy::delegate::Delegate;
/// static SOME_VALUE: Delegate<u32> = Delegate::new();
///
/// // in some task
/// async fn foo() {
///   let mut my_val = 0u32;
///   // SAFETY: `lend` is only called once.
///   unsafe { SOME_VALUE.lend(&mut my_val).await; }
///   assert_eq!(my_val, 1);
/// }
///
/// // in some other task
/// async fn bar() {
///   SOME_VALUE.with(|val| *val = 1).await;
/// }
/// ```
// TODO: this is a PoC implementation.
// - Takes 25 B for each delegate (on arm), which seems too much.
#[derive(Default)]
pub struct Delegate<T> {
    send: Signal<CriticalSectionRawMutex, SendCell<*mut T>>,
    reply: Signal<CriticalSectionRawMutex, ()>,
    was_exercised: AtomicBool,
}

impl<T> !Send for Delegate<T> {}

impl<T> Delegate<T> {
    /// Creates a new [`Delegate`].
    pub const fn new() -> Self {
        Self {
            send: Signal::new(),
            reply: Signal::new(),
            was_exercised: AtomicBool::new(false),
        }
    }

    /// Lends an object.
    ///
    /// This blocks until [`Delegate::with()`] is called on the instance.
    ///
    /// # Safety
    ///
    /// This must only be called *once* per [`Delegate`] instance.
    pub async unsafe fn lend<'a, 'b: 'a>(&'a self, something: &'b mut T) {
        let spawner = Spawner::for_current_executor().await;
        self.send
            .signal(SendCell::new(something as *mut T, spawner));

        self.reply.wait().await
    }

    /// Calls a closure on a lent object.
    ///
    /// This blocks until [`Delegate::lend()`] is called on the instance.
    ///
    /// # Panics
    ///
    /// Panics if called multiple times on the same [`Delegate`] instance.
    pub async fn with<U>(&self, func: impl FnOnce(&mut T) -> U) -> U {
        // Enforce that the value be only populated once, panic otherwise.
        assert!(!self.was_exercised.swap(true, Ordering::AcqRel));

        let data = self.send.wait().await;
        let spawner = Spawner::for_current_executor().await;
        // SAFETY:
        // - SendCell guarantees that data `lend()`ed stays on the same executor,
        //   which is single-threaded
        // - `lend()` signals the raw pointer via `self.send`, but then waits for `self.reply` to be signaled.
        //   This function waits for the `self.send` signal, uses the dereferenced only inside the
        //   closure, then signals `self.reply`
        //   => the mutable reference is never used more than once
        // - the lifetime bound on `lend` enforces that the raw pointer outlives this `Delegate`
        //   instance
        let result = func(unsafe { data.get(spawner).unwrap().as_mut().unwrap() });
        self.reply.signal(());
        result
    }
}
