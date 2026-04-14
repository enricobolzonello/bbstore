#![feature(oneshot_channel)]
use anyhow::{Result, anyhow};
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::{mpsc, oneshot},
    thread,
};

const MAX_BATCH_SIZE: usize = 64;

struct BBStoreBackend<K, V> {
    mem: HashMap<K, V>,
}

impl<K, V> Default for BBStoreBackend<K, V> {
    fn default() -> Self {
        Self {
            mem: HashMap::default(),
        }
    }
}

pub enum Command {
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

fn actor_loop(rx: mpsc::Receiver<Command>, shard: &mut BBStoreBackend<String, String>) {
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

        for cmd in batch {
            match cmd {
                Command::Write { key, value, ack } => {
                    shard.mem.insert(key, value);
                    let _ = ack.send(());
                }
                Command::Read { key, reply } => {
                    let value = shard.mem.get(&key).cloned();
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn spawn_shard() -> mpsc::Sender<Command> {
        let (tx, rx) = mpsc::channel::<Command>();
        thread::spawn(move || {
            actor_loop(rx, &mut BBStoreBackend::default());
        });
        tx
    }

    #[test]
    fn test_write_then_read() {
        let tx = spawn_shard();

        // Write
        let (ack_tx, ack_rx) = oneshot::channel();
        tx.send(Command::Write {
            key: "name".into(),
            value: "alice".into(),
            ack: ack_tx,
        })
        .unwrap();
        ack_rx.recv().unwrap(); // wait for the actor to process it

        // Read
        let (reply_tx, reply_rx) = oneshot::channel();
        tx.send(Command::Read {
            key: "name".into(),
            reply: reply_tx,
        })
        .unwrap();

        assert_eq!(reply_rx.recv().unwrap(), Some("alice".into()));
    }

    #[test]
    fn test_missing_key_returns_none() {
        let tx = spawn_shard();

        let (reply_tx, reply_rx) = oneshot::channel();
        tx.send(Command::Read {
            key: "ghost".into(),
            reply: reply_tx,
        })
        .unwrap();

        assert_eq!(reply_rx.recv().unwrap(), None);
    }

    #[test]
    fn test_overwrite() {
        let tx = spawn_shard();

        for value in ["alice", "bob", "carol"] {
            let (ack_tx, ack_rx) = oneshot::channel();
            tx.send(Command::Write {
                key: "name".into(),
                value: value.into(),
                ack: ack_tx,
            })
            .unwrap();
            ack_rx.recv().unwrap();
        }

        let (reply_tx, reply_rx) = oneshot::channel();
        tx.send(Command::Read {
            key: "name".into(),
            reply: reply_tx,
        })
        .unwrap();

        assert_eq!(reply_rx.recv().unwrap(), Some("carol".into()));
    }

    #[test]
    fn test_batch_of_writes() {
        let tx = spawn_shard();

        // Fire all writes without waiting for acks — this exercises natural batching
        let acks: Vec<_> = (0..100)
            .map(|i| {
                let (ack_tx, ack_rx) = oneshot::channel();
                tx.send(Command::Write {
                    key: format!("key_{i}"),
                    value: format!("val_{i}"),
                    ack: ack_tx,
                })
                .unwrap();
                ack_rx
            })
            .collect();

        // Now wait for all acks
        for ack in acks {
            ack.recv().unwrap();
        }

        // Spot check
        let (reply_tx, reply_rx) = oneshot::channel();
        tx.send(Command::Read {
            key: "key_42".into(),
            reply: reply_tx,
        })
        .unwrap();
        assert_eq!(reply_rx.recv().unwrap(), Some("val_42".into()));
    }
}
