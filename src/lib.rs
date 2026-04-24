#![feature(oneshot_channel)]
use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::str::FromStr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;

mod backend;
mod command;
pub use crate::backend::BBStore;
pub use crate::command::Command;

pub const DEFAULT_CONFIG_FILEPATH: &str = "/usr/local/etc/bbstore/bbstore.conf";
pub const DEFAULT_ADDRESS: &str = "127.0.0.1";
pub const DEFAULT_PORT: usize = 8080;
pub const DEFAULT_NUM_SHARDS: usize = 4;

#[derive(Serialize, Deserialize, Clone)]
pub struct BBStoreConfig {
    pub address: String,
    pub num_shards: usize,
}

pub async fn handle_connection(mut stream: TcpStream, store: Arc<BBStore>) -> Result<()> {
    let (read_half, write_half) = stream.split();
    let mut writer = BufWriter::new(write_half);
    let reader = BufReader::new(read_half);

    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        debug!("Received {}", line);
        let response: String = match Command::from_str(&line)? {
            Command::Get { key } => match store.get(key)? {
                Some(value) => format!("{}\n", value),
                None => "nil\n".to_string(),
            },
            Command::Set { key, value } => {
                store.insert(key, value)?;
                "ok\n".to_string()
            }
            Command::Config(_subcommand) => {
                // for now i can assume that the command
                // will be REWRITE (since i handle it in parsing)
                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(DEFAULT_CONFIG_FILEPATH)?;
                file.write(toml::to_string_pretty(&store.config())?.as_bytes())?;
                "ok\n".to_string()
            }
        };
        writer.write_all(response.as_bytes()).await?;
        writer.flush().await?;
    }

    Ok(())
}
