#![feature(oneshot_channel)]
use anyhow::{Result, bail};
use log::debug;
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;

mod backend;
pub use crate::backend::BBStore;

pub const DEFAULT_CONFIG_FILEPATH: &str = "/etc/bbstore/bbstore.conf";
pub const DEFAULT_ADDRESS: &str = "127.0.0.1";
pub const DEFAULT_PORT: usize = 8080;
pub const DEFAULT_NUM_SHARDS: usize = 4;

pub enum ClientCommand {
    Get { key: String },
    Insert { key: String, value: String },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BBStoreConfig {
    pub address: String,
    pub num_shards: usize,
}

impl FromStr for ClientCommand {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let words: Vec<&str> = s.splitn(3, ' ').collect();
        match words.as_slice() {
            ["GET", key] => Ok(ClientCommand::Get {
                key: key.to_string(),
            }),
            ["SET", key, value] => Ok(ClientCommand::Insert {
                key: key.to_string(),
                value: value.to_string(),
            }),
            [cmd, ..] => bail!("Unknown command {}", cmd),
            [] => bail!("Empty command"),
        }
    }
}

pub async fn handle_connection(mut stream: TcpStream, store: Arc<BBStore>) -> Result<()> {
    let (read_half, write_half) = stream.split();
    let mut writer = BufWriter::new(write_half);
    let reader = BufReader::new(read_half);

    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        debug!("Received {}", line);
        let response: String = match ClientCommand::from_str(&line)? {
            ClientCommand::Get { key } => match store.get(key)? {
                Some(value) => format!("{}\n", value),
                None => "nil\n".to_string(),
            },
            ClientCommand::Insert { key, value } => {
                store.insert(key, value)?;
                "ok\n".to_string()
            }
        };
        writer.write_all(response.as_bytes()).await?;
        writer.flush().await?;
    }

    Ok(())
}
