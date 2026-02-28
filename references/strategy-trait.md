# Strategy Trait & StrategyContext API Reference

> Source: `bot-core/src/strategy.rs`

## Strategy Trait

Every strategy must implement this trait. Strategies are **deterministic state machines** that receive events and emit commands via the context. They never call HTTP directly.

```rust
pub trait Strategy: Send + 'static {
    /// Unique identifier for this strategy instance
    fn id(&self) -> &StrategyId;

    /// Synchronization mechanism (default: Poll for incremental fills)
    fn sync_mechanism(&self) -> SyncMechanism {
        SyncMechanism::Poll
    }

    /// Called once when the engine starts this strategy.
    /// Use this to initialize state, set timers, load instrument metadata, validate config.
    fn on_start(&mut self, ctx: &mut dyn StrategyContext);

    /// Called for every event routed to this strategy.
    /// Key events: Quote (price update), OrderFilled, OrderCanceled, OrderRejected
    fn on_event(&mut self, ctx: &mut dyn StrategyContext, event: &Event);

    /// Called when a timer fires (timers set via ctx.set_timer / ctx.set_interval).
    fn on_timer(&mut self, ctx: &mut dyn StrategyContext, timer_id: TimerId);

    /// Called once when the engine is stopping this strategy.
    /// Use this to cancel all orders, log final state.
    fn on_stop(&mut self, ctx: &mut dyn StrategyContext);
}
```

### SyncMechanism

- **`Poll`** (default): Incremental fills via `poll_user_fills()`. Best for most strategies.
- **`Snapshot`**: Absolute account state via `poll_account_state()`. Use when you need position reconciliation.

---

## StrategyContext Trait

Passed to all strategy callbacks. Provides command emission, timer management, read-only market/account state, and logging.

### Commands

| Method          | Signature                                                            | Description                                                               |
| --------------- | -------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| `place_order`   | `fn place_order(&mut self, cmd: PlaceOrder)`                         | Place a single limit order. Returns immediately; result comes via events. |
| `place_orders`  | `fn place_orders(&mut self, cmds: Vec<PlaceOrder>)`                  | Batch place multiple orders in one API call.                              |
| `cancel_order`  | `fn cancel_order(&mut self, cmd: CancelOrder)`                       | Cancel an order by `ClientOrderId`.                                       |
| `cancel_all`    | `fn cancel_all(&mut self, cmd: CancelAll)`                           | Cancel all orders, optionally for a specific instrument.                  |
| `stop_strategy` | `fn stop_strategy(&mut self, strategy_id: StrategyId, reason: &str)` | Request graceful stop. Triggers `on_stop` after current event.            |

### Timers

| Method         | Signature                                                   | Description                                  |
| -------------- | ----------------------------------------------------------- | -------------------------------------------- |
| `set_timer`    | `fn set_timer(&mut self, delay: Duration) -> TimerId`       | One-shot timer that fires after `delay`.     |
| `set_interval` | `fn set_interval(&mut self, interval: Duration) -> TimerId` | Repeating timer that fires every `interval`. |
| `cancel_timer` | `fn cancel_timer(&mut self, timer_id: TimerId)`             | Cancel a previously set timer.               |

### Read-Only State

| Method            | Signature                                                                         | Description                                        |
| ----------------- | --------------------------------------------------------------------------------- | -------------------------------------------------- |
| `mid_price`       | `fn mid_price(&self, instrument: &InstrumentId) -> Option<Price>`                 | Current mid price from quote poller.               |
| `quote`           | `fn quote(&self, instrument: &InstrumentId) -> Option<Quote>`                     | Current best bid/ask.                              |
| `instrument_meta` | `fn instrument_meta(&self, instrument: &InstrumentId) -> Option<&InstrumentMeta>` | Tick size, lot size, min qty, etc.                 |
| `balance`         | `fn balance(&self, asset: &AssetId) -> Balance`                                   | Balance for an asset (total, available, reserved). |
| `position`        | `fn position(&self, instrument: &InstrumentId) -> Position`                       | Current position (signed qty, entry px, PnL).      |
| `exchange_health` | `fn exchange_health(&self, exchange: &ExchangeInstance) -> ExchangeHealth`        | Exchange status: `Active` or `Halted`.             |
| `order`           | `fn order(&self, client_id: &ClientOrderId) -> Option<&LiveOrder>`                | Look up a tracked order.                           |

### Time

| Method   | Signature                 | Description                                               |
| -------- | ------------------------- | --------------------------------------------------------- |
| `now_ms` | `fn now_ms(&self) -> i64` | Current time in milliseconds. Deterministic in backtests. |

### Logging

| Method           | Description       |
| ---------------- | ----------------- |
| `log_info(msg)`  | Info-level log    |
| `log_warn(msg)`  | Warning-level log |
| `log_error(msg)` | Error-level log   |
| `log_debug(msg)` | Debug-level log   |
