/// Vertigo Pool Initializer

use crate::constants::sol_mint;
use crate::dex::vertigo::constants::vertigo_program_id;
use crate::dex::traits::{DexPool, PoolInitializer, PoolValidator};
use crate::error::{BotError, BotResult};
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct VertigoPool {
    pub pool: Pubkey,
    pub pool_owner: Pubkey,
    pub token_vault: Pubkey,
    pub sol_vault: Pubkey,
}

#[async_trait]
impl DexPool for VertigoPool {
    async fn initialize(&mut self, _rpc_client: &RpcClient, _pool_address: &Pubkey) -> BotResult<()> {
        Ok(())
    }

    fn get_swap_accounts(&self, _wallet: &Pubkey) -> Vec<AccountMeta> {
        vec![
            AccountMeta::new_readonly(vertigo_program_id(), false),
            AccountMeta::new(self.pool, false),
            AccountMeta::new_readonly(self.pool_owner, false),
            AccountMeta::new(self.token_vault, false),
            AccountMeta::new(self.sol_vault, false),
        ]
    }

    fn get_liquidity(&self) -> (u64, u64) {
        (0, 0)
    }

    fn dex_name(&self) -> &'static str {
        "Vertigo"
    }

    fn pool_address(&self) -> Pubkey {
        self.pool
    }

    fn contains_mint(&self, _mint: &Pubkey) -> bool {
        true
    }
}

pub struct VertigoInitializer;

impl VertigoInitializer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PoolInitializer for VertigoInitializer {
    type Pool = VertigoPool;

    async fn initialize_pools(
        &self,
        addresses: &[String],
        rpc_client: Arc<RpcClient>,
        mint: &Pubkey,
    ) -> BotResult<Vec<Self::Pool>> {
        let pool_pubkeys = self.validate_addresses(addresses)?;
        let mut pools = Vec::new();

        for pool_address in pool_pubkeys {
            match self.initialize_single_pool(&rpc_client, &pool_address, mint).await {
                Ok(pool) => {
                    info!("✓ Initialized Vertigo pool: {}", pool_address);
                    pools.push(pool);
                }
                Err(e) => {
                    error!("✗ Failed Vertigo pool {}: {}", pool_address, e);
                    return Err(e);
                }
            }
        }
        Ok(pools)
    }

    fn dex_name(&self) -> &'static str {
        "Vertigo"
    }
}

impl VertigoInitializer {
    async fn initialize_single_pool(
        &self,
        rpc_client: &RpcClient,
        pool_address: &Pubkey,
        _expected_mint: &Pubkey,
    ) -> BotResult<VertigoPool> {
        let account = rpc_client.get_account(pool_address).map_err(|e| {
            BotError::AccountFetchError {
                address: *pool_address,
                reason: format!("Failed to fetch Vertigo pool: {}", e),
            }
        })?;

        PoolValidator::validate_owner(pool_address, &account.owner, &vertigo_program_id())?;

        Ok(VertigoPool {
            pool: *pool_address,
            pool_owner: Pubkey::new_unique(),
            token_vault: Pubkey::new_unique(),
            sol_vault: Pubkey::new_unique(),
        })
    }
}
