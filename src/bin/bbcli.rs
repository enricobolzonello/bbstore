use anyhow::Result;
use bbstore::{BBStoreConfig, Command, DEFAULT_ADDRESS, DEFAULT_CONFIG_FILEPATH, DEFAULT_PORT};
use clap::Parser;
use std::io::Read;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,

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
    writer
        .write_all(format!("{}\n", args.command).as_bytes())
        .await?;

    let mut response = String::new();
    BufReader::new(writer).read_line(&mut response).await?;

    print!("{}", response);

    Ok(())
}
