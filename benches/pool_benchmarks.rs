/// Performance Benchmarks using Criterion

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use solana_arbitrage_bot::pool::ObjectPool;
use solana_arbitrage_bot::metrics::BotMetrics;
use solana_arbitrage_bot::error::BotError;

fn bench_object_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("object_pool");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let pool = ObjectPool::new(|| Vec::<u8>::with_capacity(1024), size);
            
            b.iter(|| {
                let _obj = pool.acquire();
                // Object automatically returned on drop
            });
        });
    }
    
    group.finish();
}

fn bench_metrics_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("metrics");
    
    group.bench_function("increment_counter", |b| {
        let metrics = BotMetrics::new();
        b.iter(|| {
            metrics.inc_rpc_request();
        });
    });
    
    group.bench_function("snapshot", |b| {
        let metrics = BotMetrics::new();
        metrics.inc_rpc_request();
        metrics.inc_cache_hit();
        
        b.iter(|| {
            black_box(metrics.snapshot());
        });
    });
    
    group.finish();
}

fn bench_error_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("errors");
    
    group.bench_function("create_rpc_error", |b| {
        b.iter(|| {
            black_box(BotError::rpc_retryable(
                "test-endpoint".to_string(),
                "test message".to_string()
            ));
        });
    });
    
    group.bench_function("create_config_error", |b| {
        b.iter(|| {
            black_box(BotError::ConfigError("test error".to_string()));
        });
    });
    
    group.finish();
}

fn bench_error_methods(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_methods");
    
    let err = BotError::rpc_retryable("test".to_string(), "msg".to_string());
    
    group.bench_function("is_retryable", |b| {
        b.iter(|| {
            black_box(err.is_retryable());
        });
    });
    
    group.bench_function("severity", |b| {
        b.iter(|| {
            black_box(err.severity());
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_object_pool,
    bench_metrics_operations,
    bench_error_creation,
    bench_error_methods
);
criterion_main!(benches);
