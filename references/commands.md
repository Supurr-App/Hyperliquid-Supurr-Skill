# Command Structs Reference

> Source: `bot-core/src/commands.rs`

Commands are what strategies emit to request actions from the engine. You emit commands via `StrategyContext` methods — you never construct the `Command` enum directly.

---

## PlaceOrder

Place a limit order. Use the `PlaceOrder::limit()` constructor + builder methods.

```rust
pub struct PlaceOrder {
    pub client_id: ClientOrderId,      // Auto-generated UUID
    pub exchange: ExchangeInstance,     // Target exchange + environment
    pub instrument: InstrumentId,      // e.g. "BTC-PERP"
    pub side: OrderSide,               // Buy | Sell
    pub price: Price,                  // Limit price
    pub qty: Qty,                      // Quantity
    pub tif: TimeInForce,              // Gtc (default) | Ioc | Fok
    pub post_only: bool,               // Maker-only (rejected if would take)
    pub reduce_only: bool,             // Close position only
}
```

### Constructor

```rust
// Basic limit order (GTC, not post-only, not reduce-only)
let order = PlaceOrder::limit(
    exchange_instance,        // ExchangeInstance
    InstrumentId::new("BTC-PERP"),
    OrderSide::Buy,
    Price::new(dec!(85000)),
    Qty::new(dec!(0.01)),
);
```

### Builder Methods

```rust
let order = PlaceOrder::limit(exchange, instrument, OrderSide::Buy, price, qty)
    .post_only()                    // Maker-only
    .reduce_only()                  // Close position only
    .with_tif(TimeInForce::Ioc)     // Immediate or cancel
    .with_client_id(custom_id);     // Override auto-generated ID
```

### Emitting via Context

```rust
// Single order
ctx.place_order(order);

// Batch (single API call)
ctx.place_orders(vec![order1, order2, order3]);
```

---

## CancelOrder

Cancel an order by client ID.

```rust
pub struct CancelOrder {
    pub exchange: ExchangeInstance,
    pub client_id: ClientOrderId,
}
```

### Usage

```rust
ctx.cancel_order(CancelOrder::new(exchange_instance, client_order_id));
```

---

## CancelAll

Cancel all open orders, optionally for a specific instrument.

```rust
pub struct CancelAll {
    pub exchange: ExchangeInstance,
    pub instrument: Option<InstrumentId>,
}
```

### Usage

```rust
// Cancel ALL orders on this exchange
ctx.cancel_all(CancelAll::new(exchange_instance));

// Cancel only orders for BTC-PERP
ctx.cancel_all(CancelAll::for_instrument(exchange_instance, InstrumentId::new("BTC-PERP")));
```

---

## StopStrategy

Request the engine to stop this strategy gracefully. Triggers `on_stop` callback.

```rust
pub struct StopStrategy {
    pub strategy_id: StrategyId,
    pub reason: String,
}
```

### Usage

```rust
ctx.stop_strategy(self.config.strategy_id.clone(), "Take profit reached");
```
