/// Performance Metrics Collection
/// 
/// Provides prometheus-compatible metrics for monitoring bot performance.

use lazy_static::lazy_static;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

lazy_static! {
    /// Global metrics registry
    pub static ref METRICS: Arc<BotMetrics> = Arc::new(BotMetrics::new());
}

/// Bot performance metrics
pub struct BotMetrics {
    // RPC metrics
    pub rpc_requests_total: AtomicU64,
    pub rpc_failures_total: AtomicU64,
    pub rpc_cache_hits: AtomicU64,
    pub rpc_cache_misses: AtomicU64,
    
    // Pool metrics
    pub pools_initialized_total: AtomicU64,
    pub pool_initialization_failures: AtomicU64,
    
    // Transaction metrics
    pub transactions_sent: AtomicU64,
    pub transactions_confirmed: AtomicU64,
    pub transactions_failed: AtomicU64,
    
    // Arbitrage metrics
    pub opportunities_found: AtomicU64,
    pub opportunities_executed: AtomicU64,
    pub total_profit_lamports: AtomicU64,
}

impl BotMetrics {
    pub fn new() -> Self {
        Self {
            rpc_requests_total: AtomicU64::new(0),
            rpc_failures_total: AtomicU64::new(0),
            rpc_cache_hits: AtomicU64::new(0),
            rpc_cache_misses: AtomicU64::new(0),
            pools_initialized_total: AtomicU64::new(0),
            pool_initialization_failures: AtomicU64::new(0),
            transactions_sent: AtomicU64::new(0),
            transactions_confirmed: AtomicU64::new(0),
            transactions_failed: AtomicU64::new(0),
            opportunities_found: AtomicU64::new(0),
            opportunities_executed: AtomicU64::new(0),
            total_profit_lamports: AtomicU64::new(0),
        }
    }

    // RPC metrics
    pub fn inc_rpc_request(&self) {
        self.rpc_requests_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_rpc_failure(&self) {
        self.rpc_failures_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_cache_hit(&self) {
        self.rpc_cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_cache_miss(&self) {
        self.rpc_cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    // Pool metrics
    pub fn inc_pool_initialized(&self) {
        self.pools_initialized_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_pool_failure(&self) {
        self.pool_initialization_failures.fetch_add(1, Ordering::Relaxed);
    }

    // Transaction metrics
    pub fn inc_tx_sent(&self) {
        self.transactions_sent.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_tx_confirmed(&self) {
        self.transactions_confirmed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_tx_failed(&self) {
        self.transactions_failed.fetch_add(1, Ordering::Relaxed);
    }

    // Arbitrage metrics
    pub fn inc_opportunity_found(&self) {
        self.opportunities_found.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_opportunity_executed(&self) {
        self.opportunities_executed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_profit(&self, lamports: u64) {
        self.total_profit_lamports.fetch_add(lamports, Ordering::Relaxed);
    }

    /// Get metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            rpc_requests_total: self.rpc_requests_total.load(Ordering::Relaxed),
            rpc_failures_total: self.rpc_failures_total.load(Ordering::Relaxed),
            rpc_cache_hits: self.rpc_cache_hits.load(Ordering::Relaxed),
            rpc_cache_misses: self.rpc_cache_misses.load(Ordering::Relaxed),
            pools_initialized_total: self.pools_initialized_total.load(Ordering::Relaxed),
            pool_initialization_failures: self.pool_initialization_failures.load(Ordering::Relaxed),
            transactions_sent: self.transactions_sent.load(Ordering::Relaxed),
            transactions_confirmed: self.transactions_confirmed.load(Ordering::Relaxed),
            transactions_failed: self.transactions_failed.load(Ordering::Relaxed),
            opportunities_found: self.opportunities_found.load(Ordering::Relaxed),
            opportunities_executed: self.opportunities_executed.load(Ordering::Relaxed),
            total_profit_lamports: self.total_profit_lamports.load(Ordering::Relaxed),
        }
    }

    /// Print metrics summary
    pub fn print_summary(&self) {
        let snapshot = self.snapshot();
        println!("\n=== Bot Performance Metrics ===");
        println!("RPC Requests: {}", snapshot.rpc_requests_total);
        println!("RPC Failures: {}", snapshot.rpc_failures_total);
        println!("Cache Hit Rate: {:.2}%", snapshot.cache_hit_rate());
        println!("Pools Initialized: {}", snapshot.pools_initialized_total);
        println!("Transactions Sent: {}", snapshot.transactions_sent);
        println!("Transactions Confirmed: {}", snapshot.transactions_confirmed);
        println!("Success Rate: {:.2}%", snapshot.tx_success_rate());
        println!("Opportunities Found: {}", snapshot.opportunities_found);
        println!("Opportunities Executed: {}", snapshot.opportunities_executed);
        println!("Total Profit: {} SOL", snapshot.total_profit_sol());
        println!("==============================\n");
    }
}

/// Immutable metrics snapshot
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub rpc_requests_total: u64,
    pub rpc_failures_total: u64,
    pub rpc_cache_hits: u64,
    pub rpc_cache_misses: u64,
    pub pools_initialized_total: u64,
    pub pool_initialization_failures: u64,
    pub transactions_sent: u64,
    pub transactions_confirmed: u64,
    pub transactions_failed: u64,
    pub opportunities_found: u64,
    pub opportunities_executed: u64,
    pub total_profit_lamports: u64,
}

impl MetricsSnapshot {
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.rpc_cache_hits + self.rpc_cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.rpc_cache_hits as f64 / total as f64) * 100.0
        }
    }

    pub fn tx_success_rate(&self) -> f64 {
        if self.transactions_sent == 0 {
            0.0
        } else {
            (self.transactions_confirmed as f64 / self.transactions_sent as f64) * 100.0
        }
    }

    pub fn total_profit_sol(&self) -> f64 {
        self.total_profit_lamports as f64 / 1_000_000_000.0
    }
}

/// Performance timer helper
pub struct PerfTimer {
    start: Instant,
    name: String,
}

impl PerfTimer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            name: name.into(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for PerfTimer {
    fn drop(&mut self) {
        let elapsed = self.elapsed();
        if elapsed.as_millis() > 100 {
            tracing::warn!("{} took {}ms (slow!)", self.name, elapsed.as_millis());
        } else {
            tracing::debug!("{} took {}ms", self.name, elapsed.as_millis());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_increment() {
        let metrics = BotMetrics::new();
        
        metrics.inc_rpc_request();
        metrics.inc_rpc_request();
        metrics.inc_cache_hit();
        
        assert_eq!(metrics.rpc_requests_total.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.rpc_cache_hits.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_cache_hit_rate() {
        let metrics = BotMetrics::new();
        
        metrics.inc_cache_hit();
        metrics.inc_cache_hit();
        metrics.inc_cache_hit();
        metrics.inc_cache_miss();
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.cache_hit_rate(), 75.0);
    }

    #[test]
    fn test_tx_success_rate() {
        let metrics = BotMetrics::new();
        
        metrics.inc_tx_sent();
        metrics.inc_tx_sent();
        metrics.inc_tx_sent();
        metrics.inc_tx_sent();
        metrics.inc_tx_confirmed();
        metrics.inc_tx_confirmed();
        metrics.inc_tx_confirmed();
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.tx_success_rate(), 75.0);
    }
}
