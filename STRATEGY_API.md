# Strategy Authoring API

Build custom trading strategies for the Supurr bot by implementing the `Strategy` trait in Rust.

## Architecture

Your strategy is a **deterministic state machine**:

- Receives **events** (price updates, order fills, cancellations)
- Emits **commands** (place/cancel orders) via the context
- Never calls HTTP directly — the engine handles all exchange communication

```
┌──────────────────────────────────────────────────────┐
│  Engine                                              │
│  ┌──────────┐    Events     ┌──────────────────┐     │
│  │ Exchange  │ ──────────── │  Your Strategy   │     │
│  │ Adapter   │ ◄─────────── │  (config/state/  │     │
│  │           │   Commands   │   strategy.rs)   │     │
│  └──────────┘               └──────────────────┘     │
└──────────────────────────────────────────────────────┘
```

---

## The Contract: Strategy Trait

Every strategy implements 4 callbacks:

```rust
pub trait Strategy: Send + 'static {
    fn id(&self) -> &StrategyId;

    /// Called once at startup. Initialize state, set timers, validate config.
    fn on_start(&mut self, ctx: &mut dyn StrategyContext);

    /// Called for every event. React to price changes, order fills, cancellations.
    fn on_event(&mut self, ctx: &mut dyn StrategyContext, event: &Event);

    /// Called when a timer fires.
    fn on_timer(&mut self, ctx: &mut dyn StrategyContext, timer_id: TimerId);

    /// Called at shutdown. Cancel all orders, log final state.
    fn on_stop(&mut self, ctx: &mut dyn StrategyContext);
}
```

> **Full reference:** [strategy-trait.md](references/strategy-trait.md)

---

## What You Can Do (StrategyContext)

The context provides 16 methods:

### Commands

```rust
ctx.place_order(PlaceOrder::limit(exchange, instrument, OrderSide::Buy, price, qty));
ctx.place_orders(vec![order1, order2]);    // Batch
ctx.cancel_order(CancelOrder::new(exchange, client_id));
ctx.cancel_all(CancelAll::new(exchange));  // Cancel all
ctx.stop_strategy(strategy_id, "reason");  // Graceful stop
```

### Timers

```rust
let timer = ctx.set_timer(Duration::from_secs(60));     // One-shot
let interval = ctx.set_interval(Duration::from_secs(5)); // Repeating
ctx.cancel_timer(timer);
```

### Read-Only State

```rust
let mid = ctx.mid_price(&instrument_id);            // Option<Price>
let quote = ctx.quote(&instrument_id);               // Option<Quote> (bid/ask)
let meta = ctx.instrument_meta(&instrument_id);      // Option<&InstrumentMeta>
let bal = ctx.balance(&AssetId::new("USDC"));         // Balance { total, available, reserved }
let pos = ctx.position(&instrument_id);               // Position { qty, avg_entry_px, pnl }
let health = ctx.exchange_health(&exchange);           // Active | Halted
let order = ctx.order(&client_id);                     // Option<&LiveOrder>
let now = ctx.now_ms();                                // i64 (deterministic in backtests)
```

### Logging

```rust
ctx.log_info("Strategy started");
ctx.log_warn("Spread too wide");
ctx.log_error("Order placement failed");
ctx.log_debug("Internal state dump");
```

> **Full references:** [strategy-trait.md](references/strategy-trait.md), [commands.md](references/commands.md), [types.md](references/types.md)

---

## Events Your Strategy Receives

```rust
fn on_event(&mut self, ctx: &mut dyn StrategyContext, event: &Event) {
    match event {
        Event::Quote(q) => {
            // Price updated. q.bid, q.ask, q.mid()
        }
        Event::OrderFilled(f) => {
            // Fill occurred. f.side, f.price, f.qty, f.net_qty, f.fee
        }
        Event::OrderCompleted(c) => {
            // Fully filled (terminal). c.filled_qty, c.avg_fill_px
        }
        Event::OrderCanceled(c) => {
            // Canceled (terminal). c.reason
        }
        Event::OrderRejected(r) => {
            // Rejected (terminal). r.reason
        }
        Event::OrderAccepted(a) => {
            // Acknowledged by exchange
        }
        Event::FundingRate(f) => {
            // Perp funding rate changed. f.rate
        }
        Event::ExchangeStateChanged(e) => {
            // Exchange Active <-> Halted. Pause orders when Halted.
        }
    }
}
```

