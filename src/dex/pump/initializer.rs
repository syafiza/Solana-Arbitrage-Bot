/// Pump.fun Pool Initializer
/// 
/// Implementation of the PoolInitializer trait for Pump.fun pools.

use crate::constants::sol_mint;
use crate::dex::pump::{pump_fee_wallet, pump_program_id, PumpAmmInfo};
use crate::dex::traits::{DexPool, PoolInitializer, PoolValidator};
use crate::error::{BotError, BotResult};
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use std::sync::Arc;
use tracing::{error, info};

/// Pump.fun Pool structure
#[derive(Debug, Clone)]
pub struct PumpPool {
    pub pool: Pubkey,
    pub token_vault: Pubkey,
    pub sol_vault: Pubkey,
    pub fee_token_wallet: Pubkey,
    pub coin_creator_vault_ata: Pubkey,
    pub coin_creator_vault_authority: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
}

#[async_trait]
impl DexPool for PumpPool {
    async fn initialize(&mut self, _rpc_client: &RpcClient, _pool_address: &Pubkey) -> BotResult<()> {
        Ok(())
    }

    fn get_swap_accounts(&self, _wallet: &Pubkey) -> Vec<AccountMeta> {
        vec![
            AccountMeta::new_readonly(pump_program_id(), false),
            AccountMeta::new_readonly(self.pool, false),
            AccountMeta::new(self.token_vault, false),
            AccountMeta::new(self.sol_vault, false),
            AccountMeta::new(self.fee_token_wallet, false),
            AccountMeta::new(self.coin_creator_vault_ata, false),
            AccountMeta::new_readonly(self.coin_creator_vault_authority, false),
        ]
    }

    fn get_liquidity(&self) -> (u64, u64) {
        (0, 0)
    }

    fn dex_name(&self) -> &'static str {
        "Pump.fun"
    }

    fn pool_address(&self) -> Pubkey {
        self.pool
    }

    fn contains_mint(&self, mint: &Pubkey) -> bool {
        &self.base_mint == mint || &self.quote_mint == mint
    }
}

/// Pump.fun Pool Initializer
pub struct PumpInitializer;

impl PumpInitializer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PoolInitializer for PumpInitializer {
    type Pool = PumpPool;

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
                    info!("✓ Initialized Pump.fun pool: {}", pool_address);
                    pools.push(pool);
                }
                Err(e) => {
                    error!("✗ Failed to initialize Pump.fun pool {}: {}", pool_address, e);
                    return Err(e);
                }
            }
        }

        Ok(pools)
    }

    fn dex_name(&self) -> &'static str {
        "Pump.fun"
    }
}

impl PumpInitializer {
    async fn initialize_single_pool(
        &self,
        rpc_client: &RpcClient,
        pool_address: &Pubkey,
        expected_mint: &Pubkey,
    ) -> BotResult<PumpPool> {
        let account = rpc_client.get_account(pool_address).map_err(|e| {
            BotError::AccountFetchError {
                address: *pool_address,
                reason: format!("Failed to fetch Pump pool: {}", e),
            }
        })?;

        PoolValidator::validate_owner(pool_address, &account.owner, &pump_program_id())?;

        let amm_info = PumpAmmInfo::load_checked(&account.data).map_err(|e| {
            BotError::DeserializationError {
                data_type: "PumpAmmInfo".to_string(),
                source: Box::new(e),
            }
        })?;

        let sol_mint_pubkey = sol_mint();
        
        // Determine which vault is token and which is SOL
        let (token_vault, sol_vault) = if sol_mint_pubkey == amm_info.base_mint {
            (amm_info.pool_quote_token_account, amm_info.pool_base_token_account)
        } else if sol_mint_pubkey == amm_info.quote_mint {
            (amm_info.pool_base_token_account, amm_info.pool_quote_token_account)
        } else {
            // Fallback if SOL not detected (shouldn't happen)
            (amm_info.pool_base_token_account, amm_info.pool_quote_token_account)
        };

        // Validate that expected mint is present
        if expected_mint != &amm_info.base_mint && expected_mint != &amm_info.quote_mint {
            return Err(BotError::PoolValidationError(format!(
                "Mint {} is not present in Pump pool {}",
                expected_mint, pool_address
            )));
        }

        let fee_token_wallet = get_associated_token_address(&pump_fee_wallet(), &amm_info.quote_mint);
        let coin_creator_vault_ata = get_associated_token_address(
            &amm_info.coin_creator_vault_authority,
            &amm_info.quote_mint,
        );

        Ok(PumpPool {
            pool: *pool_address,
            token_vault,
            sol_vault,
            fee_token_wallet,
            coin_creator_vault_ata,
            coin_creator_vault_authority: amm_info.coin_creator_vault_authority,
            base_mint: amm_info.base_mint,
            quote_mint: amm_info.quote_mint,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pump_pool_dex_name() {
        let pool = PumpPool {
            pool: Pubkey::new_unique(),
            token_vault: Pubkey::new_unique(),
            sol_vault: Pubkey::new_unique(),
            fee_token_wallet: Pubkey::new_unique(),
            coin_creator_vault_ata: Pubkey::new_unique(),
            coin_creator_vault_authority: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
        };

        assert_eq!(pool.dex_name(), "Pump.fun");
    }
}
