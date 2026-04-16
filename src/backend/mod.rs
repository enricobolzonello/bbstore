use std::sync::oneshot;

mod store;
mod store_backend;

pub(crate) enum Command {
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

pub use store::BBStore;
