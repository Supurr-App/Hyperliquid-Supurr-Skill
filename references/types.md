# Core Types Reference

> Source: `bot-core/src/types.rs`, `bot-core/src/market.rs`

---

## Identifiers

| Type               | Example                                        | Description                                                  |
| ------------------ | ---------------------------------------------- | ------------------------------------------------------------ |
| `ExchangeId`       | `"hyperliquid"`                                | Exchange name                                                |
| `ExchangeInstance` | `hyperliquid:mainnet`                          | Exchange + environment pair                                  |
| `InstrumentId`     | `"BTC-PERP"`, `"HYPE-SPOT"`, `"hyna:BTC-PERP"` | Canonical instrument key                                     |
| `MarketIndex`      | `0` (BTC perp), `10107` (HYPE spot)            | Exchange-specific numeric index                              |
| `ClientOrderId`    | `"0x8a3f..."`                                  | Client-generated UUID (auto via `ClientOrderId::generate()`) |
| `ExchangeOrderId`  | `"12345"`                                      | Exchange-assigned order ID                                   |
| `TradeId`          | `"abc-123"`                                    | Fill/trade ID from exchange                                  |
| `StrategyId`       | `"btc-grid"`                                   | Unique strategy instance name                                |
| `TimerId`          | `TimerId(1)`                                   | Timer handle from `set_timer`/`set_interval`                 |
| `AssetId`          | `"USDC"`, `"BTC"`                              | Asset currency identifier                                    |

### Environment

```rust
pub enum Environment {
    Mainnet,
    Testnet,
}
```

### Creating an ExchangeInstance

```rust
let exchange = ExchangeInstance::new(
    ExchangeId::new("hyperliquid"),
    Environment::Mainnet,
);
```

---

## Money Types

### Price

Fixed-point decimal for prices. Wraps `rust_decimal::Decimal`.

```rust
let price = Price::new(dec!(85000));
let price = Price::from_str("85000.50")?;

// Rounding
price.round_to_tick(dec!(0.1))    // → 85000.5
price.trim_to_sig_figs(5)         // → 85000 (Hyperliquid requires max 5 sig figs)
```

### Qty

Fixed-point decimal for quantities.

```rust
let qty = Qty::new(dec!(0.01));

qty.is_zero()                     // Check if zero
qty.round_to_lot(dec!(0.001))     // Round to lot size (standard rounding)
qty.trunc_to_lot(dec!(0.001))     // Floor to lot size (use for sells to avoid overselling)
```

> **Rule**: Use `round_to_lot` for buy orders, `trunc_to_lot` for sell orders.

### Fee

```rust
pub struct Fee {
    pub amount: Decimal,
    pub asset: AssetId,    // e.g. "USDC"
}
```

---

## Balance

```rust
pub struct Balance {
    pub total: Decimal,
    pub available: Decimal,
    pub reserved: Decimal,
}
```

Access via `ctx.balance(&AssetId::new("USDC"))`.

---

## Position

Signed quantity: positive = long, negative = short, zero = flat.

```rust
pub struct Position {
    pub qty: Decimal,                    // Signed position size
    pub avg_entry_px: Option<Price>,
    pub unrealized_pnl: Option<Decimal>,
    pub realized_pnl: Decimal,
    pub total_fees: Decimal,
}

impl Position {
    pub fn side(&self) -> PositionSide;  // Long | Short | Flat
    pub fn is_flat(&self) -> bool;
    pub fn abs_qty(&self) -> Decimal;
    pub fn current_pnl(&self) -> Decimal; // realized + unrealized - fees
}
```

---

## InstrumentMeta

Instrument metadata for price/quantity rounding. Access via `ctx.instrument_meta(&instrument_id)`.

