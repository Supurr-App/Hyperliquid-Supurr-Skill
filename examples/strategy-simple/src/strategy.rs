//! Simple Buy-Low-Sell-High strategy implementation.
//!
//! Lifecycle:
//!   WaitingToBuy → BuyPlaced → WaitingToSell → SellPlaced → WaitingToBuy (repeat)

use crate::config::SimpleConfig;
use crate::state::{Phase, SimpleState};
use bot_core::*;

pub struct SimpleStrategy {
    config: SimpleConfig,
    state: SimpleState,
    meta: Option<InstrumentMeta>,
}

impl SimpleStrategy {
    pub fn new(config: SimpleConfig) -> Self {
        Self {
            config,
            state: SimpleState::new(),
            meta: None,
        }
    }

    fn exchange(&self) -> ExchangeInstance {
        self.config
            .market
            .exchange_instance(self.config.environment)
    }

    fn instrument(&self) -> InstrumentId {
        self.config.market.instrument_id()
    }

    fn place_buy(&mut self, ctx: &mut dyn StrategyContext) {
        let price = self
            .meta
            .as_ref()
            .unwrap()
            .round_price(Price::new(self.config.buy_price));
        let qty = self
            .meta
            .as_ref()
            .unwrap()
            .round_qty(Qty::new(self.config.order_size));
        let order = PlaceOrder::limit(
            self.exchange(),
            self.instrument(),
            OrderSide::Buy,
            price,
            qty,
        );
        self.state.active_order = Some(order.client_id.clone());
        self.state.phase = Phase::BuyPlaced;
        ctx.place_order(order);
        ctx.log_info(&format!("BUY order placed @ {}", price));
    }

    fn place_sell(&mut self, ctx: &mut dyn StrategyContext) {
        let price = self
            .meta
            .as_ref()
            .unwrap()
            .round_price(Price::new(self.config.sell_price));
        let qty = self
            .meta
            .as_ref()
            .unwrap()
            .trunc_qty(Qty::new(self.config.order_size));
        let order = PlaceOrder::limit(
            self.exchange(),
            self.instrument(),
            OrderSide::Sell,
            price,
            qty,
        );
        self.state.active_order = Some(order.client_id.clone());
        self.state.phase = Phase::SellPlaced;
        ctx.place_order(order);
        ctx.log_info(&format!("SELL order placed @ {}", price));
    }
}

impl Strategy for SimpleStrategy {
    fn id(&self) -> &StrategyId {
        &self.config.strategy_id
    }

    fn on_start(&mut self, ctx: &mut dyn StrategyContext) {
        self.meta = ctx.instrument_meta(&self.instrument()).cloned();
        if self.meta.is_none() {
            ctx.stop_strategy(self.config.strategy_id.clone(), "Instrument not found");
            return;
        }
        let errors = self.config.validate();
        if !errors.is_empty() {
            ctx.stop_strategy(self.config.strategy_id.clone(), &errors.join("; "));
            return;
        }
        ctx.log_info(&format!(
            "SimpleStrategy started: buy@{} sell@{} qty={}",
            self.config.buy_price, self.config.sell_price, self.config.order_size
        ));
        // Place initial buy order
        self.place_buy(ctx);
    }

    fn on_event(&mut self, ctx: &mut dyn StrategyContext, event: &Event) {
        match event {
            Event::OrderCompleted(c) => match self.state.phase {
                Phase::BuyPlaced => {
                    ctx.log_info(&format!("Buy filled @ avg={:?}", c.avg_fill_px));
                    self.place_sell(ctx);
                }
                Phase::SellPlaced => {
                    ctx.log_info(&format!(
                        "Sell filled @ avg={:?} — cycle complete!",
                        c.avg_fill_px
                    ));
                    self.place_buy(ctx);
                }
                _ => {}
            },
            Event::OrderCanceled(_) | Event::OrderRejected(_) => {
                ctx.log_warn("Order canceled/rejected — resetting to buy phase");
                self.state.active_order = None;
                self.state.phase = Phase::WaitingToBuy;
                self.place_buy(ctx);
            }
            _ => {}
        }
    }

    fn on_timer(&mut self, _ctx: &mut dyn StrategyContext, _timer_id: TimerId) {}

    fn on_stop(&mut self, ctx: &mut dyn StrategyContext) {
        ctx.cancel_all(CancelAll::new(self.exchange()));
        ctx.log_info("SimpleStrategy stopped");
    }
}
