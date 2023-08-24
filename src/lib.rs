pub use log;

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use pin_project::pin_project;

// The result of a finished [`InstrumentFuture`]
#[derive(Debug)]
pub struct InstrumentFutureResult<R> {
    pub result: R,
    pub elapsed: Duration,
}

/// Wraps a future and determines exactly how long it took to execute
///
/// ```rust
/// let my_fut: impl Future<Output = ()> = foobar();
/// let res = InstrumentFuture::new(my_fut).await;
///
/// // get the result of `my_fut`
/// res.result;
///
/// // print the elapsed time of `my_fut`
/// println!("my_fut took {:?}", res.elapsed);
/// ```
#[derive(Debug)]
#[pin_project]
pub struct InstrumentFuture<F: Future> {
    #[pin]
    future: F,
    timer: Option<Instant>,
}

impl<F: Future> InstrumentFuture<F> {
    pub fn new(future: F) -> Self {
        Self {
            future,
            timer: None,
        }
    }
}

impl<F: Future> Future for InstrumentFuture<F> {
    type Output = InstrumentFutureResult<<F as Future>::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        if this.timer.is_none() {
            *this.timer = Some(Instant::now());
        }

        this.future.poll(cx).map(|r| InstrumentFutureResult {
            result: r,
            // SAFETY: `timer` is always `Some(T)` since we ensure it's always set to Some above
            elapsed: unsafe { this.timer.unwrap_unchecked() }.elapsed(),
        })
    }
}

/// Debug log how long a future took to execute
///
/// When `debug_assertions` are enabled, does a log::debug!() with the file!(), line!(), and elapsed time.
/// Without `debug_assertions`, simply is a no-op which executes future normally.
///
/// The argument to the function must be a unexecuted future. It will return a future you must `await` on.
/// This allows you to use the created instrumenting future later if desired since it doesn't `await` immediately.
///
/// There is also an optional one with a custom log message. `elapsed` is provided as a keyword arg to the literal,
/// so you must use it somewhere in there.
///
/// If you need custom behavior, you can make a custom instrumenting future using [`InstrumentFuture`]
///
/// Examples:
///
/// ```rust
/// let my_fut: impl Future<Output = ()> = foobar();
/// dbg_instrument!(my_fut).await;
///
/// let f = 0;
/// dbg_instrument!("custom_log_message {f}: {elapsed:?}", my_fut).await;
/// ```
#[macro_export]
macro_rules! dbg_instrument {
    ($fut:expr) => {
        async {
            if cfg!(debug_assertions) {
                $crate::instrument!($fut).await
            } else {
                $fut.await
            }
        }
    };

    ($log:literal, $fut:expr) => {{
        async {
            if cfg!(debug_assertions) {
                $crate::instrument!($log, $fut).await
            } else {
                $fut.await
            }
        }
    }};
}

/// Debug log how long a future took to execute
///
/// As opposed to [`dbg_instrument!`], this always logs regardless whether `debug_assertions` are enabled or not
///
/// The argument to the function must be a unexecuted future. It will return a future you must `await` on.
/// This allows you to use the created instrumenting future later if desired since it doesn't `await` immediately.
///
/// There is also an optional one with a custom log message. `elapsed` is provided as a keyword arg to the literal,
/// so you must use it somewhere in there.
///
/// If you need custom behavior, you can make a custom instrumenting future using [`InstrumentFuture`]
///
/// Examples:
///
/// ```rust
/// let my_fut: impl Future<Output = ()> = foobar();
/// instrument!(my_fut).await;
///
/// let f = 0;
/// instrument!("custom_log_message {f}: {elapsed:?}", my_fut).await;
/// ```
#[macro_export]
macro_rules! instrument {
    ($fut:expr) => {
        async { $crate::_instrument!($fut) }
    };

    ($log:literal, $fut:expr) => {{
        async { $crate::_instrument!($log, $fut) }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! _instrument {
    ($fut:expr) => {{
        let timed = $crate::InstrumentFuture::new($fut).await;

        let _file = file!();
        let _line = line!();
        let _elapsed = timed.elapsed;

        $crate::log::debug!("{_file}:{_line} completed in {_elapsed:?}");

        timed.result
    }};

    ($log:literal, $fut:expr) => {{
        let timed = $crate::InstrumentFuture::new($fut).await;

        $crate::log::debug!($log, elapsed = timed.elapsed);

        timed.result
    }};
}
