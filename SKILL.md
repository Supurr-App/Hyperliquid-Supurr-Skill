---
name: hyperliquid-supurr
description: Build, backtest, paper trade, deploy, monitor, stop, and copy trading bots on Hyperliquid. Author custom strategies in Rust, or use built-in Grid, DCA, and Spot-Perp Arbitrage strategies across Native Perps, Spot markets (USDC/USDE/USDT0/USDH), HIP-3 sub-DEXes, and HIP-4 prediction markets.
---

# Hard Rules

## Hyperliquid-explorer

- For Hyperliquid user-state endpoints that accept `dex`, always pass `"dex": "ALL_DEXS"` when fetching aggregate state across native perps and HIP-3/sub-DEXes.
- This applies before claiming current open orders, positions/holdings, funding, or related live user state. HIP-3 markets live on other DEXes, so a plain call can miss real state.
- Do not enumerate DEX names just to aggregate state. Use one `ALL_DEXS` request unless the user explicitly asks for one specific DEX.

```bash
curl -sS -X POST https://api.hyperliquid.xyz/info \
  -H 'content-type: application/json' \
  --data '{"type":"openOrders","user":"0x...","dex":"ALL_DEXS"}'
```

## Bot

- After stopping a bot, do not assume all exchange orders were cancelled.
- Always fetch live open orders from Hyperliquid once after stop, using `"dex": "ALL_DEXS"`.
- If any bot-related orders remain live, manually cancel them and verify open orders again.

---

# Hyperliquid Supurr Skill — Complete Command Reference

> **For LLMs**: This is the authoritative reference. Use exact syntax. Config files are in `~/.supurr/configs/`.

---

## Quick Reference

| Command                | Purpose                         |
| ---------------------- | ------------------------------- |
| `supurr init`          | Setup wallet credentials        |
| `supurr whoami`        | Show current wallet             |
| `supurr new grid`      | Generate grid strategy config   |
| `supurr new arb`       | Generate spot-perp arb config   |
| `supurr new dca`       | Generate DCA strategy config    |
| `supurr new orchestrator` | Generate grouped strategy config |
| `supurr configs`       | List saved configs              |
| `supurr config <name>` | View config details             |
| `supurr backtest`      | Run historical simulation       |
| `supurr paper`         | Paper trade (real quotes, sim fills) |
| `supurr deploy`        | Deploy bot to production        |
| `supurr monitor`       | View active bots                |
| `supurr history`       | View historical bot sessions    |
| `supurr stop`          | Stop a running bot (signed)     |
| `supurr analytics`     | List analytics data commands    |
| `supurr analytics prices` | Fetch/store Oracle OHLCV candles |
| `supurr analytics funding-rates` | Fetch/store funding rates |
| `supurr prices`        | Legacy alias for analytics prices |
| `supurr update`        | Update CLI, skill, and bot source |
| `supurr dev init`      | Clone/update bot source for dev |
| `supurr dev build`     | Build bot from source           |
| `supurr dev run`       | Run dev-built bot               |
| `supurr dev backtest`  | Backtest with dev-built engine  |

---

## Agent Product Pointers

| User asks about | Read before answering | Agent job |
| --- | --- | --- |
| Prediction markets, outcome markets, HIP-4 | [Prediction Markets](tutorials/prediction-markets.md) | Validate live `outcomeMeta`, keep prices inside `0..1`, use `USDH` spot balance, no leverage. |
| Neutral prediction grid, YES/NO grid, neutral outcome board | This file, section 3d | Use `supurr new orchestrator`; model it as two long grids, not one neutral outcome grid. |
| FOMO, opportunities, copyable bots, active bots | [Bot Discovery](references/bot-discovery.md) | Fetch live bot surfaces when relevant and suggest copy candidates only as risk-aware options. |
| Missing funds, wallet connection, transfers, builder approval | [User Action Intents](references/user-action-intents.md) | Emit a machine-readable `user_action_required` object for frontend/Telegram rendering. |
| Historical 1s candles, oracle archive, processed OHLCV data | This file, Supurr Oracle Candle API | Use Supurr Oracle presigned URLs; prefer processed candles over Hyperliquid Info API for historical backtests. |

---

## Global Options

```bash
supurr --help              # Show all commands
supurr --version, -V       # Show CLI version
supurr -d, --debug         # Enable debug logging (any command)
```

---

## 1. `supurr init` — Credential Setup

```bash
# Interactive
supurr init

# Non-interactive
supurr init --address 0x... --api-wallet 0x...

# Overwrite existing
supurr init --force
```

| Option                | Description                    |
| --------------------- | ------------------------------ |
| `-f, --force`         | Overwrite existing credentials |
| `--address <address>` | Wallet address (0x...)         |
| `--api-wallet <key>`  | API wallet private key         |

