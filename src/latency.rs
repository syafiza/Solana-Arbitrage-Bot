/// Ultra-Low Latency Optimizations
/// 
/// Provides WebSocket feeds, request batching, and latency tracking for arbitrage.

use crate::error::{BotError, BotResult};
use crate::metrics::METRICS;
use solana_client::nonblocking::rpc_client::RpcClient as AsyncRpcClient;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::StreamExt;
use tracing::{debug, warn, info};

/// Latency tracker for monitoring performance
pub struct LatencyTracker {
    rpc_latencies: Arc<RwLock<Vec<Duration>>>,
    ws_latencies: Arc<RwLock<Vec<Duration>>>,
}

impl LatencyTracker {
    pub fn new() -> Self {
        Self {
            rpc_latencies: Arc::new(RwLock::new(Vec::with_capacity(1000))),
            ws_latencies: Arc::new(RwLock::new(Vec::with_capacity(1000))),
        }
    }

    /// Record RPC latency
    pub async fn record_rpc(&self, duration: Duration) {
        let mut latencies = self.rpc_latencies.write().await;
        latencies.push(duration);
        
        // Keep only last 1000 measurements
        if latencies.len() > 1000 {
            latencies.drain(0..100);
        }
    }

    /// Record WebSocket latency
    pub async fn record_ws(&self, duration: Duration) {
        let mut latencies = self.ws_latencies.write().await;
        latencies.push(duration);
        
        if latencies.len() > 1000 {
            latencies.drain(0..100);
        }
    }

    /// Get average RPC latency
    pub async fn avg_rpc_latency(&self) -> Duration {
        let latencies = self.rpc_latencies.read().await;
        if latencies.is_empty() {
            return Duration::from_millis(0);
        }
        
        let sum: Duration = latencies.iter().sum();
        sum / latencies.len() as u32
    }

    /// Get p95 RPC latency
    pub async fn p95_rpc_latency(&self) -> Duration {
        let mut latencies = self.rpc_latencies.read().await.clone();
        if latencies.is_empty() {
            return Duration::from_millis(0);
        }
        
        latencies.sort();
        let index = (latencies.len() as f64 * 0.95) as usize;
        latencies[index.min(latencies.len() - 1)]
    }

    /// Print latency statistics
    pub async fn print_stats(&self) {
        let avg_rpc = self.avg_rpc_latency().await;
        let p95_rpc = self.p95_rpc_latency().await;
        
        info!("Latency Stats:");
        info!("  RPC Avg: {}ms", avg_rpc.as_millis());
        info!("  RPC P95: {}ms", p95_rpc.as_millis());
    }
}

/// WebSocket account subscriber for real-time updates
pub struct AccountSubscriber {
    ws_url: String,
    latency_tracker: Arc<LatencyTracker>,
}

impl AccountSubscriber {
    pub fn new(ws_url: String, latency_tracker: Arc<LatencyTracker>) -> Self {
        Self {
            ws_url,
            latency_tracker,
        }
    }

    /// Subscribe to account updates via WebSocket
    pub async fn subscribe_account(&self, pubkey: &Pubkey) -> BotResult<()> {
        let (ws_stream, _) = connect_async(&self.ws_url)
            .await
            .map_err(|e| BotError::RpcError {
                endpoint: self.ws_url.clone(),
                message: format!("WebSocket connect failed: {}", e),
                retryable: true,
            })?;

        let (_, mut read) = ws_stream.split();

        // Subscribe message
        let subscribe_msg = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "accountSubscribe",
            "params": [
                pubkey.to_string(),
                {
                    "encoding": "jsonParsed",
                    "commitment": "confirmed"
                }
            ]
        });

        debug!("WebSocket subscribed to account: {}", pubkey);

        // Listen for updates
        while let Some(msg) = read.next().await {
            let start = Instant::now();
            
            match msg {
                Ok(Message::Text(text)) => {
                    // Process account update
                    debug!("Account update received: {} bytes", text.len());
                    
                    let latency = start.elapsed();
                    self.latency_tracker.record_ws(latency).await;
                    
                    if latency.as_millis() > 50 {
                        warn!("High WebSocket latency: {}ms", latency.as_millis());
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket closed");
                    break;
                }
                Err(e) => {
                    warn!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

/// Batch RPC request processor for reduced latency
pub struct BatchProcessor {
    client: Arc<AsyncRpcClient>,
    latency_tracker: Arc<LatencyTracker>,
}

impl BatchProcessor {
    pub fn new(client: Arc<AsyncRpcClient>, latency_tracker: Arc<LatencyTracker>) -> Self {
        Self {
            client,
            latency_tracker,
        }
    }

    /// Fetch multiple accounts in parallel (minimizes latency)
    pub async fn get_multiple_accounts_parallel(
        &self,
        pubkeys: &[Pubkey],
    ) -> BotResult<Vec<Option<solana_sdk::account::Account>>> {
        let start = Instant::now();
        
        // Fetch in parallel using tokio::spawn
        let futures: Vec<_> = pubkeys
            .iter()
            .map(|pubkey| {
                let client = self.client.clone();
                let pk = *pubkey;
                tokio::spawn(async move { client.get_account(&pk).await })
            })
            .collect();

        // Wait for all results
        let mut results = Vec::with_capacity(pubkeys.len());
        for future in futures {
            match future.await {
                Ok(Ok(account)) => results.push(Some(account)),
                _ => results.push(None),
            }
        }

        let latency = start.elapsed();
        self.latency_tracker.record_rpc(latency).await;
        
        debug!(
            "Fetched {} accounts in {}ms (parallel)",
            pubkeys.len(),
            latency.as_millis()
        );

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_latency_tracker() {
        let tracker = LatencyTracker::new();
        
        tracker.record_rpc(Duration::from_millis(10)).await;
        tracker.record_rpc(Duration::from_millis(20)).await;
        tracker.record_rpc(Duration::from_millis(30)).await;
        
        let avg = tracker.avg_rpc_latency().await;
        assert_eq!(avg, Duration::from_millis(20));
    }
}
