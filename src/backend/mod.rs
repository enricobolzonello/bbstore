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

pub(crate) use store::BBStore;
