//! Runtime state for the Simple strategy.

use bot_core::ClientOrderId;

/// Tracks which phase the strategy is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// No position — waiting to buy
    WaitingToBuy,
    /// Buy order placed, waiting for fill
    BuyPlaced,
    /// Holding position — waiting to sell
    WaitingToSell,
    /// Sell order placed, waiting for fill
    SellPlaced,
}

pub struct SimpleState {
    pub phase: Phase,
    pub active_order: Option<ClientOrderId>,
}

impl SimpleState {
    pub fn new() -> Self {
        Self {
            phase: Phase::WaitingToBuy,
            active_order: None,
        }
    }
}