```rust
pub struct InstrumentMeta {
    pub instrument_id: InstrumentId,
    pub market_index: MarketIndex,
    pub base_asset: AssetId,
    pub quote_asset: AssetId,
    pub tick_size: Decimal,       // Price precision (e.g. 0.1 for BTC)
    pub lot_size: Decimal,        // Qty precision (e.g. 0.001)
    pub min_qty: Option<Decimal>, // Minimum order quantity
    pub min_notional: Option<Decimal>,
    pub fee_asset_default: Option<AssetId>,
    pub kind: InstrumentKind,     // Spot | Perp
}

impl InstrumentMeta {
    pub fn round_price(&self, price: Price) -> Price;
    pub fn round_qty(&self, qty: Qty) -> Qty;
    pub fn trunc_qty(&self, qty: Qty) -> Qty;  // Floor for sells
}
```

---

## LiveOrder

Engine's view of an in-flight order.

```rust
pub struct LiveOrder {
    pub client_id: ClientOrderId,
    pub exchange_order_id: Option<ExchangeOrderId>,
    pub instrument: InstrumentId,
    pub side: OrderSide,
    pub price: Price,
    pub requested_qty: Qty,
    pub filled_qty: Qty,
    pub avg_fill_px: Option<Price>,
    pub status: OrderStatus,    // New | Accepted | PartiallyFilled | Filled | Canceled | Rejected
    pub ts_created: i64,
    pub ts_last_update: i64,
}
```

---

## Order Enums

```rust
pub enum OrderSide { Buy, Sell }
pub enum OrderType { Limit, Market }
pub enum TimeInForce { Gtc, Ioc, Fok }  // Default: Gtc
pub enum OrderStatus {
    New, Accepted, PartiallyFilled,
    Filled,    // terminal
    Canceled,  // terminal
    Rejected,  // terminal
}
```

---

## Market Enum

Unified market configuration. Used in config to specify "where to trade."

```rust
#[serde(tag = "exchange")]
pub enum Market {
    #[serde(rename = "hyperliquid")]
    Hyperliquid(HyperliquidMarket),
}
```

### HyperliquidMarket Variants

```rust
#[serde(tag = "type")]
pub enum HyperliquidMarket {
    #[serde(rename = "perp")]
    Perp { base: String, quote: String, index: u32, instrument_meta: Option<InstrumentMetaConfig> },

    #[serde(rename = "spot")]
    Spot { base: String, quote: String, index: u32, instrument_meta: Option<InstrumentMetaConfig> },

    #[serde(rename = "hip3")]
    Hip3 { base: String, quote: String, dex: String, dex_index: u32, asset_index: u32, instrument_meta: Option<InstrumentMetaConfig> },
}
```

### JSON Examples

**Perp** (BTC perpetual):

```json
{ "exchange": "hyperliquid", "type": "perp", "base": "BTC", "index": 0 }
```

**Spot** (HYPE/USDC):

```json
{
  "exchange": "hyperliquid",
  "type": "spot",
  "base": "HYPE",
  "quote": "USDC",
  "index": 10107
}
```

**HIP-3** (builder DEX):

```json
{
  "exchange": "hyperliquid",
  "type": "hip3",
  "base": "BTC",
  "quote": "USDE",
  "dex": "hyna",
  "dex_index": 4,
  "asset_index": 1
}
```

### Market Methods

| Method                   | Returns                    | Description                                    |
| ------------------------ | -------------------------- | ---------------------------------------------- |
| `instrument_id()`        | `InstrumentId`             | `"BTC-PERP"`, `"HYPE-SPOT"`, `"hyna:BTC-PERP"` |
| `market_index()`         | `MarketIndex`              | Exchange-specific index                        |
| `is_spot()`              | `bool`                     | Whether spot market                            |
| `base()` / `quote()`     | `&str`                     | Base/quote asset                               |
| `exchange_instance(env)` | `ExchangeInstance`         | Build exchange instance                        |
| `hip3_config()`          | `Option<Hip3MarketConfig>` | HIP-3 specific config                          |
