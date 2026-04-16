#![feature(oneshot_channel)]
use anyhow::{Result, bail};
use log::debug;
use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    net::TcpStream,
    str::FromStr,
    sync::Arc,
};

mod backend;
pub use crate::backend::BBStore;

pub enum ClientCommand {
    Get { key: String },
    Insert { key: String, value: String },
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

pub fn handle_connection(stream: TcpStream, store: Arc<BBStore>) -> Result<()> {
    let mut writer = BufWriter::new(&stream);
    let reader = BufReader::new(&stream);

    for line in reader.lines() {
        let line = line?;
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
        writer.write_all(response.as_bytes())?;
        writer.flush()?;
    }

    Ok(())
}
