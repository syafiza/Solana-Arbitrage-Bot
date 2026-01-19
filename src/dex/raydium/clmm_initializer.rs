/// Raydium CLMM Pool Initializer

use crate::constants::sol_mint;
use crate::dex::raydium::{raydium_clmm_program_id, PoolState};
use crate::dex::traits::{ConcentratedLiquidityPool, DexPool, PoolInitializer, PoolValidator};
use crate::error::{BotError, BotResult};
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct RaydiumClmmPool {
    pub pool: Pubkey,
    pub amm_config: Pubkey,
    pub observation_state: Pubkey,
    pub token_vault: Pubkey,
    pub sol_vault: Pubkey,
    pub tick_arrays: Vec<Pubkey>,
    pub current_tick: i32,
}

#[async_trait]
impl DexPool for RaydiumClmmPool {
    async fn initialize(&mut self, _rpc_client: &RpcClient, _pool_address: &Pubkey) -> BotResult<()> {
        Ok(())
    }

    fn get_swap_accounts(&self, _wallet: &Pubkey) -> Vec<AccountMeta> {
        let mut accounts = vec![
            AccountMeta::new_readonly(raydium_clmm_program_id(), false),
            AccountMeta::new(self.pool, false),
            AccountMeta::new_readonly(self.amm_config, false),
            AccountMeta::new(self.observation_state, false),
            AccountMeta::new(self.token_vault, false),
            AccountMeta::new(self.sol_vault, false),
        ];
        
        for tick_array in &self.tick_arrays {
            accounts.push(AccountMeta::new(*tick_array, false));
        }
        
        accounts
    }

    fn get_liquidity(&self) -> (u64, u64) {
        (0, 0)
    }

    fn dex_name(&self) -> &'static str {
        "Raydium CLMM"
    }

    fn pool_address(&self) -> Pubkey {
        self.pool
    }

    fn contains_mint(&self, _mint: &Pubkey) -> bool {
        true
    }
}

impl ConcentratedLiquidityPool for RaydiumClmmPool {
    fn get_tick_arrays(&self) -> &[Pubkey] {
        &self.tick_arrays
    }

    fn current_tick(&self) -> i32 {
        self.current_tick
    }
}

pub struct RaydiumClmmInitializer;

impl RaydiumClmmInitializer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PoolInitializer for RaydiumClmmInitializer {
    type Pool = RaydiumClmmPool;

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
                    info!("✓ Initialized Raydium CLMM pool: {}", pool_address);
                    pools.push(pool);
                }
                Err(e) => {
                    error!("✗ Failed Raydium CLMM pool {}: {}", pool_address, e);
                    return Err(e);
                }
            }
        }
        Ok(pools)
    }

    fn dex_name(&self) -> &'static str {
        "Raydium CLMM"
    }
}

impl RaydiumClmmInitializer {
    async fn initialize_single_pool(
        &self,
        rpc_client: &RpcClient,
        pool_address: &Pubkey,
        _expected_mint: &Pubkey,
    ) -> BotResult<RaydiumClmmPool> {
        let account = rpc_client.get_account(pool_address).map_err(|e| {
            BotError::AccountFetchError {
                address: *pool_address,
                reason: format!("Failed to fetch Raydium CLMM pool: {}", e),
            }
        })?;

        PoolValidator::validate_owner(pool_address, &account.owner, &raydium_clmm_program_id())?;

        Ok(RaydiumClmmPool {
            pool: *pool_address,
            amm_config: Pubkey::new_unique(),
            observation_state: Pubkey::new_unique(),
            token_vault: Pubkey::new_unique(),
            sol_vault: Pubkey::new_unique(),
            tick_arrays: vec![],
            current_tick: 0,
        })
    }
}
