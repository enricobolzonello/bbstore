use crate::{
    BBStoreConfig,
    backend::{BackendCommand, store_backend::BBStoreBackend},
};
use anyhow::{Result, anyhow};
use log::debug;
use std::hash::{DefaultHasher, Hash, Hasher};

use tokio::sync::{mpsc, oneshot};

const MAX_BATCH_SIZE: usize = 64;

pub(crate) async fn actor_loop(
    mut rx: mpsc::Receiver<BackendCommand>,
    mut shard: BBStoreBackend<String, String>,
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

        debug!("batch ready with size {}", batch.len());

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

    pub async fn insert(&self, key: String, value: String) -> Result<()> {
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

    pub async fn get(&self, key: String) -> Result<Option<String>> {
        let shard_key = self.shard_index(&key);
        let tx = self.channels[shard_key].clone();

        let (ack_tx, ack_rx) = oneshot::channel();
        tx.send(BackendCommand::Read { key, reply: ack_tx }).await?;

        ack_rx.await.map_err(|e| anyhow!(e))
    }

    pub fn config(&self) -> BBStoreConfig {
        self.config.clone()
    }

    fn shard_index(&self, key: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize % self.config.num_shards
    }
}
