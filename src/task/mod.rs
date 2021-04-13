use futures::executor::LocalPool;
use futures::future::Future;
use futures::task::LocalSpawnExt;
use futures::task::{Context, Poll};
use std::cell::UnsafeCell;
use std::pin::Pin;

thread_local! {
    static CURRENT_THREAD_EXECUTOR: UnsafeCell<LocalPool> = LocalPool::new().into();
}

pub fn spawn<F>(f: F)
where
    F: Future<Output = ()> + 'static,
{
    CURRENT_THREAD_EXECUTOR.with(|executor| {
        let executor = unsafe { &mut *executor.get() };
        executor
            .spawner()
            .spawn_local(f)
            .expect("Failed to enqueue task");
        executor.run_until_stalled()
    })
}

pub struct SleepFuture {
    duration: u32,
    slept: bool,
}

impl SleepFuture {
    pub fn new(duration: u32) -> Self {
        Self {
            duration,
            slept: false,
        }
    }
}

impl Future for SleepFuture {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("poll");
        match self.slept {
            true => Poll::Ready(()),
            false => {
                let waker = cx.waker().clone();
                crate::timeout_add(self.duration, move || {
                    println!("timeout done");
                    waker.wake_by_ref();
                    CURRENT_THREAD_EXECUTOR.with(|executor| {
                        let executor = unsafe { &mut *executor.get() };
                        executor.run_until_stalled()
                    });
                    false
                });
                self.slept = true;
                Poll::Pending
            }
        }
    }
}

pub fn sleep(duration: u32) -> SleepFuture {
    SleepFuture::new(duration)
}
