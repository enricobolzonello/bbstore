use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{TcpListener, TcpStream};

mod backend;
mod client;
mod command;
pub use crate::client::Client;
pub use crate::command::Command;

use crate::backend::BBStore;

pub const DEFAULT_CONFIG_FILEPATH: &str = "/usr/local/etc/bbstore/bbstore.conf";
pub const DEFAULT_ADDRESS: &str = "127.0.0.1";
pub const DEFAULT_PORT: usize = 8080;
pub const DEFAULT_NUM_SHARDS: usize = 4;
pub const DEFAULT_BUFFER_SIZE: usize = 10;

#[derive(Serialize, Deserialize, Clone)]
pub struct BBStoreConfig {
    pub address: String,
    pub num_shards: usize,
    pub buffer_size: usize,
}

async fn process_command(cmd: Command, store: &Arc<BBStore>) -> Result<String> {
    let rtn = match cmd {
        Command::Get { key } => match store.get(key).await? {
            Some(value) => format!("{}\n", value),
            None => "nil\n".to_string(),
        },
        Command::Set { key, value } => {
            store.insert(key, value).await?;
            "ok\n".to_string()
        }
        Command::Config(_subcommand) => {
            // for now i can assume that the command
            // will be REWRITE (since i handle it in parsing)
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(DEFAULT_CONFIG_FILEPATH)
                .await?;
            file.write(toml::to_string_pretty(&store.config())?.as_bytes())
                .await?;
            "ok\n".to_string()
        }
    };

    Ok(rtn)
}

pub async fn run(listener: TcpListener, config: BBStoreConfig) -> Result<()> {
    let store = Arc::new(BBStore::new(config));
    loop {
        let (stream, addr) = listener.accept().await?;
        debug!("Received connection from {}", addr.ip());
        let store = store.clone();
        tokio::spawn(handle_connection(stream, store));
    }
}

async fn handle_connection(mut stream: TcpStream, store: Arc<BBStore>) -> Result<()> {
    let (read_half, write_half) = stream.split();
    let mut writer = BufWriter::new(write_half);
    let reader = BufReader::new(read_half);

    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        debug!("Received {}", line);
        let response: String = match process_command(Command::from_str(&line)?, &store).await {
            Ok(r) => r,
            Err(e) => format!("ERR {}\n", e), // TODO: probably i should split between non-recoverable errors and user errors
        };
        writer.write_all(response.as_bytes()).await?;
        writer.flush().await?;
    }

    Ok(())
}
