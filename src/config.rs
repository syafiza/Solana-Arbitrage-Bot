use crate::error::{BotError, BotResult};
use serde::{Deserialize, Deserializer};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::{env, fs::File, io::Read};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub bot: BotConfig,
    pub routing: RoutingConfig,
    pub rpc: RpcConfig,
    pub spam: Option<SpamConfig>,
    pub wallet: WalletConfig,
    pub flashloan: Option<FlashloanConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BotConfig {
    pub compute_unit_limit: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RoutingConfig {
    pub mint_config_list: Vec<MintConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MintConfig {
    pub mint: String,

    pub raydium_pool_list: Option<Vec<String>>,
    pub raydium_cp_pool_list: Option<Vec<String>>,
    pub raydium_clmm_pool_list: Option<Vec<String>>,

    pub meteora_dlmm_pool_list: Option<Vec<String>>,
    pub meteora_damm_pool_list: Option<Vec<String>>,
    pub meteora_damm_v2_pool_list: Option<Vec<String>>,

    pub pump_pool_list: Option<Vec<String>>,

    pub whirlpool_pool_list: Option<Vec<String>>,

    pub solfi_pool_list: Option<Vec<String>>,

    pub vertigo_pool_list: Option<Vec<String>>,

    pub lookup_table_accounts: Option<Vec<String>>,
    pub process_delay: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RpcConfig {
    #[serde(deserialize_with = "serde_string_or_env")]
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SpamConfig {
    pub enabled: bool,
    pub sending_rpc_urls: Vec<String>,
    pub compute_unit_price: u64,
    pub max_retries: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WalletConfig {
    #[serde(deserialize_with = "serde_string_or_env")]
    pub private_key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FlashloanConfig {
    pub enabled: bool,
}

/// Deserialize a string that can either be a literal value or an environment variable reference
pub fn serde_string_or_env<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value_or_env = String::deserialize(deserializer)?;
    let value = match value_or_env.chars().next() {
        Some('$') => {
            let env_var_name = &value_or_env[1..];
            env::var(env_var_name).map_err(|_| {
                serde::de::Error::custom(format!(
                    "Environment variable '{}' is not set",
                    env_var_name
                ))
            })?
        }
        _ => value_or_env,
    };
    Ok(value)
}

impl Config {
    /// Load and validate configuration from a TOML file
    pub fn load(path: &str) -> BotResult<Self> {
        let mut file = File::open(path).map_err(|e| {
            BotError::ConfigError(format!("Cannot open config file '{}': {}", path, e))
        })?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| {
            BotError::ConfigError(format!("Cannot read config file '{}': {}", path, e))
        })?;

        let config: Config = toml::from_str(&contents)?;

        // Validate the loaded configuration
        config.validate()?;

        Ok(config)
    }

    /// Comprehensive configuration validation
    fn validate(&self) -> BotResult<()> {
        // Validate bot configuration
        self.validate_bot_config()?;

        // Validate routing configuration
        self.validate_routing_config()?;

        // Validate RPC configuration
        self.validate_rpc_config()?;

        // Validate spam configuration
        if let Some(spam_config) = &self.spam {
            self.validate_spam_config(spam_config)?;
        }

        // Validate wallet configuration
        self.validate_wallet_config()?;

        Ok(())
    }

    fn validate_bot_config(&self) -> BotResult<()> {
        // Validate compute unit limit is reasonable
        if self.bot.compute_unit_limit == 0 {
            return Err(BotError::ConfigError(
                "compute_unit_limit must be greater than 0".to_string(),
            ));
        }

        if self.bot.compute_unit_limit > 1_400_000 {
            return Err(BotError::ConfigError(format!(
                "compute_unit_limit {} exceeds Solana's maximum of 1,400,000",
                self.bot.compute_unit_limit
            )));
        }

        Ok(())
    }

    fn validate_routing_config(&self) -> BotResult<()> {
        if self.routing.mint_config_list.is_empty() {
            return Err(BotError::ConfigError(
                "mint_config_list cannot be empty".to_string(),
            ));
        }

        for (index, mint_config) in self.routing.mint_config_list.iter().enumerate() {
            // Validate mint address is a valid Pubkey
            Pubkey::from_str(&mint_config.mint).map_err(|e| BotError::InvalidPublicKey {
                key: mint_config.mint.clone(),
                source: e,
            })?;

            // Check that at least one pool list is provided
            let has_pools = mint_config.raydium_pool_list.is_some()
                || mint_config.raydium_cp_pool_list.is_some()
                || mint_config.raydium_clmm_pool_list.is_some()
                || mint_config.meteora_dlmm_pool_list.is_some()
                || mint_config.meteora_damm_pool_list.is_some()
                || mint_config.meteora_damm_v2_pool_list.is_some()
                || mint_config.pump_pool_list.is_some()
                || mint_config.whirlpool_pool_list.is_some()
                || mint_config.solfi_pool_list.is_some()
                || mint_config.vertigo_pool_list.is_some();

            if !has_pools {
                return Err(BotError::ConfigError(format!(
                    "mint_config[{}] for mint '{}' has no pool lists configured",
                    index, mint_config.mint
                )));
            }

            // Validate all pool addresses are valid Pubkeys
            self.validate_pool_addresses("raydium_pool_list", &mint_config.raydium_pool_list)?;
            self.validate_pool_addresses("raydium_cp_pool_list", &mint_config.raydium_cp_pool_list)?;
            self.validate_pool_addresses("raydium_clmm_pool_list", &mint_config.raydium_clmm_pool_list)?;
            self.validate_pool_addresses("meteora_dlmm_pool_list", &mint_config.meteora_dlmm_pool_list)?;
            self.validate_pool_addresses("meteora_damm_pool_list", &mint_config.meteora_damm_pool_list)?;
            self.validate_pool_addresses("meteora_damm_v2_pool_list", &mint_config.meteora_damm_v2_pool_list)?;
            self.validate_pool_addresses("pump_pool_list", &mint_config.pump_pool_list)?;
            self.validate_pool_addresses("whirlpool_pool_list", &mint_config.whirlpool_pool_list)?;
            self.validate_pool_addresses("solfi_pool_list", &mint_config.solfi_pool_list)?;
            self.validate_pool_addresses("vertigo_pool_list", &mint_config.vertigo_pool_list)?;

            // Validate lookup table addresses
            if let Some(lookup_tables) = &mint_config.lookup_table_accounts {
                for addr in lookup_tables {
                    Pubkey::from_str(addr).map_err(|e| BotError::InvalidPublicKey {
                        key: addr.clone(),
                        source: e,
                    })?;
                }
            }
        }

        Ok(())
    }

    fn validate_pool_addresses(&self, list_name: &str, pool_list: &Option<Vec<String>>) -> BotResult<()> {
        if let Some(pools) = pool_list {
            for addr in pools {
                Pubkey::from_str(addr).map_err(|e| BotError::ConfigError(format!(
                    "Invalid pool address in {}: {} (error: {})",
                    list_name, addr, e
                )))?;
            }
        }
        Ok(())
    }

    fn validate_rpc_config(&self) -> BotResult<()> {
        // Validate RPC URL format
        if self.rpc.url.is_empty() {
            return Err(BotError::ConfigError(
                "RPC URL cannot be empty".to_string(),
            ));
        }

        // Basic URL validation
        if !self.rpc.url.starts_with("http://") && !self.rpc.url.starts_with("https://") {
            return Err(BotError::ConfigError(format!(
                "RPC URL must start with http:// or https://, got: {}",
                self.rpc.url
            )));
        }

        Ok(())
    }

    fn validate_spam_config(&self, spam_config: &SpamConfig) -> BotResult<()> {
        if spam_config.enabled && spam_config.sending_rpc_urls.is_empty() {
            return Err(BotError::ConfigError(
                "spam.enabled is true but sending_rpc_urls is empty".to_string(),
            ));
        }

        // Validate all sending RPC URLs
        for url in &spam_config.sending_rpc_urls {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(BotError::ConfigError(format!(
                    "Sending RPC URL must start with http:// or https://, got: {}",
                    url
                )));
            }
        }

        Ok(())
    }

    fn validate_wallet_config(&self) -> BotResult<()> {
        if self.wallet.private_key.is_empty() {
            return Err(BotError::ConfigError(
                "wallet.private_key cannot be empty".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation_empty_mint_list() {
        let config = Config {
            bot: BotConfig {
                compute_unit_limit: 100_000,
            },
            routing: RoutingConfig {
                mint_config_list: vec![],
            },
            rpc: RpcConfig {
                url: "https://api.mainnet-beta.solana.com".to_string(),
            },
            spam: None,
            wallet: WalletConfig {
                private_key: "test".to_string(),
            },
            flashloan: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_invalid_compute_limit() {
        let config = Config {
            bot: BotConfig {
                compute_unit_limit: 0,
            },
            routing: RoutingConfig {
                mint_config_list: vec![],
            },
            rpc: RpcConfig {
                url: "https://api.mainnet-beta.solana.com".to_string(),
            },
            spam: None,
            wallet: WalletConfig {
                private_key: "test".to_string(),
            },
            flashloan: None,
        };

        assert!(config.validate().is_err());
    }
}
