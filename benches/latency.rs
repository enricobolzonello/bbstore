use bbstore::{BBStore, BBStoreConfig};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn bench_working_set_size(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("WorkingSetSize");
    let config = BBStoreConfig {
        num_shards: 4,
        address: "127.0.0.1".into(),
        buffer_size: 10,
    };

    for size in [1_000, 10_000, 100_000, 1_000_000].iter() {
        let store = {
            let _guard = rt.enter();
            Arc::new(BBStore::new(config.clone()))
        };

        rt.block_on(async {
            for i in 0..*size {
                let _ = store
                    .insert(format!("key-{}", i), "value".to_string())
                    .await;
            }
        });

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &s| {
            b.to_async(&rt).iter(|| async {
                let key = format!("key-{}", rand::random::<usize>() % s);
                let _ = black_box(store.get(key).await);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_working_set_size);
criterion_main!(benches);
