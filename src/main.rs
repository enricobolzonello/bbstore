use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    net::{TcpListener, TcpStream},
    str::FromStr,
    sync::Arc,
    thread,
};

use anyhow::{Result, bail};
use bbstore::BBStore;
use clap::Parser;
use log::{debug, info};

/// BB(BasicBolzo)-Store
/// Simple key-value store to practice single writer principles
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Number of threads (and shards)
    #[arg(long, short)]
    num_shards: usize,

    /// Address where the store will listen
    #[arg(long, short, default_value = "127.0.0.1")]
    address: String,

    /// Port where the store will listen
    #[arg(long, short, default_value_t = 8080)]
    port: usize,
}

enum ClientCommand {
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

fn handle_connection(stream: TcpStream, store: Arc<BBStore>) -> Result<()> {
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

fn main() -> Result<()> {
    let args = Args::parse();
    let listener = TcpListener::bind(format!("{}:{}", args.address, args.port))?;
    env_logger::init();

    let store = Arc::new(BBStore::new(args.num_shards));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                info!("Received connection from {}", stream.local_addr()?.ip());
                let store = store.clone();
                thread::spawn(move || handle_connection(stream, store));
            }
            Err(e) => bail!(e),
        }
    }

    Ok(())
}
