#![cfg_attr(feature = "select", feature(macro_metavar_expr))]
#![warn(missing_docs)]
#![doc = include_str!("../readme.md")]

/// Re-export of the [paste](https://docs.rs/paste/latest/paste/) crate.
#[cfg(feature = "select")]
pub mod paste {
    pub use paste::*;
}

use futures_concurrency::{stream::Merge, vec};
use smol::{
    Timer,
    future::FutureExt,
    stream::{Stream, StreamExt},
};
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

/// Creates tasks that get re-created infinitely as long as they are being polled.
/// Just a wrapper around [`Repeats`] and [`RepeatsBuilder`].
///
/// ```
/// # #[macro_use] extern crate smol_concurrency_tools;
/// # use std::time::Duration;
/// # use smol::{Timer, stream::StreamExt};
/// # smol::block_on(async {
/// let mut streams = repeat!(
///     async {
///         Timer::after(Duration::from_secs(1)).await;
///         1
///     },
///     async {
///         Timer::after(Duration::from_secs(2)).await;
///         2
///     }
/// );
/// let mut total = 0;
/// while let Some(p) = streams.next().await {
///     println!("{p}");
///     total += 1;
///     if total > 5 {
///         break;
///     }
/// }
/// # });
/// ```
#[macro_export]
macro_rules! repeat {
    ($(async $y:expr),*,) => {
        repeat!($($y),*)
    };
    ($(async $y:expr),*) => {
        {
            let mut a = smol_concurrency_tools::repeats();
            $(a = a.add(async || $y));*;
            a.finish()
        }
    };
}

/// A builder struct for [`Repeats`].
/// ```
/// # use smol_concurrency_tools::repeats;
/// # use smol::{stream::StreamExt};
/// # smol::block_on(async {
/// let mut rep = repeats().add(async || 1).finish();
/// for _ in 0..1000 {
///     assert_eq!(rep.next().await, Some(1));
/// }
/// # })
/// ```
pub struct RepeatsBuilder<'l, T> {
    inner: Vec<Repeat<'l, T>>,
}
impl<'l, T> RepeatsBuilder<'l, T> {
    /// Add another function, which should construct a future.
    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn add<P: FnMut() -> R + 'l + 'static + Send, R: Future<Output = T> + 'l + Send>(
        mut self,
        func: P,
    ) -> Self {
        self.inner.push(Repeat::new(func));
        self
    }
    /// Finalize the building process.
    #[must_use]
    pub fn finish(self) -> Repeats<'l, T> {
        Repeats {
            inner: self.inner.merge(),
        }
    }
}
/// Stream which repeats a set of tasks infinitely (i.e as long as it gets awaited).
pub struct Repeats<'l, T> {
    inner: vec::Merge<Repeat<'l, T>>,
}
impl<'l, T> Stream for Repeats<'l, T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.poll_next(cx)
    }
}

/// Create a [RepeatsBuilder].
pub fn repeats<'l, T>() -> RepeatsBuilder<'l, T> {
    RepeatsBuilder { inner: Vec::new() }
}

/// A Stream which contains a task which will be repeated infinitely (i.e as long as it gets awaited).
pub struct Repeat<'l, T: 'l> {
    func: Box<dyn FnMut() -> Pin<Box<dyn Future<Output = T> + 'l + Send>> + Send>,
    inner: RepeatInner<'l, T>,
}
impl<'l, T: 'l> Repeat<'l, T> {
    /// Create a new repeating task, with a function which should repeat infinitely.
    pub fn new<P: FnMut() -> R + 'l + 'static + Send, R: Future<Output = T> + 'l + Send>(
        mut func: P,
    ) -> Self {
        Self {
            func: Box::new(move || Box::pin(func())),
            inner: RepeatInner::NotSpawned,
        }
    }
}

