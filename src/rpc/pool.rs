/// RPC Connection Pool with Circuit Breaker
/// 
/// Provides connection pooling, request caching, and circuit breaker pattern
/// for resilient RPC communication.

use crate::constants::{MAX_RPC_RETRIES, RETRY_INITIAL_BACKOFF_MS, RETRY_MAX_BACKOFF_MS};
use crate::error::{BotError, BotResult};
use solana_client::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, warn};

/// Cached RPC response with TTL
#[derive(Clone)]
struct CachedResponse {
    data: Account,
    expires_at: Instant,
}

/// RPC connection pool with caching and circuit breaker
pub struct RpcPool {
    clients: Vec<Arc<RpcClient>>,
    cache: Arc<RwLock<HashMap<Pubkey, CachedResponse>>>,
    cache_ttl: Duration,
    current_client_index: Arc<RwLock<usize>>,
    circuit_breaker: Arc<RwLock<CircuitBreaker>>,
}

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,  // Normal operation
    Open,    // Failing, reject requests
    HalfOpen, // Testing if service recovered
}

/// Circuit breaker for RPC endpoints
struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    last_failure_time: Option<Instant>,
    failure_threshold: u32,
    timeout: Duration,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, timeout_secs: u64) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            last_failure_time: None,
            failure_threshold,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
            warn!("Circuit breaker OPEN after {} failures", self.failure_count);
        }
    }

    fn can_attempt(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.timeout {
                        debug!("Circuit breaker transitioning to HALF_OPEN");
                        self.state = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }
}

impl RpcPool {
    /// Create a new RPC pool
    pub fn new(urls: Vec<String>, cache_ttl_secs: u64) -> Self {
        let clients = urls
            .into_iter()
            .map(|url| Arc::new(RpcClient::new(url)))
            .collect();

        Self {
            clients,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(cache_ttl_secs),
            current_client_index: Arc::new(RwLock::new(0)),
            circuit_breaker: Arc::new(RwLock::new(CircuitBreaker::new(5, 30))),
        }
    }

    /// Get account with caching and retry logic
    pub async fn get_account_with_retry(&self, pubkey: &Pubkey) -> BotResult<Account> {
        // Check cache first
        if let Some(cached) = self.get_from_cache(pubkey) {
            debug!("Cache hit for account: {}", pubkey);
            return Ok(cached);
        }

        // Check circuit breaker
        {
            let mut cb = self.circuit_breaker.write().unwrap();
            if !cb.can_attempt() {
                return Err(BotError::RpcError {
                    endpoint: "pool".to_string(),
                    message: "Circuit breaker is OPEN".to_string(),
                    retryable: true,
                });
            }
        }

        // Attempt with exponential backoff
        let mut backoff_ms = RETRY_INITIAL_BACKOFF_MS;
        let mut last_error = None;

        for attempt in 0..MAX_RPC_RETRIES {
            match self.attempt_get_account(pubkey).await {
                Ok(account) => {
                    // Success - record in circuit breaker and cache
                    self.circuit_breaker.write().unwrap().record_success();
                    self.add_to_cache(pubkey, &account);
                    return Ok(account);
                }
                Err(e) => {
                    last_error = Some(e);
                    
                    if attempt < MAX_RPC_RETRIES - 1 {
                        debug!("RPC attempt {} failed, retrying in {}ms", attempt + 1, backoff_ms);
                        sleep(Duration::from_millis(backoff_ms)).await;
                        backoff_ms = (backoff_ms * 2).min(RETRY_MAX_BACKOFF_MS);
                    }
                }
            }
        }

        // All attempts failed
        self.circuit_breaker.write().unwrap().record_failure();

        Err(last_error.unwrap_or_else(|| {
            BotError::RpcError {
                endpoint: "pool".to_string(),
                message: "All retry attempts exhausted".to_string(),
                retryable: true,
            }
        }))
    }

    /// Attempt to get account from current client
    async fn attempt_get_account(&self, pubkey: &Pubkey) -> BotResult<Account> {
        let client = self.get_next_client();
        
        client.get_account(pubkey).map_err(|e| {
            BotError::AccountFetchError {
                address: *pubkey,
                reason: format!("RPC error: {}", e),
            }
        })
    }

    /// Get from cache if not expired
    fn get_from_cache(&self, pubkey: &Pubkey) -> Option<Account> {
        let cache = self.cache.read().unwrap();
        
        cache.get(pubkey).and_then(|cached| {
            if cached.expires_at > Instant::now() {
                Some(cached.data.clone())
            } else {
                None
            }
        })
    }

    /// Add to cache with TTL
    fn add_to_cache(&self, pubkey: &Pubkey, account: &Account) {
        let mut cache = self.cache.write().unwrap();
        
        cache.insert(
            *pubkey,
            CachedResponse {
                data: account.clone(),
                expires_at: Instant::now() + self.cache_ttl,
            },
        );
    }

    /// Get next client (round-robin)
    fn get_next_client(&self) -> Arc<RpcClient> {
        let mut index = self.current_client_index.write().unwrap();
        let client = self.clients[*index % self.clients.len()].clone();
        *index = (*index + 1) % self.clients.len();
        client
    }

    /// Clear cache (useful for testing or manual refresh)
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
        debug!("RPC cache cleared");
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read().unwrap();
        let total = cache.len();
        let expired = cache
            .values()
            .filter(|cached| cached.expires_at <= Instant::now())
            .count();
        
        (total - expired, expired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_state_transitions() {
        let mut cb = CircuitBreaker::new(3, 1);

        assert_eq!(cb.state, CircuitState::Closed);
        assert!(cb.can_attempt());

        // Record failures
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state, CircuitState::Closed);

        cb.record_failure(); // Hits threshold
        assert_eq!(cb.state, CircuitState::Open);
        assert!(!cb.can_attempt());

        // Success resets
        cb.record_success();
        assert_eq!(cb.state, CircuitState::Closed);
        assert_eq!(cb.failure_count, 0);
    }

    #[test]
    fn test_circuit_breaker_timeout() {
        let mut cb = CircuitBreaker::new(1, 0); // 0 second timeout for testing

        cb.record_failure();
        assert_eq!(cb.state, CircuitState::Open);

        std::thread::sleep(Duration::from_millis(100));
        
        // After timeout, should transition to HalfOpen
        assert!(cb.can_attempt());
        assert_eq!(cb.state, CircuitState::HalfOpen);
    }
}
