use anyhow::Result;
use bbstore::{BBStore, BBStoreConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let config = BBStoreConfig {
        address: "127.0.0.1:8080".into(),
        num_shards: 1,
        buffer_size: 10,
    };
    let store = Arc::new(BBStore::new(config));

    for i in 1..=8 {
        store
            .insert(format!("key-{}", i), format!("value-{}", i))
            .await?;
    }

    let handles: Vec<_> = (1..=8)
        .map(|i| {
            let s = Arc::clone(&store);
            tokio::spawn(async move {
                let key = format!("key-{}", i);
                println!("{:?}", s.get(key).await);
            })
        })
        .collect();

    for h in handles {
        h.await?;
    }

    Ok(())
}
