/// Whirlpool Pool Initializer (Orca)
/// 
/// Implementation of the PoolInitializer trait for Orca Whirlpool pools.

use crate::constants::sol_mint;
use crate::dex::traits::{ConcentratedLiquidityPool, DexPool, PoolInitializer, PoolValidator};
use crate::dex::whirlpool::{whirlpool_program_id, WhirlpoolInfo};
use crate::error::{BotError, BotResult};
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use tracing::{error, info};

/// Whirlpool Pool structure
#[derive(Debug, Clone)]
pub struct WhirlpoolPool {
    pub pool: Pubkey,
    pub oracle: Pubkey,
    pub token_vault_a: Pubkey,
    pub token_vault_b: Pubkey,
    pub tick_arrays: Vec<Pubkey>,
    pub current_tick: i32,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
}

#[async_trait]
impl DexPool for WhirlpoolPool {
    async fn initialize(&mut self, _rpc_client: &RpcClient, _pool_address: &Pubkey) -> BotResult<()> {
        Ok(())
    }

    fn get_swap_accounts(&self, _wallet: &Pubkey) -> Vec<AccountMeta> {
        let mut accounts = vec![
            AccountMeta::new_readonly(whirlpool_program_id(), false),
            AccountMeta::new(self.pool, false),
            AccountMeta::new(self.token_vault_a, false),
            AccountMeta::new(self.token_vault_b, false),
            AccountMeta::new_readonly(self.oracle, false),
        ];

        // Add tick arrays
        for tick_array in &self.tick_arrays {
            accounts.push(AccountMeta::new(*tick_array, false));
        }

        accounts
    }

    fn get_liquidity(&self) -> (u64, u64) {
        (0, 0)
    }

    fn dex_name(&self) -> &'static str {
        "Orca Whirlpool"
    }

    fn pool_address(&self) -> Pubkey {
        self.pool
    }

    fn contains_mint(&self, mint: &Pubkey) -> bool {
        &self.token_mint_a == mint || &self.token_mint_b == mint
    }
}

impl ConcentratedLiquidityPool for WhirlpoolPool {
    fn get_tick_arrays(&self) -> &[Pubkey] {
        &self.tick_arrays
    }

    fn current_tick(&self) -> i32 {
        self.current_tick
    }
}

/// Whirlpool Pool Initializer
pub struct WhirlpoolInitializer;

impl WhirlpoolInitializer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PoolInitializer for WhirlpoolInitializer {
    type Pool = WhirlpoolPool;

    async fn initialize_pools(
        &self,
        addresses: &[String],
        rpc_client: Arc<RpcClient>,
        mint: &Pubkey,
    ) -> BotResult<Vec<Self::Pool>> {
        let pool_pubkeys = self.validate_addresses(addresses)?;
        let mut pools = Vec::with_capacity(pool_pubkeys.len());

        for pool_address in pool_pubkeys {
            match self.initialize_single_pool(&rpc_client, &pool_address, mint).await {
                Ok(pool) => {
                    info!("✓ Initialized Whirlpool pool: {}", pool_address);
                    pools.push(pool);
                }
                Err(e) => {
                    error!("✗ Failed to initialize Whirlpool pool {}: {}", pool_address, e);
                    return Err(e);
                }
            }
        }

        Ok(pools)
    }

    fn dex_name(&self) -> &'static str {
        "Orca Whirlpool"
    }
}

impl WhirlpoolInitializer {
    async fn initialize_single_pool(
        &self,
        rpc_client: &RpcClient,
        pool_address: &Pubkey,
        expected_mint: &Pubkey,
    ) -> BotResult<WhirlpoolPool> {
        let account = rpc_client.get_account(pool_address).map_err(|e| {
            BotError::AccountFetchError {
                address: *pool_address,
                reason: format!("Failed to fetch Whirlpool pool: {}", e),
            }
        })?;

        PoolValidator::validate_owner(pool_address, &account.owner, &whirlpool_program_id())?;

        let pool_info = WhirlpoolInfo::load_checked(&account.data).map_err(|e| {
            BotError::DeserializationError {
                data_type: "WhirlpoolInfo".to_string(),
                source: Box::new(e),
            }
        })?;

        let sol_mint_pubkey = sol_mint();
        PoolValidator::validate_mint_pair(
            pool_address,
            &pool_info.token_mint_a,
            &pool_info.token_mint_b,
            expected_mint,
            &sol_mint_pubkey,
        )?;

        let (token_vault, sol_vault) = PoolValidator::order_vaults(
            &pool_info.token_mint_a,
            &pool_info.token_mint_b,
            pool_info.token_vault_a,
            pool_info.token_vault_b,
            &sol_mint_pubkey,
        );

        // For simplicity, using basic tick arrays
        // In production, calculate actual tick arrays based on current tick
        let tick_arrays = pool_info.tick_arrays.unwrap_or_default();

        Ok(WhirlpoolPool {
            pool: *pool_address,
            oracle: pool_info.oracle,
            token_vault_a: token_vault,
            token_vault_b: sol_vault,
            tick_arrays,
            current_tick: pool_info.tick_current_index,
            token_mint_a: pool_info.token_mint_a,
            token_mint_b: pool_info.token_mint_b,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whirlpool_dex_name() {
        let pool = WhirlpoolPool {
            pool: Pubkey::new_unique(),
            oracle: Pubkey::new_unique(),
            token_vault_a: Pubkey::new_unique(),
            token_vault_b: Pubkey::new_unique(),
            tick_arrays: vec![],
            current_tick: 0,
            token_mint_a: Pubkey::new_unique(),
            token_mint_b: Pubkey::new_unique(),
        };

        assert_eq!(pool.dex_name(), "Orca Whirlpool");
    }

    #[test]
    fn test_whirlpool_concentrated_liquidity_trait() {
        let tick1 = Pubkey::new_unique();
        let tick2 = Pubkey::new_unique();
        
        let pool = WhirlpoolPool {
            pool: Pubkey::new_unique(),
            oracle: Pubkey::new_unique(),
            token_vault_a: Pubkey::new_unique(),
            token_vault_b: Pubkey::new_unique(),
            tick_arrays: vec![tick1, tick2],
            current_tick: 42,
            token_mint_a: Pubkey::new_unique(),
            token_mint_b: Pubkey::new_unique(),
        };

        assert_eq!(pool.current_tick(), 42);
        assert_eq!(pool.get_tick_arrays().len(), 2);
        assert_eq!(pool.get_tick_arrays()[0], tick1);
    }
}
