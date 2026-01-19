/// Raydium CPMM Pool Initializer
/// 
/// Reference implementation of the PoolInitializer trait for Raydium AMM pools.
/// This demonstrates the pattern that will be replicated for all 10 DEX types.

use crate::constants::sol_mint;
use crate::dex::raydium::{raydium_authority, raydium_program_id, RaydiumAmmInfo};
use crate::dex::traits::{DexPool, PoolInitializer, PoolValidator};
use crate::error::{BotError, BotResult};
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use tracing::{error, info};

/// Raydium CPMM Pool structure
#[derive(Debug, Clone)]
pub struct RaydiumCpmmPool {
    pub pool: Pubkey,
    pub token_vault: Pubkey,
    pub sol_vault: Pubkey,
    pub coin_mint: Pubkey,
    pub pc_mint: Pubkey,
}

#[async_trait]
impl DexPool for RaydiumCpmmPool {
    async fn initialize(
        &mut self,
        rpc_client: &RpcClient,
        pool_address: &Pubkey,
    ) -> BotResult<()> {
        // This method would be called if the pool wasn't pre-initialized
        // For now, the initialization happens in the PoolInitializer
        Ok(())
    }

    fn get_swap_accounts(&self, _wallet: &Pubkey) -> Vec<AccountMeta> {
        vec![
            AccountMeta::new_readonly(raydium_program_id(), false),
            AccountMeta::new_readonly(raydium_authority(), false),
            AccountMeta::new(self.pool, false),
            AccountMeta::new(self.token_vault, false),
            AccountMeta::new(self.sol_vault, false),
        ]
    }

    fn get_liquidity(&self) -> (u64, u64) {
        // Would query actual liquidity in full implementation
        (0, 0)
    }

    fn dex_name(&self) -> &'static str {
        "Raydium CPMM"
    }

    fn pool_address(&self) -> Pubkey {
        self.pool
    }

    fn contains_mint(&self, mint: &Pubkey) -> bool {
        &self.coin_mint == mint || &self.pc_mint == mint
    }
}

/// Raydium CPMM Pool Initializer
pub struct RaydiumCpmmInitializer;

impl RaydiumCpmmInitializer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PoolInitializer for RaydiumCpmmInitializer {
    type Pool = RaydiumCpmmPool;

    async fn initialize_pools(
        &self,
        addresses: &[String],
        rpc_client: Arc<RpcClient>,
        mint: &Pubkey,
    ) -> BotResult<Vec<Self::Pool>> {
        let pool_pubkeys = self.validate_addresses(addresses)?;
        let mut pools = Vec::with_capacity(pool_pubkeys.len());

        for pool_address in pool_pubkeys {
            match self
                .initialize_single_pool(&rpc_client, &pool_address, mint)
                .await
            {
                Ok(pool) => {
                    info!("✓ Initialized Raydium CPMM pool: {}", pool_address);
                    pools.push(pool);
                }
                Err(e) => {
                    error!("✗ Failed to initialize Raydium CPMM pool {}: {}", pool_address, e);
                    return Err(e);
                }
            }
        }

        Ok(pools)
    }

    fn dex_name(&self) -> &'static str {
        "Raydium CPMM"
    }
}

impl RaydiumCpmmInitializer {
    /// Initialize a single pool (extracted for clarity)
    async fn initialize_single_pool(
        &self,
        rpc_client: &RpcClient,
        pool_address: &Pubkey,
        expected_mint: &Pubkey,
    ) -> BotResult<RaydiumCpmmPool> {
        // Fetch pool account
        let account = rpc_client.get_account(pool_address).map_err(|e| {
            BotError::AccountFetchError {
                address: *pool_address,
                reason: format!("Failed to fetch Raydium pool: {}", e),
            }
        })?;

        // Validate ownership
        PoolValidator::validate_owner(pool_address, &account.owner, &raydium_program_id())?;

        // Deserialize pool data
        let amm_info = RaydiumAmmInfo::load_checked(&account.data).map_err(|e| {
            BotError::DeserializationError {
                data_type: "RaydiumAmmInfo".to_string(),
                source: Box::new(e),
            }
        })?;

        // Validate mint pair (must contain expected mint and SOL)
        let sol_mint_pubkey = sol_mint();
        PoolValidator::validate_mint_pair(
            pool_address,
            &amm_info.coin_mint,
            &amm_info.pc_mint,
            expected_mint,
            &sol_mint_pubkey,
        )?;

        // Order vaults (token vault, sol vault)
        let (token_vault, sol_vault) = PoolValidator::order_vaults(
            &amm_info.coin_mint,
            &amm_info.pc_mint,
            amm_info.coin_vault,
            amm_info.pc_vault,
            &sol_mint_pubkey,
        );

        Ok(RaydiumCpmmPool {
            pool: *pool_address,
            token_vault,
            sol_vault,
            coin_mint: amm_info.coin_mint,
            pc_mint: amm_info.pc_mint,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raydium_pool_contains_mint() {
        let mint1 = Pubkey::new_unique();
        let mint2 = Pubkey::new_unique();
        
        let pool = RaydiumCpmmPool {
            pool: Pubkey::new_unique(),
            token_vault: Pubkey::new_unique(),
            sol_vault: Pubkey::new_unique(),
            coin_mint: mint1,
            pc_mint: mint2,
        };

        assert!(pool.contains_mint(&mint1));
        assert!(pool.contains_mint(&mint2));
        assert!(!pool.contains_mint(&Pubkey::new_unique()));
    }

    #[test]
    fn test_raydium_dex_name() {
        let pool = RaydiumCpmmPool {
            pool: Pubkey::new_unique(),
            token_vault: Pubkey::new_unique(),
            sol_vault: Pubkey::new_unique(),
            coin_mint: Pubkey::new_unique(),
            pc_mint: Pubkey::new_unique(),
        };

        assert_eq!(pool.dex_name(), "Raydium CPMM");
    }
}
