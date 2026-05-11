use anyhow::Result;
use bbstore::{
    BBStoreConfig, Command, DEFAULT_ADDRESS, DEFAULT_CONFIG_FILEPATH, DEFAULT_PORT, Decoder, Value,
};
use clap::Parser;

use tokio::io::AsyncWriteExt;
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();

    let server_address = if let Ok(buf) = tokio::fs::read_to_string(DEFAULT_CONFIG_FILEPATH).await {
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

    let stream = TcpStream::connect(&server_address).await?;
    let (reader, mut writer) = stream.into_split();

    writer
        .write_all(&Value::from(args.command).encode())
        .await?;

    let value = Decoder::new(reader).decode().await?;
    print!("{}", value.to_string_pretty());

    Ok(())
}
