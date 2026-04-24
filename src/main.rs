use std::sync::Arc;
use tokio::net::TcpListener;

use anyhow::Result;
use bbstore::{BBStore, handle_connection};
use clap::Parser;
use log::info;

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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let listener = TcpListener::bind(format!("{}:{}", args.address, args.port)).await?;
    env_logger::init();

    let store = Arc::new(BBStore::new(args.num_shards));

    loop {
        let (stream, _) = listener.accept().await?;
        info!("Received connection from {}", stream.local_addr()?.ip());
        let store = store.clone();
        tokio::spawn(async move { handle_connection(stream, store).await });
    }
}