> **Full reference:** [events.md](references/events.md)

---

## The 3-File Pattern

Every strategy crate follows this structure:

```
strategy-mystrategy/
├── Cargo.toml
└── src/
    ├── lib.rs         # pub mod config; pub mod state; pub mod strategy; pub use *;
    ├── config.rs      # Configuration (serde + JsonSchema for JSON config)
    ├── state.rs       # Runtime state (not serialized)
    └── strategy.rs    # Strategy trait implementation
```

### config.rs — What the user configures

```rust
use bot_core::{Environment, Market, StrategyId};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MyConfig {
    pub strategy_id: StrategyId,
    pub environment: Environment,
    pub market: Market,    // Unified market (perp/spot/hip3)

    // Your strategy-specific params:
    pub threshold: Decimal,
    pub order_size: Decimal,
}

impl MyConfig {
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.order_size <= Decimal::ZERO {
            errors.push("order_size must be > 0".into());
        }
        errors
    }
}
```

### state.rs — Runtime tracking

```rust
use bot_core::{ClientOrderId, Price};

pub struct MyState {
    pub is_initialized: bool,
    pub last_mid: Option<Price>,
    pub active_order: Option<ClientOrderId>,
    // Your runtime state...
}

impl MyState {
    pub fn new() -> Self {
        Self {
            is_initialized: false,
            last_mid: None,
            active_order: None,
        }
    }
}
```

### strategy.rs — The brain

```rust
use crate::config::MyConfig;
use crate::state::MyState;
use bot_core::*;

pub struct MyStrategy {
    config: MyConfig,
    state: MyState,
    instrument_meta: Option<InstrumentMeta>,
}

impl MyStrategy {
    pub fn new(config: MyConfig) -> Self {
        Self {
            config,
            state: MyState::new(),
            instrument_meta: None,
        }
    }
}

impl Strategy for MyStrategy {
    fn id(&self) -> &StrategyId { &self.config.strategy_id }

    fn on_start(&mut self, ctx: &mut dyn StrategyContext) {
        let instrument = self.config.market.instrument_id();
        self.instrument_meta = ctx.instrument_meta(&instrument).cloned();

        let errors = self.config.validate();
        if !errors.is_empty() {
            ctx.log_error(&format!("Config errors: {:?}", errors));
            ctx.stop_strategy(self.config.strategy_id.clone(), "Invalid config");
            return;
        }

        ctx.log_info(&format!("Strategy started for {}", instrument));
    }

    fn on_event(&mut self, ctx: &mut dyn StrategyContext, event: &Event) {
        match event {
            Event::Quote(q) => {
                // Your trading logic here
            }
            Event::OrderFilled(f) => {
                // Handle fills
            }
            Event::OrderCanceled(_) | Event::OrderRejected(_) => {
                // Reset order tracking
                self.state.active_order = None;
            }
            _ => {}
        }
    }

    fn on_timer(&mut self, ctx: &mut dyn StrategyContext, timer_id: TimerId) {
        // Handle periodic logic
    }

    fn on_stop(&mut self, ctx: &mut dyn StrategyContext) {
        ctx.cancel_all(CancelAll::new(
            self.config.market.exchange_instance(self.config.environment),
        ));
        ctx.log_info("Strategy stopped");
    }
}
```

---

## PlaceOrder Builder

The most common command. Use `PlaceOrder::limit()` constructor + builder chain:

```rust
let exchange = self.config.market.exchange_instance(self.config.environment);
let instrument = self.config.market.instrument_id();

// Round price and qty using instrument metadata
let price = self.instrument_meta.as_ref().unwrap().round_price(raw_price);
let qty = self.instrument_meta.as_ref().unwrap().round_qty(raw_qty);

let order = PlaceOrder::limit(exchange, instrument, OrderSide::Buy, price, qty)
    .post_only();      // Optional: maker-only

ctx.place_order(order);
```

