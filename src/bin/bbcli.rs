use anyhow::Result;
use bbstore::{BBStoreConfig, DEFAULT_ADDRESS, DEFAULT_CONFIG_FILEPATH, DEFAULT_PORT};
use clap::{Parser, Subcommand};
use std::io::Read;

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

    #[arg(long, short)]
    address: Option<String>,

    #[arg(long, short)]
    port: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let server_address = if let Ok(mut file) = std::fs::File::open(DEFAULT_CONFIG_FILEPATH) {
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        let file_config: BBStoreConfig = toml::from_str(&buf)?;
        if args.address.is_some() || args.port.is_some() {
            format!(
                "{}:{}",
                args.address.as_deref().unwrap_or(DEFAULT_ADDRESS),
                args.port.unwrap_or(DEFAULT_PORT)
            )
        } else {
            file_config.address
        }
    } else {
        format!(
            "{}:{}",
            args.address.as_deref().unwrap_or(DEFAULT_ADDRESS),
            args.port.unwrap_or(DEFAULT_PORT)
        )
    };

    let mut writer = TcpStream::connect(&server_address).await?;
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
