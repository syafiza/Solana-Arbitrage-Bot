// Core Modules
pub mod common {
    pub mod constants;
    pub mod error;
}
pub mod configuration {
    pub mod config;
    pub mod secrets;
}
pub mod engine {
    pub mod bot;
    pub mod refresh;
}
pub mod state {
    pub mod pools;
}
pub mod storage {
    pub mod database;
}
pub mod execution {
    pub mod transaction;
    pub mod jito;
}
pub mod monitoring {
    pub mod metrics;
    pub mod health;
    pub mod latency;
}

// Flat modules (unchanged)
pub mod cli;
pub mod dex;
pub mod pool;
pub mod rpc;

// Re-exports for easier access / compatibility
pub use common::{constants, error};
pub use configuration::{config, secrets};
pub use engine::{bot, refresh};
pub use state::pools;
pub use storage::database;
pub use execution::{transaction, jito};
pub use monitoring::{metrics, health, latency};

