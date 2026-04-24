use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::fmt;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[derive(Clone, ValueEnum, Debug)]
enum InputCommands {
    Set,
    Get,
}

impl fmt::Display for InputCommands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Get => write!(f, "GET"),
            Self::Set => write!(f, "SET"),
        }
    }
}

#[derive(Parser)]
struct Args {
    #[arg(required = true, value_enum, ignore_case = true)]
    command: InputCommands,

    #[arg(required = true)]
    key: String,

    #[arg(required_if_eq("command", "SET"))]
    value: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut writer = TcpStream::connect("127.0.0.1:8080").await?; // TODO: global config file to set the same ip address as the store
    let value_part = args.value.map(|v| format!(" {}", v)).unwrap_or_default();
    let command = format!("{} {}{}\n", args.command, args.key, value_part);
    writer.write_all(command.as_bytes()).await?;

    let mut response = String::new();
    BufReader::new(writer).read_line(&mut response).await?;

    print!("{}", response);

    Ok(())
}
