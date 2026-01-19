use crate::dex::{
    meteora::{
        damm_initializer::MeteoraDammInitializer, damm_v2_initializer::MeteoraDammV2Initializer,
        dlmm_initializer::MeteoraDlmmInitializer,
    },
    pump::initializer::PumpInitializer,
    raydium::{
        clmm_initializer::RaydiumClmmInitializer, cp_initializer::RaydiumCpInitializer,
        initializer::RaydiumCpmmInitializer,
    },
    solfi::initializer::SolfiInitializer,
    traits::PoolInitializer,
    vertigo::initializer::VertigoInitializer,
    whirlpool::initializer::WhirlpoolInitializer,
};
use crate::pools::MintPoolData;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use std::str::FromStr;
use std::sync::Arc;
use tracing::info;

pub async fn initialize_pool_data(
    mint: &str,
    wallet_account: &str,
    raydium_pools: Option<&Vec<String>>,
    raydium_cp_pools: Option<&Vec<String>>,
    pump_pools: Option<&Vec<String>>,
    dlmm_pools: Option<&Vec<String>>,
    whirlpool_pools: Option<&Vec<String>>,
    raydium_clmm_pools: Option<&Vec<String>>,
    meteora_damm_pools: Option<&Vec<String>>,
    solfi_pools: Option<&Vec<String>>,
    meteora_damm_v2_pools: Option<&Vec<String>>,
    vertigo_pools: Option<&Vec<String>>,
    rpc_client: Arc<RpcClient>,
) -> anyhow::Result<MintPoolData> {
    info!("Initializing pool data for mint: {}", mint);

    let mint_pubkey = Pubkey::from_str(mint)?;
    let mint_account = rpc_client.get_account(&mint_pubkey)?;

    let token_2022_program_id =
        Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb").unwrap();
    let token_program = if mint_account.owner == spl_token::ID {
        spl_token::ID
    } else if mint_account.owner == token_2022_program_id {
        token_2022_program_id
    } else {
        return Err(anyhow::anyhow!("Unknown token program for mint: {}", mint));
    };

    info!("Detected token program: {}", token_program);
    let mut pool_data = MintPoolData::new(mint, wallet_account, token_program)?;

    // Helper macro to initialize pools
    macro_rules! init_pools {
        ($pool_list:expr, $initializer_type:ty, $target_vec:expr, $name:expr) => {
            if let Some(pools) = $pool_list {
                if !pools.is_empty() {
                    info!("Initializing {} {} pools...", pools.len(), $name);
                    let initializer = <$initializer_type>::new();
                    match initializer
                        .initialize_pools(pools, rpc_client.clone(), &mint_pubkey)
                        .await
                    {
                        Ok(initialized) => {
                            info!("Successfully initialized {} {} pools", initialized.len(), $name);
                            $target_vec.extend(initialized);
                        }
                        Err(e) => {
                            tracing::error!("Failed to initialize {} pools: {}", $name, e);
                        }
                    }
                }
            }
        };
    }

    // 1. Pump.fun
    init_pools!(
        pump_pools,
        PumpInitializer,
        pool_data.pump_pools,
        "Pump.fun"
    );

    // 2. Raydium CPMM (Standard)
    init_pools!(
        raydium_pools,
        RaydiumCpmmInitializer,
        pool_data.raydium_pools,
        "Raydium CPMM"
    );

    // 3. Raydium CP
    init_pools!(
        raydium_cp_pools,
        RaydiumCpInitializer,
        pool_data.raydium_cp_pools,
        "Raydium CP"
    );

    // 4. Raydium CLMM
    init_pools!(
        raydium_clmm_pools,
        RaydiumClmmInitializer,
        pool_data.raydium_clmm_pools,
        "Raydium CLMM"
    );

    // 5. Orca Whirlpool
    init_pools!(
        whirlpool_pools,
        WhirlpoolInitializer,
        pool_data.whirlpool_pools,
        "Whirlpool"
    );

    // 6. Meteora DLMM
    init_pools!(
        dlmm_pools,
        MeteoraDlmmInitializer,
        pool_data.dlmm_pairs,
        "Meteora DLMM"
    );

    // 7. Meteora DAMM
    init_pools!(
        meteora_damm_pools,
        MeteoraDammInitializer,
        pool_data.meteora_damm_pools,
        "Meteora DAMM"
    );

    // 8. Meteora DAMM V2
    init_pools!(
        meteora_damm_v2_pools,
        MeteoraDammV2Initializer,
        pool_data.meteora_damm_v2_pools,
        "Meteora DAMM V2"
    );

    // 9. Solfi
    init_pools!(
        solfi_pools,
        SolfiInitializer,
        pool_data.solfi_pools,
        "Solfi"
    );

    // 10. Vertigo
    init_pools!(
        vertigo_pools,
        VertigoInitializer,
        pool_data.vertigo_pools,
        "Vertigo"
    );

    info!("Pool initialization complete for mint: {}", mint);
    Ok(pool_data)
}
