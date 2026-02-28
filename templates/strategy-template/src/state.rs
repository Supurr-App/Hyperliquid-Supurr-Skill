//! Strategy runtime state.
//!
//! This struct tracks everything the strategy needs at runtime.
//! Unlike config, this is NOT serialized — it's rebuilt on startup.

use bot_core::{ClientOrderId, Price};

/// Runtime state for MyStrategy.
///
/// TODO: Add your tracking fields here.
pub struct MyState {
    /// Whether the strategy has completed initialization
    pub is_initialized: bool,

    /// Last observed mid price
    pub last_mid: Option<Price>,

    /// Currently active order (if any)
    pub active_order: Option<ClientOrderId>,

    /// Last periodic log timestamp
    pub last_log_ts: i64,
    // TODO: Add your state fields here.
    // Examples:
    // pub total_fills: u32,
    // pub realized_pnl: Decimal,
    // pub order_registry: HashMap<String, usize>,
}

impl MyState {
    pub fn new() -> Self {
        Self {
            is_initialized: false,
            last_mid: None,
            active_order: None,
            last_log_ts: 0,
        }
    }
}

impl Default for MyState {
    fn default() -> Self {
        Self::new()
    }
}