impl<'l, T: 'l> Future for Repeat<'l, T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut self.inner {
            RepeatInner::NotSpawned => {
                self.inner = RepeatInner::Spawned(Box::pin((self.func)()));
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            RepeatInner::Spawned(future) => match future.poll(cx) {
                Poll::Ready(out) => {
                    self.inner = RepeatInner::Spawned(Box::pin((self.func)()));
                    Poll::Ready(out)
                }
                Poll::Pending => Poll::Pending,
            },
        }
    }
}
impl<'l, T: 'l> Stream for Repeat<'l, T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll(cx).map(|a| Some(a))
    }
}

enum RepeatInner<'l, T> {
    NotSpawned,
    Spawned(Pin<Box<dyn Future<Output = T> + 'l + Send>>),
}

#[allow(missing_docs)]
pub mod results;

/// A clone of the popular `select!` macro, but made compatible with `smol`.
/// Requires the feature `select` and the nightly feature `macro_metavar_expr`. Disabled by default due to these requirements.
/// ```
/// #![feature(macro_metavar_expr)]
/// # #[macro_use] extern crate smol_concurrency_tools;
/// # use smol_concurrency_tools::{select, Interval};
/// # use smol::{stream::StreamExt};
/// # use std::time::{Duration, Instant};
/// # smol::block_on(async {
/// let mut sec1 = Interval::new(Duration::from_secs(1));
/// let mut sec2 = Interval::new(Duration::from_secs(2));
/// let timer = Instant::now();
/// # let mut total = 0;
/// loop {
///     select!(
///         (sec1.next(), |d| {
///             let d = d.unwrap();
///             println!(
///                 "1 second - since_last: {:0.5}s, total_time: {:0.5}s",
///                 d.as_secs_f32(),
///                 timer.elapsed().as_secs_f32()
///             );
///         }),
///         (sec2.next(), |d| {
///             let d = d.unwrap();
///             println!(
///                 "2 second - since_last: {:0.5}s, total_time: {:0.5}s",
///                 d.as_secs_f32(),
///                 timer.elapsed().as_secs_f32()
///             );
///             # total += 1;
///             # if total > 5 {
///                 # break;                
///             # }
///         })
///     );
/// }
/// # });
/// ```
#[cfg(feature = "select")]
#[macro_export]
macro_rules! select {
    ($(($input:expr, |$pat:pat_param| $func:expr)),+) => {{
        smol_concurrency_tools::paste::paste! {
            use futures_concurrency::future::{Race as _};
            use smol_concurrency_tools::results::[<SelectionResult ${count($input)}>] as __ReturnType;
            let futures = ($(async { __ReturnType::[<Res ${index()}>] ($input.await)}),*).race();
            match futures.await {
                $(__ReturnType::[<Res ${index()}>]($pat) => $func),*
            }
        }
    }};
}

/// A Stream which runs infinitely and returns on a set interval.
/// ```
/// # use smol_concurrency_tools::{Interval};
/// # use smol::{stream::StreamExt};
/// # use std::time::{Duration, Instant};
/// # smol::block_on(async {
/// let mut interval = Interval::new(Duration::from_secs(2));
/// # let mut total = 0;
/// while let Some(_) = interval.next().await {
///     // runs every two seconds...
///     # total += 1;    
///     # if total > 5 {
///     #   break;        
///     # }
/// }
/// # });
/// ```
pub struct Interval {
    time: Duration,
    timer: Timer,
    elapse: Instant,
}
impl Interval {
    /// Creates a new interval.
    pub fn new(time: Duration) -> Self {
        Self {
            time,
            timer: Timer::after(time),
            elapse: Instant::now(),
        }
    }
}
impl Stream for Interval {
    type Item = Duration;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.timer.poll(cx) {
            Poll::Ready(_) => {
                self.timer = Timer::after(self.time);
                let elapsed = self.elapse.elapsed();
                self.elapse = Instant::now();
                Poll::Ready(Some(elapsed))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
