//! The module defines the `Future` trait, as well as a simple struct that wraps any type T
//! and implements the `Future` trait.

/// The trait represents a value that will become available in the future.
/// Concrete implementations of the trait only need to provide the implementation of the
/// `await` method, which is to return the value once it becomes available, possibly even
/// blocking the running thread until it does.
///
/// The trait provides methods that allow computations to be built up on top of the
/// future result. These are methods are analogous to the Functor/Monad functions. Using these
/// methods it is possible to define a sequence of computations to be performed on the value
/// behind the original future. Finally, once the computation is defined, the caller can wait
/// for the final result to become available (by invoking the resulting `Future`'s `await` method).
pub trait Future {
    /// The type of the object that the `Future` will produce from its `await` method.
    type Output;
    /// Return the value behind the `Future`. Possibly blocks the running thread.
    /// Implementations need to take care that the result could still eventually become
    /// available if they do block the calling thread.
    fn await(self) -> Self::Output;

    /// Returns a new `Future` instance whose value will be the result of the given
    /// function applied to the underlying value of the original `Future`. Simply
    /// calling this method does not block the caller.
    #[inline]
    fn map<U, Func>(self, f: Func) -> FutureThen<Self, Func>
            where Self: Sized,
                  Func: FnOnce(Self::Output) -> U
    {
        then(self, f)
    }

    /// Alias for `map`.
    #[inline]
    fn then<U, Func>(self, f: Func) -> FutureThen<Self, Func>
            where Self: Sized,
                  Func: FnOnce(Self::Output) -> U
    {
        then(self, f)
    }

    /// Returns a new future, provided by the given function `f` when called with the
    /// underlying value of the original `Future`.
    #[inline]
    fn bind<U, F, Func>(self, f: Func) -> FutureBind<Self, Func>
            where Self: Sized,
                  Func: FnOnce(Self::Output) -> F,
                  F: Future<Output=U>
    {
        bind(self, f)
    }
}

/// A simple implementation of the `Future` trait that returns the wrapped object from its
/// `await`.
pub struct FutureWrap<T>(pub T);
impl<T> Future for FutureWrap<T> {
    type Output = T;
    fn await(self) -> T { self.0 }
}

/// The struct represents the result of applying a function to an original future.
///
/// Naturally, the struct itself also implements the `Future` trait.
pub struct FutureThen<F, Func>
        where F: Future {
    inner: F,
    closure: Func,
}

impl<F, Func, T, U> Future for FutureThen<F, Func>
        where F: Future<Output=T>,
              Func: FnOnce(T) -> U {
    type Output = U;

    fn await(self) -> U {
        let res = self.inner.await();
        (self.closure)(res)
    }
}

#[inline]
pub fn then<T, U, F, Func>(future: F, foo: Func) -> FutureThen<F, Func>
        where F: Future<Output=T>,
              Func: FnOnce(T) -> U {
    FutureThen {
        inner: future,
        closure: foo,
    }
}

/// The struct represents the future returned by the `Future::bind` method.
pub struct FutureBind<OrigFuture, Func>
        where OrigFuture: Future {
    inner: OrigFuture,
    closure: Func,
}

impl<OrigFuture, Func, T, U, F> Future for FutureBind<OrigFuture, Func>
        where F: Future<Output=U>,
              OrigFuture: Future<Output=T>,
              Func: FnOnce(T) -> F {
    type Output = <Func::Output as Future>::Output;

    fn await(self) -> U {
        let res = self.inner.await();
        let next = (self.closure)(res);
        next.await()
    }
}

pub fn bind<T, U, OrigFuture, NextFuture, Func>(
        future: OrigFuture,
        foo: Func
        ) -> FutureBind<OrigFuture, Func>
        where OrigFuture: Future<Output=T>,
              Func: FnOnce(T) -> NextFuture,
              NextFuture: Future<Output=U> {
    FutureBind {
        inner: future,
        closure: foo,
    }
}

/// Lifts the given object into a `Future` context. This means that the returned type implements
/// the `Future` trait in such a way that its `await` method will return the originally given
/// object.
///
/// # Example
///
/// ```rust
/// use zukunft::{lift, Future};
/// assert_eq!(lift(5u8).await(), 5u8);
/// ```
pub fn lift<T>(obj: T) -> FutureWrap<T> {
    FutureWrap(obj)
}


#[cfg(test)]
pub mod tests {
    use super::{Future, lift};

    struct MockFuture;
    impl Future for MockFuture {
        type Output = u8;
        fn await(self) -> u8 { 100 }
    }

    #[test]
    fn test_lift_value_into_future() {
        let val = 5u8;
        let future = lift(val);
        assert_eq!(future.await(), val);
    }

    #[test]
    fn test_map_to_same_type() {
        let future = lift(5u8);
        let double_future = future.map(|val| 2*val);
        assert_eq!(double_future.await(), 10);
    }

    #[test]
    fn test_map_to_different_type() {
        let future = lift(5u8);
        let future_vec = future.map(|len| {
            let mut vec = Vec::new();
            for _ in 0..len { vec.push(0u8); }
            vec
        });
        assert_eq!(future_vec.await(), vec![0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_chain_map() {
        let future = lift(5u8);
        let res = future.map(|val| 2*val).map(|val| val + 1);
        assert_eq!(res.await(), 11);
    }

    #[test]
    fn test_bind_same_inner_type() {
        let future = lift(5u8);
        let res = future.bind(|val| lift(val + 50));
        assert_eq!(res.await(), 55);
    }

    #[test]
    fn test_bind_different_inner_type() {
        let future = lift(5u8);
        let res = future.bind(|val| lift(vec![0; val as usize]));
        assert_eq!(res.await(), vec![0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_bind_different_future_impl() {
        let future = MockFuture;
        // `lift` returns a FutureWrap struct, which is different than the
        // MockFuture original struct...
        let res = future.bind(|val| lift(val*2));
        assert_eq!(res.await(), 200);
    }
}
