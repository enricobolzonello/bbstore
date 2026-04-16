use crate::backend::{Command, store_backend::BBStoreBackend};
use anyhow::{Result, anyhow};
use log::debug;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::{mpsc, oneshot},
    thread,
};

const MAX_BATCH_SIZE: usize = 64;

pub(crate) fn actor_loop(rx: mpsc::Receiver<Command>, shard: &mut BBStoreBackend<String, String>) {
    loop {
        let first = match rx.recv() {
            Ok(cmd) => cmd,
            Err(_) => return,
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
                Command::Write { key, value, ack } => {
                    shard.insert(key, value);
                    let _ = ack.send(());
                }
                Command::Read { key, reply } => {
                    let value = shard.get(&key).cloned();
                    let _ = reply.send(value);
                }
            }
        }
    }
}

pub struct BBStore {
    channels: Vec<mpsc::Sender<Command>>,
    num_shards: usize,
}

impl BBStore {
    pub fn new(num_shards: usize) -> Self {
        let mut channels = Vec::new();
        for _ in 0..num_shards {
            let (tx, rx) = mpsc::channel::<Command>();
            thread::spawn(move || {
                actor_loop(rx, &mut BBStoreBackend::default());
            });
            channels.push(tx);
        }

        Self {
            channels,
            num_shards,
        }
    }

    pub fn insert(&self, key: String, value: String) -> Result<()> {
        let shard_key = self.shard_index(&key);
        let tx = self.channels[shard_key].clone();

        debug!("Inserting ({},{}) in shard {}", key, value, shard_key);

        let (ack_tx, ack_rx) = oneshot::channel();
        tx.send(Command::Write {
            key,
            value,
            ack: ack_tx,
        })?;

        ack_rx.recv()?;

        Ok(())
    }

    pub fn get(&self, key: String) -> Result<Option<String>> {
        let shard_key = self.shard_index(&key);
        let tx = self.channels[shard_key].clone();

        let (ack_tx, ack_rx) = oneshot::channel();
        tx.send(Command::Read { key, reply: ack_tx })?;

        ack_rx.recv().map_err(|e| anyhow!(e))
    }

    fn shard_index(&self, key: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize % self.num_shards
    }
}
