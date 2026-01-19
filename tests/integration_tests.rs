/// Comprehensive Integration Tests for DEX Pool Initializers

#[cfg(test)]
mod dex_initializer_integration_tests {
    use solana_arbitrage_bot::dex::traits::{DexPool, PoolInitializer};
    use solana_arbitrage_bot::dex::raydium::{RaydiumCpmmInitializer, raydium_program_id};
    use solana_arbitrage_bot::dex::pump::PumpInitializer;
    use solana_arbitrage_bot::error::BotError;
    use solana_sdk::pubkey::Pubkey;
    use std::sync::Arc;

    #[test]
    fn test_pool_initializer_validates_addresses() {
        let initializer = RaydiumCpmmInitializer::new();
        
        // Invalid address should fail
        let invalid_addresses = vec!["not-a-pubkey".to_string()];
        let result = initializer.validate_addresses(&invalid_addresses);
        assert!(result.is_err());
        
        // Valid addresses should succeed
        let valid_addresses = vec![Pubkey::new_unique().to_string()];
        let result = initializer.validate_addresses(&valid_addresses);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dex_pool_trait_methods() {
        use solana_arbitrage_bot::dex::raydium::RaydiumCpmmPool;
        
        let pool = RaydiumCpmmPool {
            pool: Pubkey::new_unique(),
            token_vault: Pubkey::new_unique(),
            sol_vault: Pubkey::new_unique(),
            coin_mint: Pubkey::new_unique(),
            pc_mint: Pubkey::new_unique(),
        };

        assert_eq!(pool.dex_name(), "Raydium CPMM");
        assert!(pool.contains_mint(&pool.coin_mint));
        assert!(pool.contains_mint(&pool.pc_mint));
        assert!(!pool.contains_mint(&Pubkey::new_unique()));
    }

    #[test]
    fn test_multiple_dex_initializers() {
        let raydium_init = RaydiumCpmmInitializer::new();
        let pump_init = PumpInitializer::new();
        
        assert_eq!(raydium_init.dex_name(), "Raydium CPMM");
        assert_eq!(pump_init.dex_name(), "Pump.fun");
    }
}

#[cfg(test)]
mod error_handling_tests {
    use solana_arbitrage_bot::error::{BotError, ErrorSeverity};
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn test_error_retryability() {
        let retryable = BotError::rpc_retryable(
            "test-endpoint".to_string(),
            "timeout".to_string()
        );
        assert!(retryable.is_retryable());

        let fatal = BotError::ConfigError("test".to_string());
        assert!(!fatal.is_retryable());
    }

    #[test]
    fn test_error_severity_levels() {
        let config_err = BotError::ConfigError("test".to_string());
        assert_eq!(config_err.severity(), ErrorSeverity::Critical);

        let rpc_err = BotError::rpc_retryable("test".to_string(), "msg".to_string());
        assert_eq!(rpc_err.severity(), ErrorSeverity::Warning);
    }

    #[test]
    fn test_error_display() {
        let err = BotError::InvalidPublicKey {
            key: "invalid-key".to_string(),
            source: solana_sdk::pubkey::ParsePubkeyError::Invalid,
        };
        
        let error_str = format!("{}", err);
        assert!(error_str.contains("invalid-key"));
    }
}

#[cfg(test)]
mod rpc_pool_tests {
    use solana_arbitrage_bot::rpc::RpcPool;

    #[test]
    fn test_rpc_pool_creation() {
        let urls = vec![
            "https://api.mainnet-beta.solana.com".to_string(),
            "https://api.devnet.solana.com".to_string(),
        ];
        
        let pool = RpcPool::new(urls, 60);
        // Pool should be created successfully
        assert_eq!(pool.get_cache_stats().0, 0); // No cache entries initially
    }

    #[test]
    fn test_cache_operations() {
        let urls = vec!["https://api.mainnet-beta.solana.com".to_string()];
        let pool = RpcPool::new(urls, 60);
        
        pool.clear_cache();
        let (valid, expired) = pool.get_cache_stats();
        assert_eq!(valid, 0);
        assert_eq!(expired, 0);
    }
}

#[cfg(test)]
mod metrics_tests {
    use solana_arbitrage_bot::metrics::{BotMetrics, METRICS};

    #[test]
    fn test_metrics_increment() {
        let metrics = BotMetrics::new();
        
        metrics.inc_rpc_request();
        metrics.inc_rpc_request();
        metrics.inc_tx_sent();
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.rpc_requests_total, 2);
        assert_eq!(snapshot.transactions_sent, 1);
    }

    #[test]
    fn test_cache_hit_rate_calculation() {
        let metrics = BotMetrics::new();
        
        metrics.inc_cache_hit();
        metrics.inc_cache_hit();
        metrics.inc_cache_hit();
        metrics.inc_cache_miss();
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.cache_hit_rate(), 75.0);
    }

    #[test]
    fn test_global_metrics_singleton() {
        // Test that METRICS global works
        METRICS.inc_opportunity_found();
        let snapshot = METRICS.snapshot();
        assert!(snapshot.opportunities_found >= 1);
    }
}

#[cfg(test)]
mod object_pool_tests {
    use solana_arbitrage_bot::pool::ObjectPool;

    #[test]
    fn test_object_pool_reuse() {
        let pool = ObjectPool::new(|| Vec::<u8>::with_capacity(1024), 3);
        
        {
            let mut obj1 = pool.acquire();
            obj1.push(42);
            assert_eq!(obj1.len(), 1);
        }
        
        // Object should be returned to pool
        assert_eq!(pool.size(), 3);
    }

    #[test]
    fn test_object_pool_creates_new_when_empty() {
        let pool = ObjectPool::new(|| Vec::<u8>::new(), 1);
        
        let _obj1 = pool.acquire();
        let _obj2 = pool.acquire(); // Should create new since pool empty
        
        assert_eq!(pool.size(), 0);
    }
}
