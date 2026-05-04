use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info};

use crate::types::{ByteStr, ByteString};

mod backend;
mod client;
mod command;
mod errors;
mod types;
#[cfg(feature = "benchmarking")]
pub use crate::backend::BBStore;
#[cfg(not(feature = "benchmarking"))]
use crate::backend::BBStore;
pub use crate::client::Client;
pub use crate::command::Command;

pub const DEFAULT_CONFIG_FILEPATH: &str = "/usr/local/etc/bbstore/bbstore.conf";
pub const DEFAULT_ADDRESS: &str = "127.0.0.1";
pub const DEFAULT_PORT: usize = 8080;
pub const DEFAULT_NUM_SHARDS: usize = 4;
pub const DEFAULT_BUFFER_SIZE: usize = 10;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BBStoreConfig {
    pub address: String,
    pub num_shards: usize,
    pub buffer_size: usize,
}

async fn process_command(cmd: Command, store: &Arc<BBStore>) -> Result<ByteString> {
    let rtn = match cmd {
        Command::Get { key } => match store.get(key.into()).await? {
            Some(value) => value,
            None => "nil".into(),
        },
        Command::Set { key, value } => {
            store.insert(key.into(), value.into()).await?;
            "ok".into()
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
            "ok".into()
        }
    };

    Ok(rtn)
}

pub async fn run(listener: TcpListener, config: BBStoreConfig) -> Result<()> {
    info!(address = %config.address, "server listening");
    let store = Arc::new(BBStore::new(config));
    loop {
        let (stream, addr) = listener.accept().await?;
        let store = store.clone();
        tokio::spawn(handle_connection(stream, addr, store));
    }
}

#[tracing::instrument(skip(stream, store), fields(peer = %addr))]
async fn handle_connection(
    mut stream: TcpStream,
    addr: std::net::SocketAddr,
    store: Arc<BBStore>,
) -> Result<()> {
    info!("connected");
    let (read_half, write_half) = stream.split();
    let mut writer = BufWriter::new(write_half);
    let reader = BufReader::new(read_half);

    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        info!(cmd = %line, "received");
        let response = match Command::from_str(&line) {
            Ok(command) => match process_command(command, &store).await {
                Ok(r) => r,
                Err(e) => {
                    error!(error = %e, "internal error");
                    return Err(e);
                }
            },
            Err(e) => {
                debug!(error = %e, "protocol error");
                format!("ERR: {}", e).into()
            }
        };
        writer.write_all(response.as_bytes()).await?;
        writer.write_all(b"\r\n").await?;
        writer.flush().await?;
    }

    info!("disconnected");
    Ok(())
}
