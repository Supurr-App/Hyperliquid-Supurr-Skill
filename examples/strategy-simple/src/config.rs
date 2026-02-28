//! Configuration for the Simple strategy.

use bot_core::{Environment, Market, StrategyId};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SimpleConfig {
    pub strategy_id: StrategyId,
    pub environment: Environment,
    pub market: Market,

    /// Price at which to place a BUY order
    pub buy_price: Decimal,
    /// Price at which to place a SELL order (take profit)
    pub sell_price: Decimal,
    /// Order quantity in base asset
    pub order_size: Decimal,
}

impl SimpleConfig {
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.buy_price >= self.sell_price {
            errors.push("buy_price must be < sell_price".into());
        }
        if self.order_size <= Decimal::ZERO {
            errors.push("order_size must be > 0".into());
        }
        errors
    }
}
