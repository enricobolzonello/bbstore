use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info};

use crate::types::ByteString;

mod backend;
mod client;
mod command;
mod errors;
mod resp;
mod types;
#[cfg(feature = "benchmarking")]
pub use crate::backend::BBStore;
#[cfg(not(feature = "benchmarking"))]
use crate::backend::BBStore;
pub use crate::client::Client;
pub use crate::command::Command;
pub use crate::resp::Decoder;
pub use crate::resp::Value;

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

async fn process_command(cmd: Command, store: &Arc<BBStore>) -> Result<Value> {
    Ok(match cmd {
        Command::Get { key } => match store.get(key.into()).await? {
            Some(value) => Value::BulkString(value),
            None => Value::Null,
        },
        Command::Set { key, value } => {
            store.insert(key.into(), value.into()).await?;
            Value::String("ok".into())
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
            Value::String("ok".into())
        }
    })
}

pub async fn run(listener: TcpListener, config: BBStoreConfig) -> Result<()> {
    info!(address = %config.address, "server listening");
    debug!(config = ?config);
    let store = Arc::new(BBStore::new(config));
    loop {
        let (stream, addr) = listener.accept().await?;
        let store = store.clone();
        tokio::spawn(handle_connection(stream, addr, store));
    }
}

#[tracing::instrument(skip(stream, store), fields(peer = %addr))]
async fn handle_connection(
    stream: TcpStream,
    addr: std::net::SocketAddr,
    store: Arc<BBStore>,
) -> Result<()> {
    info!("connected");
    let (read_half, write_half) = stream.into_split();
    let mut writer = BufWriter::new(write_half);
    let mut decoder = Decoder::with_buf_bulk(read_half);

    loop {
        let value = match decoder.decode().await {
            Ok(v) => v,
            Err(e) => {
                // TODO: instead of just breaking here i can handle graceful shutdown
                error!("{}", e);
                break;
            }
        };

        info!(cmd = ?value, "received");
        let response = match Command::try_from(value) {
            Ok(command) => match process_command(command, &store).await {
                Ok(r) => r,
                Err(e) => {
                    error!(error = %e, "internal error");
                    return Err(e);
                }
            },
            Err(e) => {
                debug!(error = %e, "protocol error");
                Value::Error(e.to_string().into())
            }
        };
        writer.write_all(response.encode().as_bytes()).await?;
        writer.flush().await?;
    }

    info!("disconnected");
    Ok(())
}
