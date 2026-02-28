//! Strategy configuration.
//!
//! This struct is deserialized from the JSON config file.
//! Derive `JsonSchema` so the CLI can generate a JSON schema for validation.

use bot_core::{Environment, Market, StrategyId};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configuration for MyStrategy.
///
/// TODO: Add your strategy-specific parameters below.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MyConfig {
    /// Unique strategy identifier (e.g., "btc-mystrategy")
    pub strategy_id: StrategyId,

    /// Trading environment (Mainnet or Testnet)
    pub environment: Environment,

    /// Market to trade on — single source of truth for exchange/instrument/index
    pub market: Market,

    // -------------------------------------------------------------------------
    // TODO: Add your strategy-specific parameters here.
    // Examples:
    // -------------------------------------------------------------------------

    /// Order size in base asset (e.g., 0.01 BTC)
    pub order_size: Decimal,

    // pub spread_pct: Decimal,
    // pub max_position: Decimal,
    // pub rebalance_interval_secs: u64,
}

impl MyConfig {
    /// Validate configuration. Return a list of error messages (empty = valid).
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.order_size <= Decimal::ZERO {
            errors.push("order_size must be > 0".into());
        }

        // TODO: Add your validation rules here.

        errors
    }
}