### Runtime Credential Resolution (v0.3.5+)

The bot binary resolves credentials at **runtime** from `~/.supurr/credentials.json`, not from the strategy config file. This means:

- **Wallet switching is instant** — run `supurr init --address 0x... --api-wallet 0x... --force` to change the active wallet. The running bot resolves the new credentials on next startup.
- **Config files are wallet-agnostic** — strategy configs (grid, arb, dca) no longer contain wallet credentials. The same config can be used across different wallets.
- **Credential priority**: `~/.supurr/credentials.json` > environment variables (`SUPURR_WALLET_ADDRESS`, `SUPURR_API_WALLET_KEY`) > config file fields (legacy, deprecated).

> **For deployed agents**: The `/activate` command in Telegram triggers `supurr init --address ... --api-wallet ... --force` on the remote FastClaw instance, switching the active wallet without redeploying.

---

## 2. `supurr whoami` — Show Identity

```bash
supurr whoami    # Shows: Address + masked key
```

---

## 3. `supurr new <strategy>` — Config Generator

Supports four strategies: `grid`, `arb`, `dca`, `orchestrator`.

```bash
supurr new grid [options]   # Grid trading
supurr new arb [options]    # Spot-perp arbitrage
supurr new dca [options]    # Dollar-cost averaging
supurr new orchestrator [options] # Grouped strategy
```

---

### 3a. `supurr new grid` — Grid Strategy

#### Market Types

| Type      | Quote    | Requires  | Example                                 |
| --------- | -------- | --------- | --------------------------------------- |
| `native`  | USDC     | —         | `--asset BTC`                           |
| `spot`    | Variable | `--quote` | `--asset HYPE --type spot --quote USDC` |
| `hip3`    | Per-DEX  | `--dex`   | `--asset BTC --type hip3 --dex hyna`    |
| `outcome` | USDH     | `--outcome-id` | `--asset BTC --type outcome --outcome-id 2 --side yes` |

#### Grid Options

| Option                  | Default       | Description                                    |
| ----------------------- | ------------- | ---------------------------------------------- |
| `-a, --asset <symbol>`  | `BTC`         | Base asset (BTC, ETH, HYPE, etc.)              |
| `-o, --output <file>`   | `config.json` | Output filename                                |
| `--type <type>`         | `native`      | Market type: native, spot, hip3, outcome       |
| `--dex <dex>`           | —             | **Required for hip3**: DEX slug from `perpDexs` (e.g., hyna, xyz, flx, cash) |
| `--quote <quote>`       | —             | **Required for spot**: USDC, USDE, USDT0, USDH |
| `--outcome-id <id>`     | —             | **Required for outcome**: ID from live `outcomeMeta` |
| `--side <side>`         | `yes`         | Outcome side: `yes`/`no` or `0`/`1`            |
| `--mode <mode>`         | `long`        | Grid mode: long, short, neutral                |
| `--levels <n>`          | `20`          | Number of grid levels                          |
| `--start-price <price>` | —             | Grid start price                               |
| `--end-price <price>`   | —             | Grid end price                                 |
| `--investment <amount>` | `1000`        | Max investment in quote currency               |
| `--leverage <n>`        | `2`           | Leverage (1 for spot)                          |
| `--testnet`             | false         | Use Hyperliquid testnet                        |

#### Grid Examples

```bash
# Native Perp (BTC-USDC)
supurr new grid --asset BTC --levels 4 --start-price 88000 --end-price 92000 --investment 100 --leverage 20

# USDC Spot (HYPE/USDC)
supurr new grid --asset HYPE --type spot --quote USDC --levels 3 --start-price 29 --end-price 32 --investment 100

# Non-USDC Spot (HYPE/USDH)
supurr new grid --asset HYPE --type spot --quote USDH --levels 3 --start-price 29 --end-price 32 --investment 100

# HIP-3 (hyna:BTC)
supurr new grid --asset BTC --type hip3 --dex hyna --levels 4 --start-price 88000 --end-price 92000 --investment 100 --leverage 20

# HIP-4 Outcome (BTC prediction market, Yes side)
supurr new grid --asset BTC --type outcome --outcome-id 2 --side yes --levels 5 --start-price 0.27 --end-price 0.32 --investment 80
```

#### Common HIP-3 DEXes (Current CLI Defaults)

| DEX    | Quote | Assets                              |
| ------ | ----- | ----------------------------------- |
| `hyna` | USDE  | Crypto perps (BTC, ETH, HYPE, etc.) |
| `xyz`  | USDC  | Stocks (AAPL, TSLA, etc.)           |
| `km`   | USDC  | Kinetiq Markets                     |
| `vntl` | USDC  | AI/tech tokens                      |
| `flx`  | USDC  | Additional HIP-3 markets            |
| `cash` | USDC  | Additional HIP-3 markets            |

