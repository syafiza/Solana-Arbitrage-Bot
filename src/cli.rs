/// Interactive CLI Interface
/// 
/// Provides an interactive command-line interface for bot management.

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "solana-arbitrage-bot")]
#[command(author = "Solana Arbitrage Team")]
#[command(version = "1.0.0")]
#[command(about = "Enterprise-grade Solana arbitrage bot", long_about = None)]
pub struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE", default_value = "config.toml")]
    pub config: PathBuf,

    /// Log level (error, warn, info, debug, trace)
    #[arg(short, long, default_value = "info")]
    pub log_level: String,

    /// Health check server port
    #[arg(long, default_value = "8080")]
    pub health_port: u16,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run the arbitrage bot
    Run {
        /// Enable dry-run mode (no actual transactions)
        #[arg(long)]
        dry_run: bool,
    },

    /// Validate configuration file
    Validate {
        /// Configuration file to validate
        #[arg(value_name = "FILE")]
        config: PathBuf,
    },

    /// Show bot statistics
    Stats,

    /// Test RPC connection
    TestRpc {
        /// RPC URL to test
        url: String,
    },

    /// List supported DEXs
    ListDexs,

    /// Generate example configuration
    GenConfig {
        /// Output file path
        #[arg(short, long, default_value = "config.example.toml")]
        output: PathBuf,
    },
}

impl Cli {
    pub fn print_banner() {
        println!("{}", "=".repeat(60).bright_cyan());
        println!("{}", "   Solana Arbitrage Bot v1.0.0".bright_green().bold());
        println!("{}", "   Enterprise-Grade Trading System".bright_white());
        println!("{}", "=".repeat(60).bright_cyan());
        println!();
    }

    pub fn print_dex_list() {
        println!("{}", "Supported DEXs:".bright_yellow().bold());
        println!();
        
        let dexs = vec![
            ("Raydium CPMM", "✓", "Standard AMM"),
            ("Raydium CP", "✓", "Constant Product"),
            ("Raydium CLMM", "✓", "Concentrated Liquidity"),
            ("Pump.fun", "✓", "Bonding Curve"),
            ("Orca Whirlpool", "✓", "Concentrated Liquidity"),
            ("Meteora DLMM", "✓", "Dynamic Liquidity"),
            ("Meteora DAMM", "✓", "Dynamic AMM"),
            ("Meteora DAMM V2", "✓", "Dynamic AMM V2"),
            ("Solfi", "✓", "Standard AMM"),
            ("Vertigo", "✓", "Standard AMM"),
        ];

        for (name, status, description) in dexs {
            println!("  {} {} - {}", 
                status.bright_green(),
                name.bright_white().bold(),
                description.bright_black()
            );
        }
        println!();
    }

    pub fn print_stats_header() {
        println!("{}", "Bot Performance Metrics".bright_yellow().bold());
        println!("{}", "-".repeat(60).bright_black());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let cli = Cli::parse_from(["bot", "--config", "test.toml"]);
        assert_eq!(cli.config, PathBuf::from("test.toml"));
    }
}
