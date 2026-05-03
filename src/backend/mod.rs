use crate::ByteString;
use tokio::sync::oneshot;

mod store;
mod store_backend;

pub(crate) enum BackendCommand {
    Write {
        key: ByteString,
        value: ByteString,
        ack: oneshot::Sender<()>,
    },
    Read {
        key: ByteString,
        reply: oneshot::Sender<Option<ByteString>>,
    },
}

#[cfg(feature = "benchmarking")]
pub use store::BBStore;
#[cfg(not(feature = "benchmarking"))]
pub(crate) use store::BBStore;