> `perpDexs` is the authoritative discovery source. The set of DEXes can change over time.

---

### 3b. `supurr new arb` — Spot-Perp Arbitrage Strategy

Generates a config that simultaneously trades the **spot** and **perp** legs of the same asset, capturing spread differentials.

> **Market Constraint**: Only assets that have **both** a spot token AND a perp market on Hyperliquid are eligible. The CLI auto-resolves the spot counterpart.

#### Spot Resolution Logic

Resolution order: try `U{ASSET}` first (e.g., `BTC` → `UBTC`) → fallback to exact name (e.g., `HYPE`, `TRUMP`) → error if neither exists.

> **⚠️ Always pass the perp ticker** (e.g., `BTC`, not `UBTC`). See full table → [references/arb-spot-resolution.md](references/arb-spot-resolution.md)

#### Arb Options

| Option                 | Default            | Description                                  |
| ---------------------- | ------------------ | -------------------------------------------- |
| `-a, --asset <symbol>` | `BTC`              | Perp asset name (BTC, ETH, HYPE, etc.)       |
| `--amount <usdc>`      | `100`              | Order amount in USDC per leg                 |
| `--leverage <n>`       | `1`                | Leverage for perp leg                        |
| `--open-spread <pct>`  | `0.003`            | Min opening spread (0.003 = 0.3%)            |
| `--close-spread <pct>` | `-0.001`           | Min closing spread (-0.001 = -0.1%)          |
| `--slippage <pct>`     | `0.001`            | Slippage buffer for both legs (0.001 = 0.1%) |
| `-o, --output <file>`  | `{asset}-arb.json` | Output filename                              |
| `--testnet`            | false              | Use Hyperliquid testnet                      |

#### Arb Examples

```bash
# BTC spot-perp arb (default $100/leg)
supurr new arb --asset BTC

# HYPE arb with $50 per leg, 2x leverage on perp
supurr new arb --asset HYPE --amount 50 --leverage 2

# ETH arb with tighter spreads
supurr new arb --asset ETH --open-spread 0.002 --close-spread -0.0005 --slippage 0.0005

# SOL arb on testnet
supurr new arb --asset SOL --testnet
```

> **Balance Requirement**: Arb bots require USDC balance in **both** Spot and Perps wallets on Hyperliquid, since the bot trades on both sides simultaneously.

---

### 3c. `supurr new dca` — DCA Strategy

Generates a Dollar-Cost Averaging config that opens positions in steps when price deviates, then takes profit on the averaged entry.

#### DCA Options

| Option                       | Default       | Description                                        |
| ---------------------------- | ------------- | -------------------------------------------------- |
| `-a, --asset <symbol>`       | `BTC`         | Base asset                                         |
| `--mode <mode>`              | `long`        | Direction: long or short                           |
| `--type <type>`              | `native`      | Market type: native, spot, hip3                    |
| `--trigger-price <price>`    | `100000`      | Price to trigger base order                        |
| `--base-order <size>`        | `0.001`       | Base order size in base asset                      |
| `--dca-order <size>`         | `0.001`       | DCA order size in base asset                       |
| `--max-orders <n>`           | `5`           | Max number of DCA orders                           |
| `--size-multiplier <x>`      | `2.0`         | Size multiplier per DCA step                       |
| `--deviation <pct>`          | `1`           | Price deviation % to trigger first DCA (whole percent: 1 = 1%) |
| `--deviation-multiplier <x>` | `1.0`         | Deviation multiplier for subsequent steps          |
| `--take-profit <pct>`        | `2`           | Take profit % from avg entry (whole percent: 2 = 2%) |
| `--stop-loss <pnl>`          | —             | Optional stop loss as absolute PnL threshold       |
| `--leverage <n>`             | `2`           | Leverage (1 for spot)                              |
| `--restart`                  | false         | Restart cycle after take profit                    |
| `--cooldown <secs>`          | `60`          | Cooldown between cycles in seconds                 |
| `-o, --output <file>`        | `config.json` | Output filename                                    |
| `--testnet`                  | false         | Use Hyperliquid testnet                            |

#### DCA Examples

```bash
# BTC DCA long, trigger at $95k
supurr new dca --asset BTC --trigger-price 95000

# ETH DCA short with custom deviation
supurr new dca --asset ETH --mode short --deviation 2

# HYPE DCA with auto-restart
supurr new dca --asset HYPE --restart --cooldown 120 --take-profit 3

# DCA on spot market
supurr new dca --asset HYPE --type spot --quote USDC --trigger-price 25
```

