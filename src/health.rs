/// Health Check Endpoint and Graceful Shutdown
/// 
/// Provides HTTP health check endpoint and graceful shutdown handling.

use crate::metrics::METRICS;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal;
use tokio::sync::RwLock;
use tracing::info;
use warp::{Filter, Reply};

/// Health status
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub uptime_seconds: u64,
    pub metrics: HealthMetrics,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthMetrics {
    pub rpc_requests: u64,
    pub rpc_failures: u64,
    pub cache_hit_rate: f64,
    pub transactions_sent: u64,
    pub opportunities_found: u64,
}

/// Shutdown signal handler
pub struct ShutdownHandler {
    should_shutdown: Arc<AtomicBool>,
    start_time: std::time::Instant,
}

impl ShutdownHandler {
    pub fn new() -> Self {
        Self {
            should_shutdown: Arc::new(AtomicBool::new(false)),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn should_shutdown(&self) -> bool {
        self.should_shutdown.load(Ordering::Relaxed)
    }

    pub fn trigger_shutdown(&self) {
        info!("Graceful shutdown initiated");
        self.should_shutdown.store(true, Ordering::Relaxed);
    }

    pub async fn wait_for_shutdown_signal(&self) {
        let shutdown_flag = self.should_shutdown.clone();
        
        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("Received Ctrl+C signal");
                shutdown_flag.store(true, Ordering::Relaxed);
            }
            #[cfg(unix)]
            _ = async {
                use tokio::signal::unix::{signal, SignalKind};
                let mut sigterm = signal(SignalKind::terminate()).unwrap();
                sigterm.recv().await
            } => {
                info!("Received SIGTERM signal");
                shutdown_flag.store(true, Ordering::Relaxed);
            }
        }
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

/// Start health check server
pub async fn start_health_server(
    port: u16,
    shutdown_handler: Arc<ShutdownHandler>,
) -> Result<(), Box<dyn std::error::Error>> {
    let health_route = warp::path("health")
        .and(warp::get())
        .and(with_shutdown(shutdown_handler.clone()))
        .map(|handler: Arc<ShutdownHandler>| {
            let snapshot = METRICS.snapshot();
            
            let status = HealthStatus {
                status: "healthy".to_string(),
                uptime_seconds: handler.uptime_seconds(),
                metrics: HealthMetrics {
                    rpc_requests: snapshot.rpc_requests_total,
                    rpc_failures: snapshot.rpc_failures_total,
                    cache_hit_rate: snapshot.cache_hit_rate(),
                    transactions_sent: snapshot.transactions_sent,
                    opportunities_found: snapshot.opportunities_found,
                },
            };

            warp::reply::json(&status)
        });

    let ready_route = warp::path("ready")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({"ready": true})));

    let metrics_route = warp::path("metrics")
        .and(warp::get())
        .map(|| {
            let snapshot = METRICS.snapshot();
            let metrics_text = format!(
                "# HELP rpc_requests_total Total RPC requests\n\
                 # TYPE rpc_requests_total counter\n\
                 rpc_requests_total {}\n\
                 # HELP rpc_failures_total Total RPC failures\n\
                 # TYPE rpc_failures_total counter\n\
                 rpc_failures_total {}\n\
                 # HELP cache_hit_rate Cache hit rate percentage\n\
                 # TYPE cache_hit_rate gauge\n\
                 cache_hit_rate {}\n\
                 # HELP transactions_sent Total transactions sent\n\
                 # TYPE transactions_sent counter\n\
                 transactions_sent {}\n\
                 # HELP opportunities_found Total opportunities found\n\
                 # TYPE opportunities_found counter\n\
                 opportunities_found {}\n",
                snapshot.rpc_requests_total,
                snapshot.rpc_failures_total,
                snapshot.cache_hit_rate(),
                snapshot.transactions_sent,
                snapshot.opportunities_found,
            );
            
            warp::reply::with_header(metrics_text, "Content-Type", "text/plain")
        });

    let routes = health_route.or(ready_route).or(metrics_route);

    info!("Starting health check server on port {}", port);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;

    Ok(())
}

fn with_shutdown(
    handler: Arc<ShutdownHandler>,
) -> impl Filter<Extract = (Arc<ShutdownHandler>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || handler.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_handler() {
        let handler = ShutdownHandler::new();
        
        assert!(!handler.should_shutdown());
        
        handler.trigger_shutdown();
        
        assert!(handler.should_shutdown());
    }

    #[test]
    fn test_uptime_tracking() {
        let handler = ShutdownHandler::new();
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        assert!(handler.uptime_seconds() >= 0);
    }
}
