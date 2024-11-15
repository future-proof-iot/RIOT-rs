//! Provides a [`block_on()`] function to use futures from a thread.

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use riot_rs_threads::{current_pid, flags, flags::ThreadFlags, ThreadId};

const THREAD_FLAG_WAKER: ThreadFlags = 1; // TODO: find more appropriate value

fn wake(ptr: *const ()) {
    #[expect(clippy::cast_possible_truncation)]
    let thread_id = ThreadId::new(ptr as usize as u8);
    flags::set(thread_id, THREAD_FLAG_WAKER);
}

static VTABLE: RawWakerVTable = RawWakerVTable::new(
    // clone
    |ptr| RawWaker::new(ptr, &VTABLE),
    wake,
    wake,
    wake,
);

/// Runs a future to completion.
///
/// This runs the given future on the current thread, blocking until it is complete, and yielding its resolved result.
///
/// # Panics
///
/// Panics when not called from a thread.
pub fn block_on<F: Future>(mut fut: F) -> F::Output {
    // safety: we don't move the future after this line.
    let mut fut = unsafe { Pin::new_unchecked(&mut fut) };

    let raw_waker = RawWaker::new(usize::from(current_pid().unwrap()) as *const (), &VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut cx = Context::from_waker(&waker);
    loop {
        if let Poll::Ready(res) = fut.as_mut().poll(&mut cx) {
            return res;
        }
        flags::wait_any(THREAD_FLAG_WAKER);
    }
}
