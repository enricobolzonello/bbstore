use anyhow::{Result, bail};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpStream, ToSocketAddrs};

use crate::Command;

pub struct Client {
    reader: BufReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
}

impl Client {
    pub async fn connect(addr: impl ToSocketAddrs) -> Result<Self> {
        let (read, write) = TcpStream::connect(addr).await?.into_split();
        Ok(Client { reader: BufReader::new(read), writer: write })
    }

    async fn send_command(&mut self, cmd: Command) -> Result<String> {
        self.writer.write_all(format!("{}\n", cmd).as_bytes()).await?;

        let mut line = String::new();
        let n = self.reader.read_line(&mut line).await?;
        if n == 0 {
            bail!("connection closed by server");
        }
        if let Some(msg) = line.strip_prefix("ERR ") {
            bail!("{}", msg.trim_end_matches('\n'));
        }

        Ok(line)
    }

    pub async fn get(&mut self, key: &str) -> Result<Option<String>> {
        let response = self.send_command(Command::Get { key: key.to_string() }).await?;
        if response == "nil\n" {
            Ok(None)
        } else {
            Ok(Some(response.trim_end_matches('\n').to_string()))
        }
    }

    pub async fn set(&mut self, key: &str, value: &str) -> Result<()> {
        self.send_command(Command::Set {
            key: key.to_string(),
            value: value.to_string(),
        })
        .await?;
        Ok(())
    }
}
