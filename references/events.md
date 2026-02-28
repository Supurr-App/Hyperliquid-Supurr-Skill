# Event Enum Reference

> Source: `bot-core/src/events.rs`

All events that strategies receive via `on_event`. Every event has a `.ts` (timestamp in ms) and most have `.instrument`.

```rust
pub enum Event {
    // Market data
    Quote(QuoteEvent),
    FundingRate(FundingRateEvent),

    // Execution (order lifecycle)
    OrderAccepted(OrderAcceptedEvent),
    OrderRejected(OrderRejectedEvent),
    OrderFilled(OrderFilledEvent),
    OrderCompleted(OrderCompletedEvent),
    OrderCanceled(OrderCanceledEvent),

    // System
    ExchangeStateChanged(ExchangeStateChangedEvent),
}
```

---

## Market Events

### QuoteEvent

Fired when bid/ask prices update. The most frequent event — drives strategy logic.

```rust
pub struct QuoteEvent {
    pub exchange: ExchangeId,
    pub instrument: InstrumentId,
    pub bid: Price,
    pub ask: Price,
    pub ts: i64,
}

impl QuoteEvent {
    pub fn mid(&self) -> Price;  // (bid + ask) / 2
}
```

### FundingRateEvent

Fired when funding rate changes (perps only).

```rust
pub struct FundingRateEvent {
    pub exchange: ExchangeId,
    pub instrument: InstrumentId,
    pub rate: Decimal,  // e.g. 0.0001 = 0.01%
    pub ts: i64,
}
```

---

## Execution Events (Order Lifecycle)

Order lifecycle: **Place** → `OrderAccepted` → `OrderFilled`\* → `OrderCompleted` (terminal)

\* Partial fills produce multiple `OrderFilled` events before `OrderCompleted`.

### OrderAcceptedEvent

Exchange acknowledged the order.

```rust
pub struct OrderAcceptedEvent {
    pub exchange: ExchangeId,
    pub instrument: InstrumentId,
    pub client_id: ClientOrderId,
    pub exchange_order_id: Option<ExchangeOrderId>,
    pub ts: i64,
}
```

### OrderRejectedEvent

Order was rejected by exchange or engine. **Terminal** — no further events for this order.

```rust
pub struct OrderRejectedEvent {
    pub exchange: ExchangeId,
    pub instrument: InstrumentId,
    pub client_id: ClientOrderId,
    pub reason: String,
    pub ts: i64,
}
```

### OrderFilledEvent

A fill occurred (partial or full). May fire multiple times per order.

```rust
pub struct OrderFilledEvent {
    pub exchange: ExchangeId,
    pub instrument: InstrumentId,
    pub client_id: ClientOrderId,
    pub trade_id: TradeId,
    pub side: OrderSide,
    pub price: Price,
    pub qty: Qty,       // Gross quantity filled
    pub net_qty: Qty,   // Net after fees (important for spot BUY)
    pub fee: Fee,
    pub ts: i64,
}
```

> **Spot BUY gotcha**: `net_qty < qty` when fee is in base asset. Always use `net_qty` for position tracking on spot buys.

### OrderCompletedEvent

Order fully filled. **Terminal** — the order lifecycle is complete.

```rust
pub struct OrderCompletedEvent {
    pub exchange: ExchangeId,
    pub instrument: InstrumentId,
    pub client_id: ClientOrderId,
    pub filled_qty: Qty,
    pub avg_fill_px: Option<Price>,
    pub ts: i64,
}
```

### OrderCanceledEvent

Order was canceled. **Terminal**.

```rust
pub struct OrderCanceledEvent {
    pub exchange: ExchangeId,
    pub instrument: InstrumentId,
    pub client_id: ClientOrderId,
    pub reason: Option<String>,
    pub ts: i64,
}
```

---

## System Events

### ExchangeStateChangedEvent

Exchange went Active ↔ Halted (e.g., 502 errors). Strategy should pause order placement when `Halted`.

```rust
pub struct ExchangeStateChangedEvent {
    pub exchange: ExchangeId,
    pub old_state: ExchangeHealth,  // Active | Halted
    pub new_state: ExchangeHealth,
    pub reason: String,
    pub ts: i64,
}
```

---

## Common Patterns

### Matching events in `on_event`

```rust
fn on_event(&mut self, ctx: &mut dyn StrategyContext, event: &Event) {
    match event {
        Event::Quote(q) => {
            let mid = q.mid();
            // React to price update
        }
        Event::OrderFilled(f) => {
            ctx.log_info(&format!("Filled {} @ {}", f.qty, f.price));
            // Update internal state
        }
        Event::OrderCanceled(c) => {
            // Reset order tracking
        }
        Event::OrderRejected(r) => {
            ctx.log_warn(&format!("Rejected: {}", r.reason));
        }
        _ => {} // Ignore other events
    }
}
```