> [!NOTE]
> Outcome DCA is supported by the engine where `outcomeMeta` returns live markets, but `supurr new dca` does not generate outcome configs yet. Use [tutorials/prediction-markets.md](tutorials/prediction-markets.md) for the manual config shape.

---

### 3d. `supurr new orchestrator` — Prediction YES/NO Neutral Grid

Use this when the user asks for a neutral prediction grid, neutral outcome board, YES/NO grid, or market-neutral prediction bot.

Model: two long grids under one orchestrator. The bot buys YES below the YES market and buys NO below the NO market.

| Leg | Market | Range logic |
| --- | --- | --- |
| YES | outcome side `0` | Long YES below/around YES price |
| NO | outcome side `1` | Long NO below/around NO price |

Rules:
- Use live `outcomeMeta`. Never infer HIP-4 availability from `meta`, `metaAndAssetCtxs`, or `perpDexs`.
- Do not use single-market `supurr new grid --type outcome --mode neutral`.
- Do not short outcome markets for neutral exposure. Use two long grids: long YES and long NO.
- Do not call HIP-3 builder perps HIP-4. HIP-4/prediction availability comes only from `outcomeMeta`.
- User/agent supplies explicit YES and NO ranges from live market context. CLI does not auto-calculate ranges in v1.
- `--investment` is total group capital; Rust splits static 50/50.
- Long grid active buys are `--levels - 1`; the top grid point is the first TP spacer.
- Minimum total capital is `2 * active_buy_levels * 20 USDH`. Three active buys per YES/NO leg needs `--levels 4` and at least `120 USDH`.
- Put `yes-end` and `no-end` below the current best bid when the goal is maker-only entries without immediate fills. If the user wants "three levels below current", use `--levels 4`.
- Outcome labels may say Above/Below/Range. Do not infer side from wording; use `outcome_id`, YES side `0`, and NO side `1` from `outcomeMeta`.
- Backtests are price-path simulations, not proof of live fill quality.
- Backtest first. Deploy live only after explicit confirmation.

Agent execution flow:

| Step | Agent action | Hard check |
| --- | --- | --- |
| 1 | Detect neutral prediction intent | Use `supurr new orchestrator`, not single-leg grid |
| 2 | Fetch live markets with `outcomeMeta` | Outcome must be live, tradable, and not pinned near `0` or `1` |
| 3 | Map coins | YES coin is `#<10*outcome_id>`, NO coin is `#<10*outcome_id+1>` |
| 4 | Fetch `allMids` and both `l2Book`s | Use book/mid to choose explicit YES and NO ranges |
| 5 | Generate config | `--investment` is total group capital; static 50/50 split happens in Rust |
| 6 | Backtest | Report PnL, fills, and open exposure; ask before deploy |
| 7 | Deploy after confirmation | Verify open orders exist on both YES and NO coins |
| 8 | Monitor live | YES fill should create sell YES TP; NO fill should create sell NO TP |
| 9 | Stop | After stop, verify open outcome orders are gone |

Live setup flow:

```bash
# 1. Fetch live prediction markets.
curl -sS -X POST https://api.hyperliquid.xyz/info \
  -H 'content-type: application/json' \
  --data '{"type":"outcomeMeta"}'

# 2. Fetch current outcome prices / books.
curl -sS -X POST https://api.hyperliquid.xyz/info \
  -H 'content-type: application/json' \
  --data '{"type":"allMids"}'

curl -sS -X POST https://api.hyperliquid.xyz/info \
  -H 'content-type: application/json' \
  --data '{"type":"l2Book","coin":"#70"}'

curl -sS -X POST https://api.hyperliquid.xyz/info \
  -H 'content-type: application/json' \
  --data '{"type":"l2Book","coin":"#71"}'
```

Choose one live outcome where both YES and NO are tradable and not pinned near `0` or `1`. For outcome id `7`, YES coin is `#70`, NO coin is `#71`.

```bash
supurr new orchestrator \
  --kind prediction-yes-no-grid \
  --asset BTC \
  --outcome-id 7 \
  --yes-start 0.235 \
  --yes-end 0.265 \
  --no-start 0.610 \
  --no-end 0.650 \
  --levels 4 \
  --investment 120 \
  --take-profit 10 \
  --stop-loss 10 \
  --output btc-yes-no-neutral

supurr backtest -c btc-yes-no-neutral

# Deploy only after the user explicitly approves live trading.
supurr deploy -c btc-yes-no-neutral
supurr monitor
```

After deploy, verify both legs on Hyperliquid:

```bash
curl -sS -X POST https://api.hyperliquid.xyz/info \
  -H 'content-type: application/json' \
  --data '{"type":"openOrders","user":"0x...","dex":"ALL_DEXS"}'
```

