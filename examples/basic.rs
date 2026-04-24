use anyhow::Result;
use bbstore::{BBStore, BBStoreConfig};
use std::sync::Arc;

fn main() -> Result<()> {
    let config = BBStoreConfig {
        address: "127.0.0.1:8080".into(),
        num_shards: 1,
    };
    let store = Arc::new(BBStore::new(config));
    store.insert("key-1".into(), "value-1".into())?;
    store.insert("key-2".into(), "value-2".into())?;
    store.insert("key-3".into(), "value-3".into())?;
    store.insert("key-4".into(), "value-4".into())?;
    store.insert("key-5".into(), "value-5".into())?;
    store.insert("key-6".into(), "value-6".into())?;
    store.insert("key-7".into(), "value-7".into())?;
    store.insert("key-8".into(), "value-8".into())?;

    std::thread::scope(|s| {
        for i in 1..9 {
            let thread_store = Arc::clone(&store);
            s.spawn(move || {
                let key = format!("key-{}", i);
                println!("{:?}", thread_store.get(key));
            });
        }
    });

    Ok(())
}