> **Important**: Always round price and quantity using `InstrumentMeta` before placing orders. Hyperliquid requires max 5 significant figures for prices.

---

## Market Configuration (JSON)

The bot reads a JSON config file. Your strategy maps to a section:

```json
{
  "strategy_type": "mystrategy",
  "environment": "mainnet",
  "markets": [
    {
      "exchange": "hyperliquid",
      "type": "perp",
      "base": "BTC",
      "index": 0
    }
  ],
  "wallet_address": "0x...",
  "mystrategy": {
    "threshold": "0.005",
    "order_size": "0.01"
  }
}
```

---

## E2E: Build & Run

### 1. Clone the bot repository

```bash
git clone https://github.com/Supurr-App/bot.git
cd bot
```

### 2. Create your strategy crate

```bash
mkdir -p crates/strategy-mystrategy/src
```

Copy the template from `supurr_skill/templates/strategy-template/` and fill in the TODOs.

### 3. Wire into the bot (3 files, ~10 lines total)

**File 1** — Root `Cargo.toml` (add 2 lines):

```toml
[workspace]
members = [
    # ... existing crates
    "crates/strategy-mystrategy",        # ← add
]

[workspace.dependencies]
strategy-mystrategy = { path = "crates/strategy-mystrategy" }  # ← add
```

**File 2** — `crates/bot-engine/Cargo.toml` (add 1 line):

```toml
[dependencies]
strategy-mystrategy = { workspace = true, optional = true }  # ← add
```

Also add it to the `native` feature list:

```toml
native = ["tokio", "reqwest", "strategy-grid", "strategy-dca", "strategy-market-maker", "strategy-arbitrage", "strategy-mystrategy"]
```

**File 3** — `crates/bot-engine/src/config.rs` (add ~8 lines):

```rust
// 1. Add import at top:
use strategy_mystrategy::{MyConfig, MyStrategy};

// 2. Add branch in build_strategy() BEFORE the final `else`:
    } else if strategy_type == "mystrategy" {
        // custom_config() auto-deserializes from the JSON's "mystrategy" section
        let my_config: MyConfig = config.custom_config("mystrategy")?;
        let errors = my_config.validate();
        if !errors.is_empty() {
            anyhow::bail!("Config validation failed: {}", errors.join(", "));
        }
        Ok(Box::new(MyStrategy::new(my_config)))
    } else {
        // ... existing fallback
```

> **That's it.** No need to modify the `BotConfig` struct — your strategy's JSON config section is automatically captured by `serde(flatten)` and deserialized by `custom_config()`.

### 4. Build & run

```bash
cargo build --release
./target/release/bot-cli --config config-mystrategy.json
```

---

## Dependencies (Cargo.toml)

Your strategy crate's `Cargo.toml`:

```toml
[package]
name = "strategy-mystrategy"
version.workspace = true
edition.workspace = true

[dependencies]
bot-core = { workspace = true }
rust_decimal = { workspace = true }
rust_decimal_macros = { workspace = true }
serde = { workspace = true }
schemars = { workspace = true }
tracing = { workspace = true }
```

---

## API Reference Index

| Document                                            | Content                                                          |
| --------------------------------------------------- | ---------------------------------------------------------------- |
| [strategy-trait.md](references/strategy-trait.md)   | `Strategy` + `StrategyContext` full API                          |
| [events.md](references/events.md)                   | `Event` enum + all event structs                                 |
| [commands.md](references/commands.md)               | `PlaceOrder`, `CancelOrder`, `CancelAll` + builders              |
| [types.md](references/types.md)                     | All types: `Price`, `Qty`, `Market`, `Position`, `Balance`, etc. |
| [hl-info-api.md](references/hl-info-api.md)         | Hyperliquid Info API (market data queries)                       |
| [troubleshooting.md](references/troubleshooting.md) | Common issues and solutions                                      |
