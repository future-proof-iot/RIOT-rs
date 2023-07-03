use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use riot_rs_core::thread::{current_pid, sleep, wakeup, ThreadId};

fn wake(ptr: *const ()) {
    // wake
    let thread_id = ptr as usize as ThreadId;
    wakeup(thread_id);
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

    let raw_waker = RawWaker::new(current_pid().unwrap() as usize as *const (), &VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut cx = Context::from_waker(&waker);
    loop {
        if let Poll::Ready(res) = fut.as_mut().poll(&mut cx) {
            return res;
        }
        sleep();
    }
}
