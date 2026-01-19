/// DEX abstraction traits for the Solana Arbitrage Bot
/// 
/// This module provides trait-based abstractions for all DEX interactions,
/// enabling uniform handling of different DEX protocols and eliminating code duplication.

use crate::error::{BotError, BotResult};
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

/// Common trait for all DEX pool types
/// 
/// This trait provides a uniform interface for interacting with different DEX protocols.
/// Each DEX implements this trait to provide protocol-specific behavior while
/// maintaining a consistent API across the bot.
#[async_trait]
pub trait DexPool: Send + Sync + std::fmt::Debug {
    /// Initialize pool data from on-chain accounts
    /// 
    /// # Arguments
    /// * `rpc_client` - RPC client for fetching on-chain data
    /// * `pool_address` - The pool's public key
    /// 
    /// # Returns
    /// * `Ok(())` if initialization succeeded
    /// * `Err(BotError)` if initialization failed with specific error context
    async fn initialize(
        &mut self,
        rpc_client: &RpcClient,
        pool_address: &Pubkey,
    ) -> BotResult<()>;

    /// Generate account metadata for swap instructions
    /// 
    /// # Arguments
    /// * `wallet` - The wallet's public key that will execute the swap
    /// 
    /// # Returns
    /// Vector of AccountMeta in the order required by this DEX's swap instruction
    fn get_swap_accounts(&self, wallet: &Pubkey) -> Vec<AccountMeta>;

    /// Get current pool liquidity
    /// 
    /// # Returns
    /// (token_amount, sol_amount) tuple representing current reserves
    fn get_liquidity(&self) -> (u64, u64);

    /// Get the DEX protocol name for logging and metrics
    fn dex_name(&self) -> &'static str;

    /// Get the pool's public key
    fn pool_address(&self) -> Pubkey;

    /// Check if this pool contains the specified mint
    fn contains_mint(&self, mint: &Pubkey) -> bool;
}

/// Trait for initializing multiple pools of the same DEX type
/// 
/// This trait handles batch initialization of pools for a specific DEX protocol.
#[async_trait]
pub trait PoolInitializer: Send + Sync {
    type Pool: DexPool;

    /// Initialize multiple pools from their addresses
    /// 
    /// # Arguments
    /// * `addresses` - List of pool address strings
    /// * `rpc_client` - RPC client for fetching on-chain data
    /// * `mint` - The token mint that should be present in these pools
    /// 
    /// # Returns
    /// * `Ok(Vec<Self::Pool>)` with successfully initialized pools
    /// * `Err(BotError)` if any pool fails to initialize
    async fn initialize_pools(
        &self,
        addresses: &[String],
        rpc_client: Arc<RpcClient>,
        mint: &Pubkey,
    ) -> BotResult<Vec<Self::Pool>>;

    /// Get the name of this DEX for logging
    fn dex_name(&self) -> &'static str;

    /// Validate pool addresses before initialization
    fn validate_addresses(&self, addresses: &[String]) -> BotResult<Vec<Pubkey>> {
        addresses
            .iter()
            .map(|addr| {
                Pubkey::try_from(addr.as_str()).map_err(|e| BotError::InvalidPublicKey {
                    key: addr.clone(),
                    source: e,
                })
            })
            .collect()
    }
}

/// Helper trait for pools that support concentrated liquidity
pub trait ConcentratedLiquidityPool: DexPool {
    /// Get tick array accounts for concentrated liquidity pools
    fn get_tick_arrays(&self) -> &[Pubkey];
    
    /// Get the current tick for this pool
    fn current_tick(&self) -> i32;
}

/// Helper trait for pools that use oracles
pub trait OracleBasedPool: DexPool {
    /// Get the oracle account for this pool
    fn oracle_account(&self) -> Pubkey;
}

/// Common pool validation logic
pub struct PoolValidator;

