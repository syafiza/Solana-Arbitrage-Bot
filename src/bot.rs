use crate::config::Config;
use crate::constants::{
    ATA_CREATION_COMPUTE_UNIT_LIMIT, ATA_CREATION_COMPUTE_UNIT_PRICE,
    DEFAULT_BLOCKHASH_REFRESH_INTERVAL_SECS, DEFAULT_LOOKUP_TABLE_PUBKEY,
};
use crate::error::{BotError, BotResult};
use crate::refresh::initialize_pool_data;
use crate::transaction::build_and_send_transaction;
use solana_client::rpc_client::RpcClient;
use solana_sdk::address_lookup_table::AddressLookupTableAccount;
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::{
    address_lookup_table::state::AddressLookupTable, compute_budget::ComputeBudgetInstruction,
};
use spl_associated_token_account::{
    get_associated_token_address, get_associated_token_address_with_program_id,
};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

pub async fn run_bot(config_path: &str) -> BotResult<()> {
    let config = Config::load(config_path)?;
    info!("Configuration loaded successfully");

    let rpc_client = Arc::new(RpcClient::new(config.rpc.url.clone()));

    let sending_rpc_clients = if let Some(spam_config) = &config.spam {
        if spam_config.enabled {
            spam_config
                .sending_rpc_urls
                .iter()
                .map(|url| Arc::new(RpcClient::new(url.clone())))
                .collect::<Vec<_>>()
        } else {
            vec![rpc_client.clone()]
        }
    } else {
        vec![rpc_client.clone()]
    };

    let wallet_kp = load_keypair(&config.wallet.private_key)?;
    info!("Wallet loaded: {}", wallet_kp.pubkey());

    let initial_blockhash = rpc_client
        .get_latest_blockhash()
        .map_err(|e| BotError::rpc_retryable(config.rpc.url.clone(), format!("Failed to get initial blockhash: {}", e)))?;
    let cached_blockhash = Arc::new(Mutex::new(initial_blockhash));

    let refresh_interval = Duration::from_secs(DEFAULT_BLOCKHASH_REFRESH_INTERVAL_SECS);
    let blockhash_client = rpc_client.clone();
    let blockhash_cache = cached_blockhash.clone();
    let rpc_url_for_task = config.rpc.url.clone();
    tokio::spawn(async move {
        blockhash_refresher(blockhash_client, blockhash_cache, refresh_interval, rpc_url_for_task).await;
    });

    for mint_config in &config.routing.mint_config_list {
        // Get the mint account info to check owner
        let mint_pubkey = Pubkey::from_str(&mint_config.mint)
            .map_err(|e| BotError::InvalidPublicKey {
                key: mint_config.mint.clone(),
                source: e,
            })?;

        let mint_account = rpc_client
            .get_account(&mint_pubkey)
            .map_err(|e| BotError::AccountFetchError {
                address: mint_pubkey,
                reason: format!("Failed to fetch mint account: {}", e),
            })?;
        
        let mint_owner = mint_account.owner;
        let wallet_token_account = get_associated_token_address_with_program_id(
            &wallet_kp.pubkey(),
            &mint_pubkey,
            &mint_owner,
        );

        println!("   Token mint: {}", mint_config.mint);
        println!("   Wallet token ATA: {}", wallet_token_account);
        // Check if the PWEASE token account exists and create it if it doesn't
        println!("\n   Checking if token account exists...");
        loop {
            match rpc_client.get_account(&wallet_token_account) {
                Ok(_) => {
                    println!("   token account exists!");
                    break;
                }
                Err(_) => {
                    println!("   token account does not exist. Creating it...");

                    // Create the instruction to create the associated token account
                    let create_ata_ix =
                            spl_associated_token_account::instruction::create_associated_token_account_idempotent(
                                &wallet_kp.pubkey(), // Funding account
                                &wallet_kp.pubkey(), // Wallet account
                                &mint_pubkey,        // Token mint
                                &spl_token::ID,      // Token program
                            );

                    // Get a recent blockhash
                    let blockhash = rpc_client.get_latest_blockhash()?;

                    let compute_unit_price_ix =
                        ComputeBudgetInstruction::set_compute_unit_price(ATA_CREATION_COMPUTE_UNIT_PRICE);
                    let compute_unit_limit_ix =
                        ComputeBudgetInstruction::set_compute_unit_limit(ATA_CREATION_COMPUTE_UNIT_LIMIT);

                    // Create the transaction
                    let create_ata_tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
                        &[compute_unit_price_ix, compute_unit_limit_ix, create_ata_ix],
                        Some(&wallet_kp.pubkey()),
                        &[&wallet_kp],
                        blockhash,
                    );

                    // Send the transaction
                    match rpc_client.send_and_confirm_transaction(&create_ata_tx) {
                        Ok(sig) => {
                            println!("   token account created successfully! Signature: {}", sig);
                        }
                        Err(e) => {
                            let err = BotError::WalletError(format!("Failed to create token account for {}: {}", mint_config.mint, e));
                            error!("{}", err);
                            return Err(err);
                        }
                    }
                }
            }
        }
    }

    for mint_config in &config.routing.mint_config_list {
        info!("Processing mint: {}", mint_config.mint);

        let pool_data = initialize_pool_data(
            &mint_config.mint,
            &wallet_kp.pubkey().to_string(),
            mint_config.raydium_pool_list.as_ref(),
            mint_config.raydium_cp_pool_list.as_ref(),
            mint_config.pump_pool_list.as_ref(),
            mint_config.meteora_dlmm_pool_list.as_ref(),
            mint_config.whirlpool_pool_list.as_ref(),
            mint_config.raydium_clmm_pool_list.as_ref(),
            mint_config.meteora_damm_pool_list.as_ref(),
            mint_config.solfi_pool_list.as_ref(),
            mint_config.meteora_damm_v2_pool_list.as_ref(),
            mint_config.vertigo_pool_list.as_ref(),
            rpc_client.clone(),
        )
        .await?;

        let mint_pool_data = Arc::new(Mutex::new(pool_data));

        // TODO: Add logic to periodically refresh pool data

        let config_clone = config.clone();
        let mint_config_clone = mint_config.clone();
        let sending_rpc_clients_clone = sending_rpc_clients.clone();
        let cached_blockhash_clone = cached_blockhash.clone();
        let wallet_bytes = wallet_kp.to_bytes();
        let wallet_kp_clone = Keypair::from_bytes(&wallet_bytes)
            .map_err(|e| BotError::WalletError(format!("Failed to clone keypair: {}", e)))?;
        let mut lookup_table_accounts = mint_config_clone.lookup_table_accounts.unwrap_or_default();
        lookup_table_accounts.push(DEFAULT_LOOKUP_TABLE_PUBKEY.to_string());

        let mut lookup_table_accounts_list = vec![];

        for lookup_table_account in lookup_table_accounts {
            match Pubkey::from_str(&lookup_table_account) {
                Ok(pubkey) => {
                    match rpc_client.get_account(&pubkey) {
                        Ok(account) => {
                            match AddressLookupTable::deserialize(&account.data) {
                                Ok(lookup_table) => {
                                    let lookup_table_account = AddressLookupTableAccount {
                                        key: pubkey,
                                        addresses: lookup_table.addresses.into_owned(),
                                    };
                                    lookup_table_accounts_list.push(lookup_table_account);
                                    info!("   Successfully loaded lookup table: {}", pubkey);
                                }
                                Err(e) => {
                                    error!(
                                        "   Failed to deserialize lookup table {}: {}",
                                        pubkey, e
                                    );
                                    continue; // Skip this lookup table but continue processing others
                                }
                            }
                        }
                        Err(e) => {
                            error!("   Failed to fetch lookup table account {}: {}", pubkey, e);
                            continue; // Skip this lookup table but continue processing others
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "   Invalid lookup table pubkey string {}: {}",
                        lookup_table_account, e
                    );
                    continue; // Skip this lookup table but continue processing others
                }
            }
        }
        if lookup_table_accounts_list.is_empty() {
            warn!("   Warning: No valid lookup tables were loaded");
        } else {
            info!(
                "   Loaded {} lookup tables successfully",
                lookup_table_accounts_list.len()
            );
        }

        tokio::spawn(async move {
            let process_delay = Duration::from_millis(mint_config_clone.process_delay);

            loop {
                let latest_blockhash = {
                    let guard = cached_blockhash_clone.lock().await;
                    *guard
                };

                let guard = mint_pool_data.lock().await;

                match build_and_send_transaction(
                    &wallet_kp_clone,
                    &config_clone,
                    &*guard, // Dereference the guard here
                    &sending_rpc_clients_clone,
                    latest_blockhash,
                    &lookup_table_accounts_list,
                )
                .await
                {
                    Ok(signatures) => {
                        info!(
                            "Transactions sent successfully for mint {}",
                            mint_config_clone.mint
                        );
                        for signature in signatures {
                            info!("  Signature: {}", signature);
                        }
                    }
                    Err(e) => {
                        error!(
                            "Error sending transaction for mint {}: {}",
                            mint_config_clone.mint, e
                        );
                    }
                }

                tokio::time::sleep(process_delay).await;
            }
        });
    }

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn blockhash_refresher(
    rpc_client: Arc<RpcClient>,
    cached_blockhash: Arc<Mutex<Hash>>,
    refresh_interval: Duration,
    rpc_url: String,
) {
    loop {
        match rpc_client.get_latest_blockhash() {
            Ok(blockhash) => {
                let mut guard = cached_blockhash.lock().await;
                *guard = blockhash;
                info!("Blockhash refreshed: {}", blockhash);
            }
            Err(e) => {
                let error = BotError::rpc_retryable(rpc_url.clone(), format!("Failed to refresh blockhash: {}", e));
                error!("{} (severity: {})", error, error.severity().as_str());
            }
        }
        tokio::time::sleep(refresh_interval).await;
    }
}

fn load_keypair(private_key: &str) -> BotResult<Keypair> {
    // Try base58 decoding first
    if let Ok(bytes) = bs58::decode(private_key).into_vec() {
        if let Ok(keypair) = Keypair::from_bytes(&bytes) {
            return Ok(keypair);
        }
    }

    // Try loading from file
    if let Ok(keypair) = solana_sdk::signature::read_keypair_file(private_key) {
        return Ok(keypair);
    }

    Err(BotError::WalletError(format!(
        "Failed to load keypair from '{}'. Expected base58-encoded private key or path to keypair file.",
        private_key
    )))
}
