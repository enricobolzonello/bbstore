use anyhow::Result;
use bbstore::{BBStoreConfig, Client};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let config = BBStoreConfig {
        address: "127.0.0.1:8080".into(),
        num_shards: 1,
        buffer_size: 10,
    };

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tokio::spawn(bbstore::run(listener, config));

    let mut client = Client::connect(addr).await?;

    for i in 1..=8 {
        client.set(&format!("key-{}", i), &format!("value-{}", i)).await?;
    }

    for i in 1..=8 {
        println!("{:?}", client.get(&format!("key-{}", i)).await?);
    }

    Ok(())
}
