use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use bbstore::{BBStore, BBStoreConfig};
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use tokio::runtime::Runtime;
use tokio::task::JoinSet;

const OPS_PER_TASK: usize = 20;

fn bench_throughput_scaling_clients(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("ScalingClients");

    let keys: Arc<Vec<String>> = Arc::new((0..100).map(|i| format!("key-{}", i)).collect());

    let store = {
        let _guard = rt.enter();
        Arc::new(BBStore::new(BBStoreConfig {
            num_shards: 4,
            address: "127.0.0.1".into(),
            buffer_size: 64,
        }))
    };

    rt.block_on(async {
        for i in 0..100 {
            let _ = store.insert(format!("key-{}", i), "value".to_string()).await;
        }
    });

    for &concurrency in &[1usize, 4, 16, 64] {
        group.throughput(Throughput::Elements((concurrency * OPS_PER_TASK) as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrency),
            &concurrency,
            |b, &clients| {
                b.to_async(&rt).iter_custom(|iters| {
                    let store = store.clone();
                    let keys = keys.clone();
                    async move {
                        let mut total = Duration::ZERO;
                        for _ in 0..iters {
                            let mut set = JoinSet::new();
                            for i in 0..clients {
                                let store = store.clone();
                                let keys = keys.clone();
                                set.spawn(async move {
                                    for j in 0..OPS_PER_TASK {
                                        let _ = black_box(
                                            store.get(keys[(i + j) % 100].clone()).await,
                                        );
                                    }
                                });
                            }
                            let start = Instant::now();
                            while set.join_next().await.is_some() {}
                            total += start.elapsed();
                        }
                        total
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_throughput_scaling_shards(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("ScalingShards");

    let keys: Arc<Vec<String>> = Arc::new((0..100).map(|i| format!("key-{}", i)).collect());

    const CONCURRENCY: usize = 64;

    for &num_shards in &[1usize, 2, 4, 8] {
        let store = {
            let _guard = rt.enter();
            Arc::new(BBStore::new(BBStoreConfig {
                num_shards,
                address: "127.0.0.1".into(),
                buffer_size: 64,
            }))
        };

        rt.block_on(async {
            for i in 0..100 {
                let _ = store.insert(format!("key-{}", i), "value".to_string()).await;
            }
        });

        group.throughput(Throughput::Elements((CONCURRENCY * OPS_PER_TASK) as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_shards),
            &num_shards,
            |b, _| {
                b.to_async(&rt).iter_custom(|iters| {
                    let store = store.clone();
                    let keys = keys.clone();
                    async move {
                        let mut total = Duration::ZERO;
                        for _ in 0..iters {
                            let mut set = JoinSet::new();
                            for i in 0..CONCURRENCY {
                                let store = store.clone();
                                let keys = keys.clone();
                                set.spawn(async move {
                                    for j in 0..OPS_PER_TASK {
                                        let _ = black_box(
                                            store.get(keys[(i + j) % 100].clone()).await,
                                        );
                                    }
                                });
                            }
                            let start = Instant::now();
                            while set.join_next().await.is_some() {}
                            total += start.elapsed();
                        }
                        total
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_throughput_scaling_clients,
    bench_throughput_scaling_shards
);
criterion_main!(benches);