Expected:
- Before fills: buy orders on both `#<10*outcome_id>` and `#<10*outcome_id+1>`.
- After a YES fill: a sell YES TP order appears.
- After a NO fill: a sell NO TP order appears.
- After `supurr stop`: no live orders remain for either outcome coin.

---

## 4. `supurr configs` — List Saved Configs

```bash
supurr configs    # Lists all configs in ~/.supurr/configs/
```

**Output:**

```
📁 Configs (/Users/you/.supurr/configs):
  btc-grid.json         grid     BTC-USDC
  hype-usdc-spot.json   grid     HYPE-USDC
  hyna-btc.json         grid     BTC-USDE
```

---

## 5. `supurr config <name>` — View Config

```bash
supurr config btc-grid        # View btc-grid.json
supurr config btc-grid.json   # Same
```

---

## 6. `supurr backtest` — Run Backtest

### Syntax

```bash
supurr backtest -c <config> [options]
```

> **Supported strategies**: Grid, DCA, and custom strategies. Arb (spot-perp arbitrage) backtesting is **not supported** — arb requires simultaneous dual-market execution that cannot be accurately simulated from single-asset price feeds.

> **⚠️ IMPORTANT — Asset ID Required**: Bot configs require a `market_index` (Hyperliquid's internal asset index, e.g. `3` for BTC). You **must** know this value beforehand. It is **not** auto-resolved by the CLI. To find it, query the Hyperliquid Info API: `POST https://api.hyperliquid.xyz/info` with `{"type": "meta"}` — the response `universe` array contains all assets with their indices. Alternatively, check the [Hyperliquid Info API reference](references/hl-info-api.md) for the exact endpoint.

### Options

| Option                | Description                              |
| --------------------- | ---------------------------------------- |
| `-c, --config <file>` | **Required.** Config file (name or path) |
| `-s, --start <date>`  | Start date (YYYY-MM-DD)                  |
| `-e, --end <date>`    | End date (YYYY-MM-DD)                    |
| `-p, --prices <file>` | Use local prices file                    |
| `-o, --output <file>` | Save results to JSON                     |
| `--no-cache`          | Disable price caching                    |
| `--disassembly <mode>` | OHLC disassembly: `low-first`, `high-first`, or `random` |

### Examples

```bash
# By config name (looks in ~/.supurr/configs/)
supurr backtest -c btc-grid.json -s 2026-01-28 -e 2026-02-01

# By full path
supurr backtest -c ~/.supurr/configs/btc-grid.json -s 2026-01-28 -e 2026-02-01

# Save results
supurr backtest -c btc-grid.json -s 2026-01-28 -e 2026-02-01 -o results.json

# Choose OHLC path inside each 1s candle
supurr backtest -c btc-grid.json -s 2026-06-14 -e 2026-06-14 --disassembly random

# Reuse candles fetched by supurr analytics prices
supurr analytics prices --market BTC --start 2026-06-14 --end 2026-06-14 --loc ./btc-candles.json
supurr backtest -c btc-grid.json -p ./btc-candles.json --disassembly high-first
```

### Archive Data Availability

| Dex           | Asset Format           | Example            |
| ------------- | ---------------------- | ------------------ |
| `hyperliquid` | `BTC`, `HYPE`          | Native perp + Spot |
| `hyna`        | `hyna:BTC`, `hyna:ETH` | HIP-3 DEX          |

> **Note**: Archive data available from 2026-01-28 onwards.
>
> **Important**: Backtests fetch Supurr Oracle 1s OHLCV and disassemble each candle into synthetic price points. `--prices` accepts either old `{ prices: [...] }` tick files or `supurr analytics prices` candle exports. Do **not** use Hyperliquid Info API mids/candles for backtests; they do not provide the same replay contract.

### Supurr Oracle Candle API

Base URL:

```txt
https://oracle.supurr.app
```

Health:

```bash
curl -sS https://oracle.supurr.app/health
```

Day file:

```bash
curl -sS 'https://oracle.supurr.app/v1/candles/day?date=2026-06-14'
```

Window, max 7 UTC days:

```bash
curl -sS 'https://oracle.supurr.app/v1/candles/window?end_date=2026-06-14&days=2'
```

Responses return presigned object URLs:

```json
{
  "key": "global/candles/1s/date=2026-06-14/candles-processed.json.gz",
  "url": "<presigned_download_url>",
  "expires_in_secs": 3600
}
```

Date APIs resolve in this order:

```txt
candles-processed.json.gz -> candles-live.json.gz -> 404
```

Use exact-key presign only when the caller already knows the bucket key:

```bash
curl -sS --get 'https://oracle.supurr.app/v1/presign' \
  --data-urlencode 'key=global/candles/1s/date=2026-06-14/candles-processed.json.gz'
```

---

## 6a. `supurr paper` — Paper Trading

Run any strategy with **real market quotes** but **simulated fills**. No real orders are placed on the exchange.

Uses the same isolated-margin simulation engine as `supurr backtest`, including:
- Per-position margin reservation and liquidation detection
- Configurable leverage, fee rates, and starting balance
- Real-time equity, unrealized PnL, and position tracking

### Syntax

```bash
supurr paper -c <config> [--debug]
```

### Options

| Option                | Description                              |
| --------------------- | ---------------------------------------- |
| `-c, --config <path>` | **Required.** Config file (name or path) |
| `-d, --debug`         | Enable debug engine logs (RUST_LOG=debug)|

### Examples

```bash
# By config name (looks in ~/.supurr/configs/)
supurr paper -c my-grid-bot

# By file path
supurr paper -c ./config-v2-grid-perp.json

# With debug logging
supurr paper -c my-grid-bot --debug
```

### Simulation Config (optional)

Add a top-level `simulation` block to your config JSON to control paper trading assumptions:

```json
{
  "simulation": {
    "starting_balance_usdc": "10000",
    "fee_rate": "0.00025"
  }
}
```

| Field | Default | Description |
|---|---|---|
| `starting_balance_usdc` | `"10000"` | Starting USDC balance |
| `fee_rate` | `"0.00025"` | Fee rate per fill (0.025% = taker) |

> **Ctrl+C** to stop paper trading cleanly. The engine cancels all open orders on shutdown.

---

## 6b. `supurr dev` — Custom Strategy Development

Development commands for building and testing custom strategies locally.

```bash
supurr dev init              # Clone/update bot source to ~/.supurr/bot-source/
supurr dev build             # Build from source (cargo build --release)
supurr dev run -c <config>   # Run locally-built bot
supurr dev backtest -c <config> [options]  # Backtest with dev-built engine
```

### `supurr dev backtest`

Identical to `supurr backtest` but uses the **dev-built engine** at `~/.supurr/bot-source/target/release/bot` instead of the installed binary. This lets you backtest **custom strategies** that you've added to the bot source.

```bash
# Backtest your custom strategy
supurr dev backtest -c config-mystrategy.json -s 2026-01-28 -e 2026-02-01

# With local prices
supurr dev backtest -c config-mystrategy.json -p prices.json
```

All options from `supurr backtest` apply (`-s`, `-e`, `-p`, `--no-cache`, `-o`).

> **Prerequisite**: Run `supurr dev build` first to compile your custom strategy into the dev binary.

### Full Custom Strategy Workflow

```bash
# 1. Clone the bot source
supurr dev init

# 2. Add your strategy crate (see STRATEGY_API.md)
# 3. Build with your strategy
supurr dev build

# 4. Backtest it
supurr dev backtest -c config-mystrategy.json -s 2026-01-28 -e 2026-02-01

# 5. Run live (paper or real)
supurr dev run -c config-mystrategy.json
```

---

## 7. `supurr deploy` — Deploy Bot

```bash
supurr deploy -c <config> [-s <address> | -v <address>]
```

| Option                       | Description                                             |
| ---------------------------- | ------------------------------------------------------- |
| `-c, --config <file>`        | **Required.** Config file (name or path)                |
| `-s, --subaccount <address>` | Trade from a subaccount (validates master ownership)    |
| `-v, --vault <address>`      | Trade from a vault (validates you are the vault leader) |

> **Subaccount vs Vault:**
>
> - **Subaccount** = personal trading account under your master wallet. Verified via `subAccounts` API (checks `master` field).
> - **Vault** = shared investment pool you manage. Verified via `vaultDetails` API (checks `leader` field).
> - Both set `vault_address` in the bot config on success.
> - **Cannot use both** `--subaccount` and `--vault` simultaneously.

### Examples

```bash
# Deploy from main wallet
supurr deploy -c btc-grid.json

# Deploy from subaccount
supurr deploy -c btc-grid.json -s 0x804e57d7baeca937d4b30d3cbe017f8d73c21f1b

# Deploy from vault (you must be the vault leader)
supurr deploy -c config.json --vault 0xdc89f67e74098dd93a1476f7da79747f71ccb5d9

# HL: prefix is auto-stripped (copy-paste from Hyperliquid UI)
supurr deploy -c config.json -s HL:0x804e57d7baeca937d4b30d3cbe017f8d73c21f1b
```

**Output:**

```
✔ Loaded config for grid strategy
✔ Subaccount verified: 0x804e57d7...
✔ Bot deployed successfully!
📦 Deployment Details
  Bot ID:       217
  Pod Name:     bot-217
  Bot Type:     grid
  Market:       BTC-USDC
```

> **Gotchas**: `HL:` prefix is auto-stripped from addresses. Subaccount requires `master` to match your `supurr whoami` address. Only the vault leader can deploy.

---

## 8. `supurr monitor` — View User's Bots

> **Updated in v0.2.8**: Now shows only the user's bots by default (requires `supurr init`). Use `--history` to include stopped bots.

### Syntax

```bash
supurr monitor [options]
```

### Options

| Option                   | Description                          |
| ------------------------ | ------------------------------------ |
| `-w, --wallet <address>` | Filter by wallet address             |
| `--watch`                | Live mode (refreshes every 2s)       |
| `--history`              | Show all bots including stopped ones |

### Examples

```bash
supurr monitor                 # Show only active bots for current user
supurr monitor --history       # Show all bots (active + stopped)
supurr monitor --watch         # Live monitoring (Ctrl+C to exit)
supurr monitor --watch --history  # Live monitoring with history
supurr monitor -w 0x1234...    # Filter by wallet
```

### Behavior

- **User-Specific**: Fetches bots for the address in `~/.supurr/credentials.json` (from `supurr init`)
- **Default**: Shows only active bots (status = "running" or "starting")
- **With `--history`**: Shows all bots including stopped ones
- **Header**: Displays user address and sync delay: `🤖 Active Bots  │  User: 0x0ecba...  │  Sync delay: 0s`
- **Trading Link**: Shows clickable visualization link at the end with correct market format:
  - **Spot**: `KNTQ_USDH` (underscore separator)
  - **Perp**: `BTC-USDC` (hyphen separator)
  - **HIP-3**: `vntl:ANTHROPIC` (dex:base format)

**Output Columns:** ID, Type, Market, Position (size+direction), PnL. Includes a clickable trading visualization link.

---

## 9. `supurr history` — View Bot History

```bash
supurr history             # Show last 20 bot sessions
supurr history -n 50       # Show last 50 bot sessions
```

| Option                | Default | Description            |
| --------------------- | ------- | ---------------------- |
| `-n, --limit <count>` | `20`    | Number of bots to show |

**Output Columns:** ID, Market, Type, PnL, Stop Reason.

---

## 10. `supurr stop` — Stop Bot (Signature Auth)

Signs `Stop <bot-id>` with your API wallet private key (EIP-191 personal_sign) and sends the signature to the bot API.

```bash
supurr stop              # Interactive - select from list
supurr stop --id 217     # Stop specific bot by ID
```

| Option          | Description                            |
| --------------- | -------------------------------------- |
| `--id <bot_id>` | Bot ID to stop (from `supurr monitor`) |

> **Crypto:** Uses `@noble/curves/secp256k1` + `@noble/hashes/sha3` (pure JS, no native deps). Signature format: `0x{r}{s}{v}` (65 bytes).

---

## 11. `supurr analytics` — Fetch Analytics Data

```bash
supurr analytics                         # List analytics commands
supurr analytics prices --market BTC --start 2026-06-14
supurr analytics prices --market BTC --start 20260614 --end 20260615 --duration 1M
supurr analytics prices --market BTC --start 2026-06-14 --duration 1H --loc ./prices
supurr analytics funding-rates --market BTC
supurr analytics funding-rates --market BTC --start 2026-06-14 --end 2026-06-15
```

### `supurr analytics prices`

Fetches Supurr Oracle 1s OHLCV candles, optionally aggregates them, and writes a `supurr.candles.v1` JSON file.

| Option                  | Description                                      |
| ----------------------- | ------------------------------------------------ |
| `-m, --market <symbol>` | Market symbol, e.g. `BTC`                        |
| `-a, --asset <symbol>`  | Legacy alias for `--market`                      |
| `--dex <dex>`           | DEX name, default `hyperliquid`                  |
| `--asset-class <class>` | `perp`, `spot`, or `outcome`                     |
| `-s, --start <date>`    | **Required.** `YYYY-MM-DD` or `YYYYMMDD`         |
| `-e, --end <date>`      | End date, defaults to today                      |
| `--duration <duration>` | Output duration: `1S`, `1M`, `5M`, `1H`, `1D`    |
| `--loc, --location <path>` | Output directory or JSON file path            |
| `--concurrency <count>` | Concurrent candle downloads, default `50`        |

`supurr prices` remains as a legacy alias for this command.

### `supurr analytics funding-rates`

Fetches latest or historical funding-rate rows from Supurr Oracle.

| Option                  | Description                              |
| ----------------------- | ---------------------------------------- |
| `-m, --market <symbol>` | **Required.** Market symbol, e.g. `BTC`  |
| `--dex <dex>`           | DEX name, default `hyperliquid`          |
| `-s, --start <date>`    | Start date for history                   |
| `-e, --end <date>`      | End date for history                     |
| `--limit <count>`       | Max rows                                 |
| `--order <order>`       | `asc` or `desc`, default `asc`           |
| `--loc, --location <path>` | Output directory or JSON file path    |

---

## 12. `supurr update` — Update All Components

```bash
supurr update    # Updates CLI, AI skill, and bot source
```

Runs three independent steps (one failing won't block others):

| Step | What | How |
|---|---|---|
| 1. CLI binary | Downloads latest from `cli.supurr.app` | Install script (same as `curl \| bash`) |
| 2. AI skill | Pulls latest skill files | `npx skills add Supurr-App/Hyperliquid-Supurr-Skill` |
| 3. Bot source | Git pull in `~/.supurr/bot-source/` | Only if previously cloned via `supurr dev init` |

---

## More Info

- **Workflows**: See → [references/workflows.md](references/workflows.md)
- **Config storage**: `~/.supurr/` contains `credentials.json`, `configs/`, and `cache/`
- **Credential resolution**: Bot reads `~/.supurr/credentials.json` at runtime (v0.3.5+). Use `supurr init` to set/switch wallets.
- **Troubleshooting**: See → [references/troubleshooting.md](references/troubleshooting.md)

---

# References

## CLI & Exchange References

| Reference                                                | Contents                                               |
| -------------------------------------------------------- | ------------------------------------------------------ |
| [Hyperliquid Info API](references/hl-info-api.md)        | All `POST /info` endpoints, TypeScript helper, hazards |
| [Arb Spot Resolution](references/arb-spot-resolution.md) | Full U-prefix table, resolution logic, edge cases      |
| [Troubleshooting](references/troubleshooting.md)         | Common errors and fixes                                |
| [Complete Workflows](references/workflows.md)            | End-to-end Grid, Arb, DCA, HIP-3 workflows             |
| [Bot Discovery](references/bot-discovery.md)             | Active bot surfaces, copy-bot route, recommendation rules |
| [User Action Intents](references/user-action-intents.md) | Machine-readable user-action schema for UI/Telegram handoff |

## Strategy Authoring References

> **For LLMs**: To build a custom trading strategy, read [STRATEGY_API.md](STRATEGY_API.md) first — it has the full contract, 3-file pattern, and E2E build instructions. **For indicator-based strategies** (RSI, MACD, Bollinger, EMA crossover, etc.), also read [indicator-strategies.md](references/indicator-strategies.md) for the 3-layer pattern (BarBuilder → Indicator → Phase Machine) and the `strategy-rsi` reference implementation. Then use the API references below for exact signatures.

| Reference                                                  | Contents                                                                                        |
| ---------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| [Strategy Authoring API](STRATEGY_API.md)                  | **START HERE** — Architecture, Strategy trait, StrategyContext, 3-file pattern, E2E build guide |
| [Indicator Strategies](references/indicator-strategies.md) | 3-layer pattern (BarBuilder → Indicator → Phase Machine) for RSI, MACD, Bollinger, EMA, etc.    |
| [Strategy Trait & Context](references/strategy-trait.md)   | `Strategy` trait + `StrategyContext` method signatures (commands, timers, read-only state)      |
| [Command Structs](references/commands.md)                  | `PlaceOrder`, `CancelOrder`, `CancelAll`, `StopStrategy` — constructors + builders              |
| [Event Enum](references/events.md)                         | All events: `Quote`, `OrderFilled`, `OrderCompleted`, `OrderCanceled`, `OrderRejected`, etc.    |
| [Core Types](references/types.md)                          | All types: `Price`, `Qty`, `Market`, `Position`, `Balance`, `InstrumentMeta`, `LiveOrder`, etc. |
| [Strategy Template](templates/strategy-template/)          | Scaffold crate with TODO markers — copy to start a new strategy                                 |
| [Simple Strategy Example](examples/strategy-simple/)       | Complete working buy-low-sell-high strategy (~140 lines)                                        |
| [Custom Strategy Tutorial](tutorials/custom-strategy.md)   | End-to-end walkthrough: scaffold → implement → register → build → run                           |

---

## Tutorials

- [Grid Bot Tutorial](tutorials/grid.md) — Range trading with buy/sell grids
- [Arb Bot Tutorial](tutorials/arb.md) — Market-neutral spot-perp arbitrage
- [DCA Bot Tutorial](tutorials/dca.md) — Dollar-cost averaging with auto-restart
- [Prediction Markets](tutorials/prediction-markets.md) — Trade binary HIP-4 outcomes using live `outcomeMeta`
- [Build Your Own Strategy](tutorials/custom-strategy.md) — Custom strategy from scratch
