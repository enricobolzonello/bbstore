use anyhow::{Result, bail};
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpStream, ToSocketAddrs};

use crate::Command;
use crate::resp::{Decoder, Value};

pub struct Client {
    decoder: Decoder<OwnedReadHalf>,
    writer: OwnedWriteHalf,
}

impl Client {
    pub async fn connect(addr: impl ToSocketAddrs) -> Result<Self> {
        let (read, write) = TcpStream::connect(addr).await?.into_split();
        Ok(Client { decoder: Decoder::new(read), writer: write })
    }

    async fn send_command(&mut self, cmd: Command) -> Result<Value> {
        self.writer.write_all(Value::from(cmd).encode().as_bytes()).await?;
        self.decoder.decode().await
    }

    pub async fn get(&mut self, key: &str) -> Result<Option<String>> {
        match self.send_command(Command::Get { key: key.to_string() }).await? {
            Value::Null => Ok(None),
            Value::BulkString(s) | Value::String(s) => Ok(Some(s.to_string())),
            Value::Error(e) => bail!("{}", e),
            other => bail!("unexpected response: {:?}", other),
        }
    }

    pub async fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match self.send_command(Command::Set {
            key: key.to_string(),
            value: value.to_string(),
        })
        .await?
        {
            Value::String(_) => Ok(()),
            Value::Error(e) => bail!("{}", e),
            other => bail!("unexpected response: {:?}", other),
        }
    }
}
