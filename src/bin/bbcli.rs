use anyhow::Result;
use clap::{Parser, Subcommand};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[derive(Subcommand, Clone, Debug)]
enum InputCommands {
    #[command(name = "SET")]
    Set { key: String, value: String },
    #[command(name = "GET")]
    Get { key: String },
}

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: InputCommands,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut writer = TcpStream::connect("127.0.0.1:8080").await?; // TODO: global config file to set the same ip address as the store
    let command = match &args.command {
        InputCommands::Set { key, value } => format!("SET {} {}\n", key, value),
        InputCommands::Get { key } => format!("GET {}\n", key),
    };
    writer.write_all(command.as_bytes()).await?;

    let mut response = String::new();
    BufReader::new(writer).read_line(&mut response).await?;

    print!("{}", response);

    Ok(())
}
