//! Simple Buy-Low-Sell-High Strategy
//!
//! A minimal working strategy (~100 lines) that demonstrates the full pattern:
//! - Buy when price drops below `buy_price`
//! - Sell when price rises above `sell_price`
//! - Tracks one position at a time (no grid, no scaling)

mod config;
mod state;
mod strategy;

pub use config::*;
pub use state::*;
pub use strategy::*;
