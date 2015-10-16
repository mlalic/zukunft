//! Provides a channel-based Future implementation: the value that the Future resolves to
//! is expected to arrive on an `std::mpsc::Receiver`.

use base::Future;
use std::sync::mpsc;

/// An implementation of the `Future` trait that wraps an `std::mpsc::Receiver`. It resolves
/// to the first un-read value on the channel, possibly blocking until one is available.
pub struct ChannelFuture<T> {
    rx: mpsc::Receiver<T>,
}

impl<T> ChannelFuture<T> {
    /// Create a new `ChannelFuture` and a `mpsc::Sender` pair. The first value sent on
    /// the `Sender` will be the value that the `ChannelFuture` resolves to.
    pub fn new() -> (ChannelFuture<T>, mpsc::Sender<T>) {
        let (tx, rx) = mpsc::channel();
        (
            ChannelFuture { rx: rx },
            tx,
        )
    }

    /// Create a new `ChannelFuture` from an existing `Receiver`. The first value that
    /// is sent on the corresponding `Sender` will be the value that the `ChannelFuture`
    /// resolves to.
    pub fn from_receiver(rx: mpsc::Receiver<T>) -> ChannelFuture<T> {
        ChannelFuture { rx: rx }
    }
}

impl<T> Future for ChannelFuture<T> {
    type Output = T;
    fn await(self) -> T {
        self.rx.recv().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::ChannelFuture;
    use base::{Future, lift};

    #[test]
    fn test_future_awaits_resolution() {
        let (future, resolver) = ChannelFuture::new();
        thread::spawn(move || {
            thread::sleep_ms(10);
            assert!(resolver.send(10u8).is_ok());
        });
        assert_eq!(future.await(), 10u8);
    }

    #[test]
    fn test_map() {
        let (future, resolver) = ChannelFuture::new();
        let future = future.map(|val| 2*val);
        thread::spawn(move || {
            thread::sleep_ms(10);
            assert!(resolver.send(10u8).is_ok());
        });
        assert_eq!(future.await(), 20u8);
    }

    #[test]
    fn test_bind_sequences_two_channel_futures() {
        let (first_future, first_resolver) = ChannelFuture::new();
        let (second_future, second_resolver) = ChannelFuture::new();
        let future = first_future.bind(|val1| {
            // Now here, we need to use the `val1` to a result of a different future
            second_future.map(move |val2| val1 * val2)
        });
        thread::spawn(move || {
            // The futures are resolved out of the expected order...
            thread::sleep_ms(10);
            assert!(second_resolver.send(10u8).is_ok());
            thread::sleep_ms(10);
            assert!(first_resolver.send(6u8).is_ok());
        });
        assert_eq!(future.await(), 60);
    }

    #[test]
    fn test_composes_with_wrapped_bind() {
        let (future, resolver) = ChannelFuture::new();
        let future = future.bind(|val| lift(2 * val));
        thread::spawn(move || {
            thread::sleep_ms(10);
            assert!(resolver.send(10u8).is_ok());
        });
        assert_eq!(future.await(), 20u8);
    }
}
