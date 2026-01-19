/// Comprehensive error types for the Solana Arbitrage Bot
/// 
/// This module provides a type-safe error hierarchy using thiserror,
/// enabling better error handling, debugging, and monitoring.

use solana_sdk::pubkey::Pubkey;
use thiserror::Error;

/// Main error type for the bot
#[derive(Debug, Error)]
pub enum BotError {
    /// Pool initialization errors
    #[error("Failed to initialize {dex} pool at {pool_address}: {source}")]
    PoolInitialization {
        dex: String,
        pool_address: String,
        #[source]
        source: anyhow::Error,
    },

    /// RPC communication errors
    #[error("RPC error from {endpoint}: {message}")]
    RpcError {
        endpoint: String,
        message: String,
        retryable: bool,
    },

    /// Configuration errors
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Invalid public key errors
    #[error("Invalid public key '{key}': {source}")]
    InvalidPublicKey {
        key: String,
        #[source]
        source: solana_sdk::pubkey::ParsePubkeyError,
    },

    /// Account fetch errors
    #[error("Failed to fetch account {address}: {reason}")]
    AccountFetchError { address: Pubkey, reason: String },

    /// Account ownership verification errors
    #[error("Account {address} is owned by {actual_owner}, expected {expected_owner}")]
    InvalidAccountOwner {
        address: Pubkey,
        expected_owner: Pubkey,
        actual_owner: Pubkey,
    },

    /// Pool validation errors
    #[error("Invalid pool configuration: {0}")]
    PoolValidationError(String),

    /// Transaction building errors
    #[error("Failed to build transaction: {0}")]
    TransactionBuildError(String),

    /// Transaction sending errors
    #[error("Failed to send transaction: {0}")]
    TransactionSendError(String),

    /// Wallet errors
    #[error("Wallet error: {0}")]
    WalletError(String),

    /// Deserialization errors
    #[error("Failed to deserialize {data_type}: {source}")]
    DeserializationError {
        data_type: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// IO errors
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// TOML parsing errors
    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),

    /// Generic Solana client errors
    #[error("Solana client error: {0}")]
    SolanaClientError(#[from] solana_client::client_error::ClientError),

    /// Anyhow errors (for gradual migration)
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl BotError {
    /// Check if this error is retryable (for automatic retry logic)
    pub fn is_retryable(&self) -> bool {
        match self {
            BotError::RpcError { retryable, .. } => *retryable,
            BotError::SolanaClientError(_) => true,
            BotError::AccountFetchError { .. } => true,
            BotError::TransactionSendError(_) => true,
            _ => false,
        }
    }

    /// Get error severity for logging and alerting
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            BotError::ConfigError(_) => ErrorSeverity::Critical,
            BotError::WalletError(_) => ErrorSeverity::Critical,
            BotError::InvalidPublicKey { .. } => ErrorSeverity::Error,
            BotError::PoolInitialization { .. } => ErrorSeverity::Warning,
            BotError::RpcError { .. } => ErrorSeverity::Warning,
            BotError::TransactionSendError(_) => ErrorSeverity::Info,
            _ => ErrorSeverity::Error,
        }
    }

    /// Create a retryable RPC error
    pub fn rpc_retryable(endpoint: String, message: String) -> Self {
        BotError::RpcError {
            endpoint,
            message,
            retryable: true,
        }
    }

    /// Create a non-retryable RPC error
    pub fn rpc_fatal(endpoint: String, message: String) -> Self {
        BotError::RpcError {
            endpoint,
            message,
            retryable: false,
        }
    }
}

/// Error severity levels for monitoring and alerting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Critical errors that require immediate attention
    Critical,
    /// Errors that indicate problems but are recoverable
    Error,
    /// Warnings about potential issues
    Warning,
    /// Informational messages about expected failures
    Info,
}

impl ErrorSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorSeverity::Critical => "CRITICAL",
            ErrorSeverity::Error => "ERROR",
            ErrorSeverity::Warning => "WARNING",
            ErrorSeverity::Info => "INFO",
        }
    }
}

/// Result type alias for bot operations
pub type BotResult<T> = Result<T, BotError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_retryable() {
        let retryable = BotError::rpc_retryable(
            "http://test".to_string(),
            "timeout".to_string(),
        );
        assert!(retryable.is_retryable());

        let fatal = BotError::rpc_fatal(
            "http://test".to_string(),
            "invalid endpoint".to_string(),
        );
        assert!(!fatal.is_retryable());
    }

    #[test]
    fn test_error_severity() {
        let config_err = BotError::ConfigError("test".to_string());
        assert_eq!(config_err.severity(), ErrorSeverity::Critical);

        let rpc_err = BotError::rpc_retryable(
            "http://test".to_string(),
            "test".to_string(),
        );
        assert_eq!(rpc_err.severity(), ErrorSeverity::Warning);
    }
}
