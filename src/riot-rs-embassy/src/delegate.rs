//! Delegate or lend an object to another task

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

use crate::sendcell::SendCell;

/// [`Delegate`] or lend an object to another task.
///
/// This struct can be used to lend a `&mut T` to another task on the same executor.
/// The other task can then call a closure on it. After that, the `&mut T` is returned
/// to the original task.
///
/// Under the hood, [`Delegate`] leverages [`SendCell`] to ensure the delegated
/// object stays on the same executor.
///
/// Example:
/// ```Rust
/// static SOME_VALUE: Delegate<u32> = Delegate::new();
///
/// // in some task
/// async fn foo() {
///   let mut my_val = 0u32;
///   SOME_VALUE.lend(&mut my_val).await;
///   assert_eq!(my_val, 1);
/// }
///
/// // in some other task
/// async fn bar() {
///   SOME_VALUE.with(|val| *val = 1).await;
/// }
/// ```
///
/// TODO: this is a PoC implementation.
/// - takes 24b for each delegate (on arm), which seems too much.
/// - doesn't protect at all against calling [`lend()`](Delegate::lend) or
///   [`with()`](Delegate::with) multiple times
///   each, breaking safety assumptions. So while the API seems OK, the implementation
///   needs work.
#[derive(Default)]
pub struct Delegate<T> {
    send: Signal<CriticalSectionRawMutex, SendCell<*mut T>>,
    reply: Signal<CriticalSectionRawMutex, ()>,
}

impl<T> Delegate<T> {
    /// Creates a new [`Delegate`].
    pub const fn new() -> Self {
        Self {
            send: Signal::new(),
            reply: Signal::new(),
        }
    }

    /// Lends an object.
    ///
    /// This blocks until another task called [`with()`](Delegate::with).
    pub async fn lend<'a, 'b: 'a>(&'a self, something: &'b mut T) {
        let spawner = Spawner::for_current_executor().await;
        self.send
            .signal(SendCell::new(something as *mut T, spawner));

        self.reply.wait().await
    }

    /// Calls a closure on a lended object.
    ///
    /// This blocks until another task called [`lend(something)`](Delegate::lend).
    pub async fn with<U>(&self, func: impl FnOnce(&mut T) -> U) -> U {
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
        // TODO: it is actually possible to call `with()` twice, which breaks assumptions.
        let result = func(unsafe { data.get(spawner).unwrap().as_mut().unwrap() });
        self.reply.signal(());
        result
    }
}
