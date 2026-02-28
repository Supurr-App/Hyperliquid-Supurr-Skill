# Building a Custom Strategy: End-to-End Tutorial

This tutorial walks you through creating a custom trading strategy from scratch, building it, and running it.

---

## 1. Describe Your Strategy

Start with a plain-English description:

> "I want a strategy that places a BUY order when BTC drops below $84,000 and a SELL order when it rises above $86,000. Order size: 0.01 BTC."

---

## 2. Scaffold the Crate

Clone the template into the bot workspace:

```bash
cd ~/bot                      # Your bot repo
cp -r supurr_skill/templates/strategy-template/ crates/strategy-buylow/
```

Update `crates/strategy-buylow/Cargo.toml`:

```toml
[package]
name = "strategy-buylow"
description = "Buy low, sell high strategy"
```

Add to the root `Cargo.toml`:

```toml
[workspace]
members = [
    # ...existing...
    "crates/strategy-buylow",
]

[workspace.dependencies]
strategy-buylow = { path = "crates/strategy-buylow" }
```

---

## 3. Define Your Config

Edit `crates/strategy-buylow/src/config.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BuyLowConfig {
    pub strategy_id: StrategyId,
    pub environment: Environment,
    pub market: Market,
    pub buy_price: Decimal,      // Buy when price drops below this
    pub sell_price: Decimal,     // Sell when price rises above this
    pub order_size: Decimal,     // Order qty in base asset
}
```

---

## 4. Implement the Strategy

Your `strategy.rs` core logic (inside `on_event`):

```rust
Event::Quote(q) => {
    let mid = q.mid();

    // Phase: WaitingToBuy → place BUY below target
    if self.state.phase == Phase::WaitingToBuy && mid.0 <= self.config.buy_price {
        self.place_buy(ctx);
    }
}

Event::OrderCompleted(_) => {
    match self.state.phase {
        Phase::BuyPlaced => self.place_sell(ctx),      // Buy filled → sell
        Phase::SellPlaced => self.place_buy(ctx),      // Sell filled → buy again
        _ => {}
    }
}
```

See `examples/strategy-simple/` for the complete implementation.

---

## 5. Register in the Engine

Edit `crates/bot-engine/src/config.rs`, add to `build_strategy()`:

```rust
} else if strategy_type == "buylow" {
    let json = config.buylow.as_ref().context("buylow config missing")?;
    let buylow_config = BuyLowConfig {
        strategy_id: StrategyId::new("btc-buylow"),
        environment,
        market: config.primary_market().clone(),
        buy_price: Decimal::from_str(&json.buy_price)?,
        sell_price: Decimal::from_str(&json.sell_price)?,
        order_size: Decimal::from_str(&json.order_size)?,
    };
    Ok(Box::new(BuyLowStrategy::new(buylow_config)))
}
```

Add the dependency in `crates/bot-engine/Cargo.toml`:

```toml
strategy-buylow = { workspace = true }
```

---

## 6. Create a Config File

`config-buylow.json`:

```json
{
  "strategy_type": "buylow",
  "environment": "testnet",
  "markets": [
    {
      "exchange": "hyperliquid",
      "type": "perp",
      "base": "BTC",
      "index": 0
    }
  ],
  "wallet_address": "0xYOUR_WALLET",
  "buylow": {
    "buy_price": "84000",
    "sell_price": "86000",
    "order_size": "0.01"
  }
}
```

---

## 7. Build & Test

```bash
# Compile
cargo build --release

# Run tests
cargo test --workspace

# Run with config
./target/release/bot-cli --config config-buylow.json
```

---

## Key Patterns to Remember

| Pattern               | How                                                             |
| --------------------- | --------------------------------------------------------------- |
| Round prices          | `meta.round_price(price)` — Hyperliquid requires max 5 sig figs |
| Round buy qty         | `meta.round_qty(qty)` — standard rounding                       |
| Round sell qty        | `meta.trunc_qty(qty)` — floor to avoid overselling              |
| Track orders          | Store `ClientOrderId` in state, clear on cancel/reject/complete |
| Handle exchange halts | Check `ExchangeStateChanged` event, pause when `Halted`         |
| Graceful stop         | Always cancel all orders in `on_stop`                           |
| Deterministic time    | Use `ctx.now_ms()`, not `SystemTime` — works in backtests       |
