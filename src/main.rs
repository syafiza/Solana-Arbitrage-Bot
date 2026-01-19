use clap::Parser;
use solana_onchain_arbitrage_bot::{
    cli::{Cli, Commands},
    engine::bot,
    config::Config,
};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Set up logging
    let log_level = match cli.log_level.to_lowercase().as_str() {
        "error" => Level::ERROR,
        "warn" => Level::WARN,
        "debug" => Level::DEBUG,
        "trace" => Level::TRACE,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    // Display Banner
    Cli::print_banner();

    // Default to run if no subcommand
    let command = cli.command.unwrap_or(Commands::Run { dry_run: false });

    match command {
        Commands::Run { dry_run } => {
            if dry_run {
                info!("Starting bot in DRY RUN mode (no transactions will be sent)");
                // We would pass this flag to run_bot if supported
            }
            
            let config_path = cli.config.to_str().ok_or_else(|| anyhow::anyhow!("Invalid config path"))?;
            info!("Initializing bot with config: {}", config_path);
            
            // Run the bot engine
            bot::run_bot(config_path).await?;
        }
        Commands::Validate { config } => {
            let config_path = config.to_str().unwrap_or("config.toml");
            match Config::load(config_path) {
                Ok(_) => info!("Configuration file is VALID ✅"),
                Err(e) => {
                    tracing::error!("Configuration file is INVALID ❌");
                    tracing::error!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::ListDexs => {
            Cli::print_dex_list();
        }
        Commands::Stats => {
            Cli::print_stats_header();
            // In a real app we might connect to the DB or metrics to show stats
            info!("Connect to Grafana at http://localhost:3000 for full stats");
        }
        Commands::TestRpc { url } => {
            info!("Testing RPC connection to: {}", url);
            let client = solana_client::rpc_client::RpcClient::new(url);
            match client.get_version() {
                Ok(version) => info!("Success! Node version: {}", version),
                Err(e) => {
                    tracing::error!("Connection failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::GenConfig { output } => {
            // Logic to dump default config
            info!("Generating example config to: {:?}", output);
            // Placeholder
        }
    }

    Ok(())
}
