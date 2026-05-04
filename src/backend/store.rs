use crate::{
    BBStoreConfig, ByteStr, ByteString,
    backend::{BackendCommand, store_backend::BBStoreBackend},
};
use anyhow::{Result, anyhow};
use std::hash::{DefaultHasher, Hash, Hasher};
use tracing::{debug, warn};

use tokio::sync::{mpsc, oneshot};

const MAX_BATCH_SIZE: usize = 64;

pub(crate) async fn actor_loop(
    mut rx: mpsc::Receiver<BackendCommand>,
    mut shard: BBStoreBackend<ByteString, ByteString>,
) {
    loop {
        let first = match rx.recv().await {
            Some(cmd) => cmd,
            None => return,
        };

        let mut batch = Vec::with_capacity(MAX_BATCH_SIZE);
        batch.push(first);

        while batch.len() < MAX_BATCH_SIZE {
            match rx.try_recv() {
                Ok(cmd) => batch.push(cmd),
                Err(_) => break,
            }
        }

        if batch.len() == MAX_BATCH_SIZE {
            warn!(size = MAX_BATCH_SIZE, "batch full, backpressure likely");
        }
        debug!(size = batch.len(), "batch ready");

        for cmd in batch {
            match cmd {
                BackendCommand::Write { key, value, ack } => {
                    shard.insert(key, value);
                    let _ = ack.send(());
                }
                BackendCommand::Read { key, reply } => {
                    let value = shard.get(&key).cloned();
                    let _ = reply.send(value);
                }
            }
        }
    }
}

pub struct BBStore {
    channels: Vec<mpsc::Sender<BackendCommand>>,
    config: BBStoreConfig,
}

impl BBStore {
    pub fn new(config: BBStoreConfig) -> Self {
        let mut channels = Vec::new();
        for _ in 0..config.num_shards {
            let (tx, rx) = mpsc::channel::<BackendCommand>(config.buffer_size);
            tokio::spawn(actor_loop(rx, BBStoreBackend::default()));
            channels.push(tx);
        }

        Self { channels, config }
    }

    pub async fn insert(&self, key: ByteString, value: ByteString) -> Result<()> {
        let shard_key = self.shard_index(&key);
        let tx = self.channels[shard_key].clone();

        debug!("Inserting ({},{}) in shard {}", key, value, shard_key);

        let (ack_tx, ack_rx) = oneshot::channel();
        tx.send(BackendCommand::Write {
            key,
            value,
            ack: ack_tx,
        })
        .await?;

        ack_rx.await?;

        Ok(())
    }

    pub async fn get(&self, key: ByteString) -> Result<Option<ByteString>> {
        let shard_key = self.shard_index(&key);
        let tx = self.channels[shard_key].clone();

        debug!("Getting ({}) from shard {}", key, shard_key);

        let (ack_tx, ack_rx) = oneshot::channel();
        tx.send(BackendCommand::Read { key, reply: ack_tx }).await?;

        ack_rx.await.map_err(|e| anyhow!(e))
    }

    pub fn config(&self) -> BBStoreConfig {
        self.config.clone()
    }

    fn shard_index(&self, key: &ByteStr) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize % self.config.num_shards
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BBStoreConfig;

    fn config() -> BBStoreConfig {
        BBStoreConfig {
            num_shards: 2,
            address: "127.0.0.1".into(),
            buffer_size: 10,
        }
    }

    #[tokio::test]
    async fn bbstore_get_missing_key_returns_none() {
        let store = BBStore::new(config());
        assert!(store.get("missing".into()).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn bbstore_insert_then_get_returns_value() {
        let store = BBStore::new(config());
        store.insert("k".into(), "v".into()).await.unwrap();
        assert_eq!(store.get("k".into()).await.unwrap(), Some("v".into()));
    }

    #[tokio::test]
    async fn bbstore_insert_overwrites_existing_key() {
        let store = BBStore::new(config());
        store.insert("k".into(), "v1".into()).await.unwrap();
        store.insert("k".into(), "v2".into()).await.unwrap();
        assert_eq!(store.get("k".into()).await.unwrap(), Some("v2".into()));
    }

    #[tokio::test]
    async fn bbstore_distinct_keys_do_not_collide() {
        let store = BBStore::new(config());
        store.insert("a".into(), "1".into()).await.unwrap();
        store.insert("b".into(), "2".into()).await.unwrap();
        assert_eq!(store.get("a".into()).await.unwrap(), Some("1".into()));
        assert_eq!(store.get("b".into()).await.unwrap(), Some("2".into()));
    }

    #[tokio::test]
    async fn bbstore_keys_are_isolated_across_shards() {
        let store = BBStore::new(BBStoreConfig {
            num_shards: 4,
            ..config()
        });
        for i in 0..20 {
            store
                .insert(format!("key-{}", i).into(), format!("val-{}", i).into())
                .await
                .unwrap();
        }
        for i in 0..20 {
            assert_eq!(
                store.get(format!("key-{}", i).into()).await.unwrap(),
                Some(format!("val-{}", i).into())
            );
        }
    }
}