impl PoolValidator {
    /// Validate that a pool contains both the specified mint and SOL
    pub fn validate_mint_pair(
        pool_address: &Pubkey,
        mint_a: &Pubkey,
        mint_b: &Pubkey,
        expected_mint: &Pubkey,
        sol_mint: &Pubkey,
    ) -> BotResult<()> {
        // Check that expected mint is present
        if mint_a != expected_mint && mint_b != expected_mint {
            return Err(BotError::PoolValidationError(format!(
                "Mint {} is not present in pool {}. Pool has {} and {}",
                expected_mint, pool_address, mint_a, mint_b
            )));
        }

        // Check that SOL is present
        if mint_a != sol_mint && mint_b != sol_mint {
            return Err(BotError::PoolValidationError(format!(
                "SOL mint is not present in pool {}. Pool has {} and {}",
                pool_address, mint_a, mint_b
            )));
        }

        Ok(())
    }

    /// Validate pool ownership
    pub fn validate_owner(
        pool_address: &Pubkey,
        actual_owner: &Pubkey,
        expected_owner: &Pubkey,
    ) -> BotResult<()> {
        if actual_owner != expected_owner {
            return Err(BotError::InvalidAccountOwner {
                address: *pool_address,
                expected_owner: *expected_owner,
                actual_owner: *actual_owner,
            });
        }
        Ok(())
    }

    /// Determine which vault is for the token and which is for SOL
    pub fn order_vaults(
        mint_a: &Pubkey,
        mint_b: &Pubkey,
        vault_a: Pubkey,
        vault_b: Pubkey,
        sol_mint: &Pubkey,
    ) -> (Pubkey, Pubkey) {
        if mint_a == sol_mint {
            (vault_b, vault_a) // (token_vault, sol_vault)
        } else {
            (vault_a, vault_b) // (token_vault, sol_vault)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_validate_addresses() {
        struct TestInitializer;
        
        #[async_trait]
        impl PoolInitializer for TestInitializer {
            type Pool = (); // Dummy type for testing
            
            async fn initialize_pools(
                &self,
                _addresses: &[String],
                _rpc_client: Arc<RpcClient>,
                _mint: &Pubkey,
            ) -> BotResult<Vec<Self::Pool>> {
                Ok(vec![])
            }
            
            fn dex_name(&self) -> &'static str {
                "Test"
            }
        }

        let initializer = TestInitializer;
        
        // Valid addresses
        let valid = vec![
            "So11111111111111111111111111111111111111112".to_string(),
        ];
        assert!(initializer.validate_addresses(&valid).is_ok());

        // Invalid address
        let invalid = vec!["invalid".to_string()];
        assert!(initializer.validate_addresses(&invalid).is_err());
    }

    #[test]
    fn test_pool_validator_mint_pair() {
        let pool = Pubkey::from_str("11111111111111111111111111111111").unwrap();
        let mint_a = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
        let mint_b = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
        let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();

        // Valid: contains both expected mint (mint_b) and SOL (mint_a)
        assert!(PoolValidator::validate_mint_pair(&pool, &mint_a, &mint_b, &mint_b, &sol_mint).is_ok());

        // Invalid: expected mint not present
        let wrong_mint = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
        assert!(PoolValidator::validate_mint_pair(&pool, &mint_a, &mint_b, &wrong_mint, &sol_mint).is_err());

        // Invalid: SOL not present
        let no_sol = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
        assert!(PoolValidator::validate_mint_pair(&pool, &mint_b, &wrong_mint, &mint_b, &no_sol).is_err());
    }

    #[test]
    fn test_order_vaults() {
        let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
        let token_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
        let vault_a = Pubkey::from_str("11111111111111111111111111111111").unwrap();
        let vault_b = Pubkey::from_str("22222222222222222222222222222222").unwrap();

        // When SOL is mint_a, token vault should be vault_b
        let (token, sol) = PoolValidator::order_vaults(&sol_mint, &token_mint, vault_a, vault_b, &sol_mint);
        assert_eq!(token, vault_b);
        assert_eq!(sol, vault_a);

        // When SOL is mint_b, token vault should be vault_a
        let (token, sol) = PoolValidator::order_vaults(&token_mint, &sol_mint, vault_a, vault_b, &sol_mint);
        assert_eq!(token, vault_a);
        assert_eq!(sol, vault_b);
    }
}
