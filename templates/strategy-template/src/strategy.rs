//! Strategy implementation.
//!
//! This is where the trading logic lives. Implement the 4 Strategy callbacks:
//! - on_start: Initialize state, set timers, validate config
//! - on_event: React to price changes, order fills, cancellations
//! - on_timer: Handle periodic logic
//! - on_stop: Cancel all orders, log final state

use crate::config::MyConfig;
use crate::state::MyState;
use bot_core::*;

/// MyStrategy trading strategy.
///
/// TODO: Rename to match your strategy (e.g., MomentumStrategy, MeanRevertStrategy).
pub struct MyStrategy {
    config: MyConfig,
    state: MyState,
    instrument_meta: Option<InstrumentMeta>,
}

impl MyStrategy {
    pub fn new(config: MyConfig) -> Self {
        Self {
            config,
            state: MyState::new(),
            instrument_meta: None,
        }
    }

    /// Round price to tick size and 5 significant figures.
    fn round_price(&self, price: Price) -> Price {
        let trimmed = price.trim_to_sig_figs(5);
        if let Some(ref meta) = self.instrument_meta {
            meta.round_price(trimmed)
        } else {
            trimmed
        }
    }

    /// Round quantity to lot size.
    fn round_qty(&self, qty: Qty) -> Qty {
        if let Some(ref meta) = self.instrument_meta {
            meta.round_qty(qty)
        } else {
            qty
        }
    }

    /// Get the exchange instance for order commands.
    fn exchange_instance(&self) -> ExchangeInstance {
        self.config
            .market
            .exchange_instance(self.config.environment)
    }

    /// Get the instrument ID.
    fn instrument_id(&self) -> InstrumentId {
        self.config.market.instrument_id()
    }
}

impl Strategy for MyStrategy {
    fn id(&self) -> &StrategyId {
        &self.config.strategy_id
    }

    fn on_start(&mut self, ctx: &mut dyn StrategyContext) {
        // Load instrument metadata (tick size, lot size, etc.)
        let instrument = self.instrument_id();
        self.instrument_meta = ctx.instrument_meta(&instrument).cloned();

        if self.instrument_meta.is_none() {
            ctx.log_error(&format!("Instrument not found: {}", instrument));
            ctx.stop_strategy(self.config.strategy_id.clone(), "Instrument not found");
            return;
        }

        // Validate config
        let errors = self.config.validate();
        if !errors.is_empty() {
            for err in &errors {
                ctx.log_error(&format!("Config error: {}", err));
            }
            ctx.stop_strategy(
                self.config.strategy_id.clone(),
                &format!("Config validation failed: {}", errors.join("; ")),
            );
            return;
        }

        ctx.log_info(&format!(
            "MyStrategy started: {} order_size={}",
            instrument, self.config.order_size
        ));

        // TODO: Set up timers, initialize state, etc.
        // Example: ctx.set_interval(Duration::from_secs(30));

        self.state.is_initialized = true;
    }

    fn on_event(&mut self, ctx: &mut dyn StrategyContext, event: &Event) {
        if !self.state.is_initialized {
            return;
        }

        match event {
            Event::Quote(q) => {
                let mid = q.mid();
                self.state.last_mid = Some(mid);

                // TODO: Your price-reactive trading logic here.
                // Example: Check spread, place orders, rebalance grid, etc.
            }
            Event::OrderFilled(f) => {
                ctx.log_info(&format!(
                    "Filled: {} {} @ {} qty={}",
                    f.side, f.client_id, f.price, f.qty
                ));

                // TODO: Handle fill — update state, place counter-order, etc.
            }
            Event::OrderCompleted(c) => {
                ctx.log_info(&format!(
                    "Completed: {} filled_qty={}",
                    c.client_id, c.filled_qty
                ));

                // TODO: Order fully filled — cycle logic, place next order, etc.
                self.state.active_order = None;
            }
            Event::OrderCanceled(c) => {
                ctx.log_info(&format!("Canceled: {}", c.client_id));
                self.state.active_order = None;

                // TODO: Handle cancel — retry, reset level, etc.
            }
            Event::OrderRejected(r) => {
                ctx.log_warn(&format!("Rejected: {} reason={}", r.client_id, r.reason));
                self.state.active_order = None;

                // TODO: Handle rejection — adjust price/qty, log error, etc.
            }
            Event::ExchangeStateChanged(e) => {
                ctx.log_info(&format!(
                    "Exchange state: {:?} -> {:?} ({})",
                    e.old_state, e.new_state, e.reason
                ));
                // Pause order placement when Halted
            }
            _ => {}
        }
    }

    fn on_timer(&mut self, ctx: &mut dyn StrategyContext, _timer_id: TimerId) {
        // TODO: Periodic logic — status logging, rebalancing, health checks, etc.

        // Example: periodic status log
        let now = ctx.now_ms();
        if now - self.state.last_log_ts > 30_000 {
            if let Some(mid) = self.state.last_mid {
                ctx.log_info(&format!(
                    "Status: mid={} active_order={:?}",
                    mid, self.state.active_order
                ));
            }
            self.state.last_log_ts = now;
        }
    }

    fn on_stop(&mut self, ctx: &mut dyn StrategyContext) {
        ctx.log_info("MyStrategy stopping — canceling all orders");
        ctx.cancel_all(CancelAll::new(self.exchange_instance()));
    }
}
