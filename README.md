# Solana Arbitrage Bot 🚀

[![Build Status](https://github.com/syafiza/Solana-Arbitrage-Bot/workflows/Rust%20CI%2FCD/badge.svg)](https://github.com/syafiza/Solana-Arbitrage-Bot/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

**World-class, production-ready Solana arbitrage bot with comprehensive monitoring and testing.**

## ✨ Features

- 🔥 **Multi-DEX Support** - 10 DEXs: Raydium (3 types), Pump, Whirlpool, Meteora (3 types), Solfi, Vertigo
- ⚡ **High Performance** - RPC connection pooling, TTL caching, circuit breaker pattern
- 🛡️ **Production-Ready** - Zero unwraps, comprehensive error handling, graceful shutdown
- 📊 **Full Observability** - Prometheus metrics, health checks, structured logging
- 🧪 **Thoroughly Tested** - Integration tests, property-based tests, benchmarks
- 🚀 **Easy Deployment** - Docker, Kubernetes ready, CI/CD automated

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+
- Solana CLI tools
- RPC endpoint (Quicknode, Helius, or local validator)

### Installation

```bash
git clone https://github.com/syafiza/Solana-Arbitrage-Bot.git
cd Solana-Arbitrage-Bot
cargo build --release
```

### Configuration

Create `config.toml`:

```toml
[bot]
compute_unit_limit = 1_400_000

[rpc]
url = "https://api.mainnet-beta.solana.com"

[wallet]
private_key = "$PRIVATE_KEY"  # Or path to keypair file

[[routing.mint_config_list]]
mint = "So11111111111111111111111111111111111111112"
process_delay = 100

# Add DEX pools...
raydium_pool_list = ["PoolAddress1", "PoolAddress2"]
```

### Run

```bash
# Development
cargo run -- --config config.toml

# Production
./target/release/solana-arbitrage-bot --config config.toml
```

## 🐳 Docker Deployment

### Using Docker Compose (Recommended)

```bash
# Set environment variables
export SOLANA_RPC_URL="your-rpc-url"
export WALLET_PRIVATE_KEY="your-key"

# Start the bot + monitoring stack
docker-compose up -d

# View logs
docker-compose logs -f arbitrage-bot

# Stop
docker-compose down
```

### Manual Docker Build

```bash
docker build -t solana-arbitrage-bot .
docker run -d \
  -v $(pwd)/config.toml:/app/config.toml:ro \
  -p 8080:8080 \
  --name arbitrage-bot \
  solana-arbitrage-bot
```

## 📊 Monitoring

### Health Checks

```bash
# Health status
curl http://localhost:8080/health

# Readiness probe
curl http://localhost:8080/ready

# Prometheus metrics
curl http://localhost:8080/metrics
```

### Grafana Dashboards

Access Grafana at `http://localhost:3000` (default: admin/admin)

Pre-configured dashboards track:
- RPC request rates and failures
- Cache hit rates
- Transaction success rates
- Arbitrage opportunities found/executed
- Profit tracking

## 🧪 Testing

```bash
# All tests
cargo test

# Integration tests only
cargo test --test '*'

# Property-based tests
cargo test --test property_tests

# Benchmarks
cargo bench
```

## 📈 Architecture

```
┌─────────────────────────────────────────────┐
│         Solana Arbitrage Bot                │
├─────────────────────────────────────────────┤
│  Health Server  │  Metrics  │  Shutdown     │
├─────────────────┴───────────┴───────────────┤
│              Bot Core Logic                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ Raydium  │  │  Pump    │  │Whirlpool │  │
│  │ CPMM/CP/ │  │  .fun    │  │  (Orca)  │  │
│  │   CLMM   │  │          │  │          │  │
│  └──────────┘  └──────────┘  └──────────┘  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ Meteora  │  │  Solfi   │  │ Vertigo  │  │
│  │DLMM/DAMM │  │          │  │          │  │
│  └──────────┘  └──────────┘  └──────────┘  │
├─────────────────────────────────────────────┤
│         RPC Pool + Circuit Breaker          │
│   (Caching, Retry, Load Balancing)          │
└─────────────────────────────────────────────┘
```

## 🛠️ Development

### Project Structure

```
src/
├── error.rs          # Error type hierarchy
├── constants.rs      # Centralized constants
├── config.rs         # Configuration with validation
├── bot.rs            # Main bot logic
├── health.rs         # Health checks & shutdown
├── metrics.rs        # Performance metrics
├── dex/
│   ├── traits.rs     # DEX abstraction traits
│   ├── raydium/      # Raydium initializers
│   ├── pump/         # Pump initializer
│   ├── whirlpool/    # Whirlpool initializer
│   ├── meteora/      # Meteora initializers
│   ├── solfi/        # Solfi initializer
│   └── vertigo/      # Vertigo initializer
├── rpc/
│   ├── pool.rs       # Connection pool
│   └── mock.rs       # Mock for testing
└── pool/
    └── object_pool.rs # Memory optimization

tests/
├── integration_tests.rs  # Integration tests
└── property_tests.rs     # Property-based tests

benches/
└── pool_benchmarks.rs    # Performance benchmarks
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings

# Check coverage
cargo tarpaulin --out Html
```

## 🔒 Security

- **Non-root Docker user** - Runs as `botuser` (UID 1000)
- **No unwrap() calls** - All errors handled gracefully
- **Input validation** - Config validated at load time
- **Rate limiting** - Circuit breaker prevents RPC abuse

## 📊 Performance

- **78% code reduction** - Optimized from 930 to ~200 lines in core module
- **RPC caching** - Reduces redundant requests
- **Connection pooling** - Efficient resource usage
- **Object pooling** - Minimized allocations in hot paths

## 🤝 Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new features
4. Ensure CI passes
5. Submit a pull request

## 📄 License

MIT License - see [LICENSE](LICENSE) file

## 🙏 Acknowledgments

Built with world-class Rust patterns inspired by production systems at scale.

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/syafiza/Solana-Arbitrage-Bot/issues)
- **Discussions**: [GitHub Discussions](https://github.com/syafiza/Solana-Arbitrage-Bot/discussions)

---

**⚠️ Disclaimer:** Use at your own risk. Always test thoroughly on devnet before mainnet deployment.
