use bbstore::{BBStore, BBStoreConfig};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::sync::Arc;

fn bench_working_set_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("WorkingSetSize");
    let config = BBStoreConfig {
        num_shards: 4,
        address: "127.0.0.1".into(),
        buffer_size: 10,
    };

    for size in [1_000, 10_000, 100_000, 1_000_000].iter() {
        let store = Arc::new(BBStore::new(config.clone()));

        for i in 0..*size {
            let _ = store.insert(format!("key-{}", i), "value".to_string());
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &s| {
            b.iter(|| {
                // Random access to trigger cache misses
                let key = format!("key-{}", rand::random::<usize>() % s);
                let _ = black_box(store.get(key));
            });
        });
    }
    group.finish();
}

fn bench_batch_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("BatchingImpact");
    let config = BBStoreConfig {
        num_shards: 1,
        address: "127.0.0.1".into(),
        buffer_size: 10,
    };
    let store = Arc::new(BBStore::new(config)); // Single shard to force high load

    // We flood the actor to ensure the batching logic (try_recv) triggers
    group.bench_function("flood_sequential", |b| {
        b.iter(|| {
            for i in 0..64 {
                let _ = black_box(store.get(format!("key-{}", i)));
            }
        });
    });
    group.finish();
}

criterion_group!(benches, bench_working_set_size, bench_batch_impact);
criterion_main!(benches);
