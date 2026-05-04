use anyhow::Result;
use bbstore::{
    BBStoreConfig, DEFAULT_ADDRESS, DEFAULT_BUFFER_SIZE, DEFAULT_CONFIG_FILEPATH,
    DEFAULT_NUM_SHARDS, DEFAULT_PORT,
};
use clap::Parser;
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

/// BB(BasicBolzo)-Store
/// Simple key-value store to practice single writer principles
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Number of threads (and shards)
    #[arg(long, short)]
    num_shards: Option<usize>,

    /// Buffer size of each shard's channel
    #[arg(long, short)]
    buffer_size: Option<usize>,

    /// Address where the store will listen
    #[arg(long, short)]
    address: Option<String>,

    /// Port where the store will listen
    #[arg(long, short)]
    port: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_filter(env_filter);
    tracing_subscriber::registry().with(fmt_layer).init();
    let args = Args::parse();

    let config = if let Ok(buf) = tokio::fs::read_to_string(DEFAULT_CONFIG_FILEPATH).await {
        let file_config: BBStoreConfig = toml::from_str(&buf)?;
        BBStoreConfig {
            address: if args.address.is_some() || args.port.is_some() {
                format!(
                    "{}:{}",
                    args.address.as_deref().unwrap_or(DEFAULT_ADDRESS),
                    args.port.unwrap_or(DEFAULT_PORT)
                )
            } else {
                file_config.address
            },
            num_shards: args.num_shards.unwrap_or(file_config.num_shards),
            buffer_size: args.buffer_size.unwrap_or(DEFAULT_BUFFER_SIZE),
        }
    } else {
        BBStoreConfig {
            address: format!(
                "{}:{}",
                args.address.as_deref().unwrap_or(DEFAULT_ADDRESS),
                args.port.unwrap_or(DEFAULT_PORT)
            ),
            num_shards: args.num_shards.unwrap_or(DEFAULT_NUM_SHARDS),
            buffer_size: args.buffer_size.unwrap_or(DEFAULT_BUFFER_SIZE),
        }
    };

    let listener = TcpListener::bind(&config.address).await?;
    bbstore::run(listener, config).await
}
