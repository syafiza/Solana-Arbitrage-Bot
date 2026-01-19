/// Central constants module for the Solana Arbitrage Bot
/// 
/// All magic values, program IDs, and default configurations are defined here
/// for easy maintenance and environment-specific overrides.

use lazy_static::lazy_static;
use solana_program::pubkey::Pubkey;
use std::str::FromStr;

// ============================================================================
// Token Mints
// ============================================================================

pub const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
pub const TOKEN_2022_PROGRAM: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

// ============================================================================
// Executor Program (MEV Bot)
// ============================================================================

pub const EXECUTOR_PROGRAM_ID: &str = "MEViEnscUm6tsQRoGd9h6nLQaQspKj7DB2M5FwM3Xvz";
pub const FEE_COLLECTOR: &str = "6AGB9kqgSp2mQXwYpdrV4QVV8urvCaDS35U1wsLssy6H";

// ============================================================================
// Pump.fun Program Constants
// ============================================================================

pub const PUMP_GLOBAL_CONFIG: &str = "ADyA8hdefvWN2dbGGWFotbzWxrAvLW83WG6QCVXvJKqw";
pub const PUMP_AUTHORITY: &str = "GS4CU59F31iL7aR2Q8zVS8DRrcRnXX1yjQ66TqNVQnaR";

// ============================================================================
// Kamino Flashloan Constants
// ============================================================================

pub const KAMINO_LENDING_PROGRAM: &str = "5LFpzqgsxrSfhKwbaFiAEJ2kbc9QyimjKueswsyU4T3o";

// ============================================================================
// System Programs
// ============================================================================

pub const SYSVAR_INSTRUCTIONS: &str = "Sysvar1nstructions1111111111111111111111111";

// ============================================================================
// Default Configuration Values
// ============================================================================

pub const DEFAULT_COMPUTE_UNIT_PRICE: u64 = 1_000;
pub const DEFAULT_MAX_RETRIES: u64 = 3;
pub const DEFAULT_BLOCKHASH_REFRESH_INTERVAL_SECS: u64 = 10;
pub const DEFAULT_PROCESS_DELAY_MS: u64 = 100;
pub const DEFAULT_COMPUTE_UNIT_LIMIT: u32 = 1_400_000;

// Randomization range for compute unit limit (makes transactions unique)
pub const COMPUTE_UNIT_RANDOMIZATION_RANGE: u32 = 1_000;

// ============================================================================
// Transaction Configuration
// ============================================================================

pub const MINIMUM_PROFIT_DEFAULT: u64 = 0;
pub const NO_FAILURE_MODE_DEFAULT: bool = false;

// ATA creation compute limits
pub const ATA_CREATION_COMPUTE_UNIT_PRICE: u64 = 1_000_000;
pub const ATA_CREATION_COMPUTE_UNIT_LIMIT: u32 = 60_000;

// ============================================================================
// Retry Configuration
// ============================================================================

pub const MAX_RPC_RETRIES: u32 = 5;
pub const RETRY_INITIAL_BACKOFF_MS: u64 = 100;
pub const RETRY_MAX_BACKOFF_MS: u64 = 5_000;
pub const RETRY_BACKOFF_MULTIPLIER: f64 = 2.0;

// ============================================================================
// Lookup Tables
// ============================================================================

/// Default lookup table for common accounts
pub const DEFAULT_LOOKUP_TABLE: &str = "4sKLJ1Qoudh8PJyqBeuKocYdsZvxTcRShUt9aKqwhgvC";

// ============================================================================
// Lazy-initialized Pubkeys (computed once)
// ============================================================================

lazy_static! {
    pub static ref SOL_MINT_PUBKEY: Pubkey = 
        Pubkey::from_str(SOL_MINT).expect("Invalid SOL mint address");
    
    pub static ref TOKEN_2022_PROGRAM_PUBKEY: Pubkey = 
        Pubkey::from_str(TOKEN_2022_PROGRAM).expect("Invalid Token 2022 program");
    
    pub static ref EXECUTOR_PROGRAM_PUBKEY: Pubkey = 
        Pubkey::from_str(EXECUTOR_PROGRAM_ID).expect("Invalid executor program ID");
    
    pub static ref FEE_COLLECTOR_PUBKEY: Pubkey = 
        Pubkey::from_str(FEE_COLLECTOR).expect("Invalid fee collector address");
    
    pub static ref PUMP_GLOBAL_CONFIG_PUBKEY: Pubkey = 
        Pubkey::from_str(PUMP_GLOBAL_CONFIG).expect("Invalid pump global config");
    
    pub static ref PUMP_AUTHORITY_PUBKEY: Pubkey = 
        Pubkey::from_str(PUMP_AUTHORITY).expect("Invalid pump authority");
    
    pub static ref KAMINO_LENDING_PROGRAM_PUBKEY: Pubkey = 
        Pubkey::from_str(KAMINO_LENDING_PROGRAM).expect("Invalid Kamino program");
    
    pub static ref SYSVAR_INSTRUCTIONS_PUBKEY: Pubkey = 
        Pubkey::from_str(SYSVAR_INSTRUCTIONS).expect("Invalid sysvar instructions");
    
    pub static ref DEFAULT_LOOKUP_TABLE_PUBKEY: Pubkey = 
        Pubkey::from_str(DEFAULT_LOOKUP_TABLE).expect("Invalid default lookup table");
}

// ============================================================================
// Helper Functions (for backward compatibility)
// ============================================================================

/// Returns the SOL mint Pubkey
/// 
/// # Note
/// Prefer using `SOL_MINT_PUBKEY` directly for better performance
pub fn sol_mint() -> Pubkey {
    *SOL_MINT_PUBKEY
}

/// Returns the Token 2022 program Pubkey
pub fn token_2022_program() -> Pubkey {
    *TOKEN_2022_PROGRAM_PUBKEY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_pubkeys_valid() {
        // This test ensures all string constants parse to valid Pubkeys
        assert_eq!(sol_mint().to_string(), SOL_MINT);
        assert_eq!(EXECUTOR_PROGRAM_PUBKEY.to_string(), EXECUTOR_PROGRAM_ID);
        assert_eq!(FEE_COLLECTOR_PUBKEY.to_string(), FEE_COLLECTOR);
    }

    #[test]
    fn test_default_values() {
        assert_eq!(DEFAULT_COMPUTE_UNIT_PRICE, 1_000);
        assert_eq!(DEFAULT_MAX_RETRIES, 3);
        assert!(DEFAULT_COMPUTE_UNIT_LIMIT > 0);
    }
}
