//! A simple library for composing the results of future computation results with a minimal
//! overhad.
//!
//! The library does not concern itself with the way that the future result is obtained.
//!
//! It exports the `Future` trait, which is in essence quite similar to the `Iterator` trait.
//! Implementors are required to state which type their `Future` implementation produces and
//! provide the implementation of a single method---`await`---which blocks the running thread
//! and eventually returns an instance of the declared type.
//!
//! The `Future` trait itself provides implementations of methods that allow any `Future`'s
//! implementation to be composed by way of Functor/Monad functions. Therefore, to create a
//! new computation that depends on the future value behind any `Future` implementation, the
//! following can be used:
//!
//! ```rust
//! use zukunft::{Future, lift};
//! // Create a Future that whose await method would immediately resolve to the given value
//! let future = lift(5u8);
//! // Create a computation that would double the original future's value. This, of course,
//! // also returns a future.
//! let future = future.map(|val| 2*val);
//! // Once we have that, increment the value by one
//! let future = future.map(|val| val + 1);
//! // Finally we've set up the computations that should be performed and we sit and wait
//! // for it.
//! assert_eq!(future.await(), 11);
//! ```
//!
//! Additionally, a computation can depend on multiple future results:
//!
//! ```rust
//! use zukunft::{Future, lift};
//! let first = lift(5u8);
//! let second = lift(2u8);
//! let future = first.bind(|val1| {
//!     second.map(move |val2| val1 + val2)
//! });
//! assert_eq!(future.await(), 7);
//! ```
//!
//! Finally, the `mpsc` module provides a simple `Future` implementation that waits for
//! a value to become available on an `std::mpsc` channel before resoloving the future.
//!
//! ```rust
//! use std::thread;
//! use zukunft::{Future, ChannelFuture};
//! let (future, resolver) = ChannelFuture::new();
//! // We can compose computations on top of the future, even if its value is unknown
//! // as of yet...
//! let future = future.map(|val| 2*val);
//! // Resolve the future from a different thread...
//! thread::spawn(move || resolver.send(2u8).unwrap());
//! assert_eq!(future.await(), 4);
//! ```
//!
//! As can be seen from the short examples, the library is not concerned with integrating
//! with any event-loop style mechanism that would allow the computations to automatically
//! run whenever the underlying value resolves. Rather, its only (and quite humble) goal is
//! to provide an API for composing computations on top of results that are not immediately
//! available.
//!
//! Check out [eventual](https://github.com/carllerche/eventual) or
//! [gj](https://github.com/dwrensha/gj) for a future implementation that is async/evented.

mod base;
pub mod mpsc;

pub use base::Future;
pub use base::FutureThen;
pub use base::FutureBind;
pub use base::FutureWrap;
pub use base::lift;

pub use mpsc::ChannelFuture;
