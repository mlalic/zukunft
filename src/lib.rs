mod base;
pub mod mpsc;

pub use base::Future;
pub use base::FutureThen;
pub use base::FutureBind;
pub use base::FutureWrap;
pub use base::lift;

pub use mpsc::ChannelFuture;
