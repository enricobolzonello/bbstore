use tokio::sync::oneshot;

mod store;
mod store_backend;

pub(crate) enum BackendCommand {
    Write {
        key: String,
        value: String,
        ack: oneshot::Sender<()>,
    },
    Read {
        key: String,
        reply: oneshot::Sender<Option<String>>,
    },
}

#[cfg(feature = "benchmarking")]
pub use store::BBStore;
#[cfg(not(feature = "benchmarking"))]
pub(crate) use store::BBStore;
