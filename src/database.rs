/// Database Module
/// 
/// Handles logging of historical trades and opportunities to SQLite.
/// Critical for strategy backtesting and performance analysis.

use crate::error::BotResult;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(database_url: &str) -> BotResult<Self> {
        info!("Connecting to database: {}", database_url);
        
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(|e| crate::error::BotError::ConfigError(format!("Database connection failed: {}", e)))?;

        // Initialize schema
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS trades (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                mint TEXT NOT NULL,
                profit_lamports INTEGER NOT NULL,
                signature TEXT NOT NULL,
                dexes TEXT NOT NULL,
                input_amount INTEGER NOT NULL,
                output_amount INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_trades_timestamp ON trades(timestamp);
            CREATE INDEX IF NOT EXISTS idx_trades_mint ON trades(mint);
            "#
        )
        .execute(&pool)
        .await
        .map_err(|e| crate::error::BotError::ConfigError(format!("Schema init failed: {}", e)))?;

        Ok(Self { pool })
    }

    pub async fn log_trade(
        &self,
        mint: &str,
        profit: u64,
        signature: &str,
        dexes: &[String],
        input: u64,
        output: u64,
    ) -> BotResult<()> {
        let timestamp = chrono::Utc::now().timestamp();
        let dexes_str = dexes.join(",");

        sqlx::query(
            r#"
            INSERT INTO trades (timestamp, mint, profit_lamports, signature, dexes, input_amount, output_amount)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(timestamp)
        .bind(mint)
        .bind(profit as i64)
        .bind(signature)
        .bind(dexes_str)
        .bind(input as i64)
        .bind(output as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| crate::error::BotError::TransactionError(format!("Failed to log trade: {}", e)))?;

        Ok(())
    }

    pub async fn get_total_profit(&self) -> BotResult<u64> {
        let result: (i64,) = sqlx::query_as("SELECT COALESCE(SUM(profit_lamports), 0) FROM trades")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| crate::error::BotError::TransactionError(format!("Failed to fetch profit: {}", e)))?;

        Ok(result.0 as u64)
    }
}
