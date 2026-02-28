# Hyperliquid Supurr Skill

> Build custom trading strategies in Rust, backtest, deploy, and monitor bots on Hyperliquid. Supports Grid, DCA, Spot-Perp Arbitrage, and custom user-authored strategies across Native Perps, Spot markets (USDC/USDH), and HIP-3 sub-DEXes.

## Installation

### Option 1: skills.sh (Recommended)

```bash
npx skills add Supurr-App/Hyperliquid-Supurr-Skill
```

Supports Claude Code, Cursor, OpenCode, Antigravity, and more.

### Option 2: Direct Install

```bash
# Install AI skill to your tools
curl -fsSL https://cli.supurr.app/skill-install | bash
```

## Also Install: Supurr CLI

The skill teaches AI assistants to use the CLI. The skill installer automatically installs the CLI, but you can also install it manually:

```bash
curl -fsSL https://cli.supurr.app/install | bash
```

## Available Skills

| Skill                                | Description                                                                   |
| ------------------------------------ | ----------------------------------------------------------------------------- |
| [SKILL.md](./SKILL.md)               | Complete CLI reference for backtesting, deploying, and monitoring bots        |
| [STRATEGY_API.md](./STRATEGY_API.md) | Strategy authoring API — build custom Rust strategies with the Strategy trait |

## Quick Start

```bash
# 1. Install CLI
curl -fsSL https://cli.supurr.app/install | bash

# 2. Setup credentials
supurr init --address 0x... --api-wallet 0x...

# 3. Create config
supurr new grid --asset BTC --levels 4 --start-price 88000 --end-price 92000

# 4. Backtest
supurr backtest -c config.json -s 2026-01-28 -e 2026-02-01

# 5. Deploy
supurr deploy -c config.json

# 6. Monitor
supurr monitor --watch
```

## Repository Structure

```
supurr-skill/
├── .claude-plugin/
│   └── plugin.json               # Claude plugin manifest
├── scripts/
│   ├── install.sh                # CLI binary installer
│   └── skill-install.sh          # AI skill installer
├── references/
│   ├── strategy-trait.md         # Strategy + StrategyContext API
│   ├── commands.md               # PlaceOrder, CancelOrder, CancelAll
│   ├── events.md                 # Event enum (Quote, OrderFilled, etc.)
│   ├── types.md                  # Core types (Price, Qty, Market, etc.)
│   ├── hl-info-api.md            # Hyperliquid Info API reference
│   ├── arb-spot-resolution.md    # Spot token U-prefix resolution
│   ├── workflows.md              # End-to-end CLI workflows
│   └── troubleshooting.md        # Common errors and fixes
├── templates/
│   └── strategy-template/        # Scaffold crate for new strategies
├── examples/
│   └── strategy-simple/          # Working buy-low-sell-high example
├── tutorials/
│   ├── grid.md                   # Grid bot tutorial
│   ├── arb.md                    # Spot-perp arb tutorial
│   ├── dca.md                    # DCA tutorial
│   └── custom-strategy.md        # Build your own strategy tutorial
├── SKILL.md                      # Main skill documentation (CLI + Strategy authoring)
├── STRATEGY_API.md               # Strategy authoring API contract
└── README.md                     # This file
```

## License

MIT
