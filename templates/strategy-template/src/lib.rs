//! Custom Trading Strategy
//!
//! Replace "MyStrategy" with your strategy name throughout.
//! See STRATEGY_API.md for the full API contract.

mod config;
mod state;
mod strategy;

pub use config::*;
pub use state::*;
pub use strategy::*;
