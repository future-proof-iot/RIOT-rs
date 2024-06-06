use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use riot_rs_threads::{current_pid, flags, flags::THREAD_FLAG_WAKEUP, ThreadId};

fn wake(ptr: *const ()) {
    // wake
    let thread_id = ThreadId::new(ptr as usize as u8);
    flags::set(thread_id, THREAD_FLAG_WAKEUP);
}

static VTABLE: RawWakerVTable = RawWakerVTable::new(
    // clone
    |ptr| RawWaker::new(ptr, &VTABLE),
    wake,
    wake,
    wake,
);

/// Run a future to completion, using thread_sleep()
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
        flags::wait_any(THREAD_FLAG_WAKEUP);
    }
}
