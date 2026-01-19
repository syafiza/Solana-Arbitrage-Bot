/// Property-Based Tests using Proptest
/// 
/// Tests invariants that should hold for all inputs.

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use solana_arbitrage_bot::error::BotError;
    use solana_sdk::pubkey::Pubkey;

    proptest! {
        #[test]
        fn test_pubkey_validation_never_panics(s in "\\PC*") {
            // Should never panic, only return Err
            let _ = Pubkey::try_from(s.as_str());
        }

        #[test]
        fn test_error_severity_is_consistent(
            endpoint in "[a-z]{1,20}",
            message in "[a-z]{1,50}"
        ) {
            let err1 = BotError::rpc_retryable(endpoint.clone(), message.clone());
            let err2 = BotError::rpc_retryable(endpoint, message);
            
            // Same error type should have same severity
            prop_assert_eq!(err1.severity(), err2.severity());
        }

        #[test]
        fn test_retryable_errors_are_consistent(retryable in any::<bool>()) {
            let err = if retryable {
                BotError::rpc_retryable("test".to_string(), "msg".to_string())
            } else {
                BotError::ConfigError("test".to_string())
            };
            
            prop_assert_eq!(err.is_retryable(), retryable);
        }

        #[test]
        fn test_cache_hit_rate_bounds(hits in 0u64..1000, misses in 0u64..1000) {
            use solana_arbitrage_bot::metrics::BotMetrics;
            
            let metrics = BotMetrics::new();
            
            for _ in 0..hits {
                metrics.inc_cache_hit();
            }
            for _ in 0..misses {
                metrics.inc_cache_miss();
            }
            
            let snapshot = metrics.snapshot();
            let rate = snapshot.cache_hit_rate();
            
            // Cache hit rate should always be between 0 and 100
            prop_assert!(rate >= 0.0 && rate <= 100.0);
        }

        #[test]
        fn test_profit_calculation_no_overflow(lamports in 0u64..u64::MAX / 2) {
            use solana_arbitrage_bot::metrics::BotMetrics;
            
            let metrics = BotMetrics::new();
            metrics.add_profit(lamports);
            
            let snapshot = metrics.snapshot();
           prop_assert_eq!(snapshot.total_profit_lamports, lamports);
            
            let sol = snapshot.total_profit_sol();
            prop_assert!(sol >= 0.0);
        }
    }
}

#[cfg(test)]
mod invariant_tests {
    use proptest::prelude::*;
    use solana_arbitrage_bot::pool::ObjectPool;

    proptest! {
        #[test]
        fn test_object_pool_size_invariant(initial_size in 1usize..100) {
            let pool = ObjectPool::new(|| Vec::<u8>::new(), initial_size);
            
            // After creation, size should equal initial_size
            prop_assert_eq!(pool.size(), initial_size);
            
            {
                let _obj = pool.acquire();
                // After acquiring, size should decrease
                prop_assert_eq!(pool.size(), initial_size - 1);
            }
            
            // After drop, size should be restored
            prop_assert_eq!(pool.size(), initial_size);
        }
    }
}
