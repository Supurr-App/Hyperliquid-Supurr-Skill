# Prediction Market (HIP-4 Outcome) Trading

Supurr supports Hyperliquid binary outcome markets as spot-like markets. Agents should use live metadata, produce a safe config, and keep the user out of manual ID/encoding work.

## Agent Checklist

| Step | Rule |
| --- | --- |
| Find market | Query `outcomeMeta` on the selected environment. Never invent `outcome_id`. |
| Pick side | `0 = Yes`, `1 = No`. Confirm the side in plain language. |
| Price range | Outcomes trade from `0` to `1`. Reject grid/DCA prices outside this range. |
| Quote balance | Outcomes quote in `USDH`; user needs `USDH` in spot balance. |
| Risk | No leverage, no funding, no liquidation. Max loss is premium paid. |
| Expiry | Explain settlement/expiry risk. Do not add custom deadline automation unless product explicitly supports it. |
| Strategy | Treat outcome as a market type. Grid, DCA, and custom strategies should not need outcome-specific hacks. |
| Neutral grid | Use `supurr new orchestrator --kind prediction-yes-no-grid`, not one outcome grid in neutral mode. |

## Protocol Facts

| Property | Value |
| --- | --- |
| Market type | `outcome` |
| Info endpoint | `{"type":"outcomeMeta"}` |
| Side | `0 = Yes`, `1 = No` |
| Encoding | `10 * outcome_id + side` |
| Coin key | `#<encoding>` |
| Token name | `+<encoding>` |
| Order asset ID | `100000000 + encoding` |
| Balance source | `spotClearinghouseState` |
| Quote | `USDH` |
| Leverage | None |

Example:

```text
Outcome: BTC > 79980
outcome_id: 2
side: 0
encoding: 20
coin: #20
asset_id: 100000020
```

## Find Live Outcomes

Mainnet:

```bash
curl -s https://api.hyperliquid.xyz/info \
  -H 'Content-Type: application/json' \
  -d '{"type":"outcomeMeta"}' | python3 -m json.tool
```

Testnet:

```bash
curl -s https://api.hyperliquid-testnet.xyz/info \
  -H 'Content-Type: application/json' \
  -d '{"type":"outcomeMeta"}' | python3 -m json.tool
```

Pick an active outcome from this response. The exact IDs rotate, so old configs are only examples.

## Check Book

```bash
# Replace #20 with "#" + (10 * outcome_id + side)
curl -s https://api.hyperliquid.xyz/info \
  -H 'Content-Type: application/json' \
  -d '{"type":"l2Book","coin":"#20"}' | python3 -m json.tool
```

Use the live bid/ask to choose range and order size. Thin books need smaller sizes.

## Grid Config Example

```json
{
  "environment": "mainnet",
  "strategy_type": "grid",
  "markets": [
    {
      "exchange": "hyperliquid",
      "type": "outcome",
      "name": "BTC > 79980",
      "outcome_id": 2,
      "side": 0,
      "instrument_meta": {
        "tick_size": "0.001",
        "lot_size": "1",
        "min_qty": "1",
        "min_notional": "10"
      }
    }
  ],
  "grid": {
    "mode": "long",
    "levels": 5,
    "start_price": "0.42",
    "end_price": "0.50",
    "max_investment_quote": "80",
    "leverage": "1",
    "max_leverage": "1",
    "post_only": true
  }
}
```

## DCA Config Example

```json
{
  "environment": "mainnet",
  "strategy_type": "dca",
  "markets": [
    {
      "exchange": "hyperliquid",
      "type": "outcome",
      "name": "BTC > 79980",
      "outcome_id": 2,
      "side": 0,
      "instrument_meta": {
        "tick_size": "0.001",
        "lot_size": "1",
        "min_qty": "1",
        "min_notional": "10"
      }
    }
  ],
  "dca": {
    "direction": "long",
    "trigger_price": "0.50",
    "base_order_size": "20",
    "dca_order_size": "20",
    "max_dca_orders": 2,
    "size_multiplier": "1.5",
    "price_deviation_pct": "5",
    "deviation_multiplier": "1.0",
    "take_profit_pct": "5",
    "leverage": "1",
    "max_leverage": "1",
    "restart_on_complete": false,
    "cooldown_period_secs": 60
  }
}
```

## Neutral YES/NO Orchestrator

Use this for neutral prediction grids. It is two long grids under one bot identity:

| Leg | Coin | Action |
| --- | --- | --- |
| YES | `#<10*outcome_id>` | Buy YES below the YES market, then sell YES TP after fill |
| NO | `#<10*outcome_id+1>` | Buy NO below the NO market, then sell NO TP after fill |

Rules:
- Query `outcomeMeta`; never infer prediction markets from `meta` or `perpDexs`.
- Agent/user supplies explicit YES and NO ranges from live `allMids` and both `l2Book`s.
- If the user wants three active buy levels below current price, use `--levels 4`; active buys are `levels - 1`.
- `--investment` is total group capital. Static v1 split is 50/50 across YES and NO.
- Backtest first. Deploy only after explicit user confirmation.

```bash
supurr new orchestrator \
  --kind prediction-yes-no-grid \
  --asset BTC \
  --outcome-id 7 \
  --yes-start 0.235 \
  --yes-end 0.265 \
  --no-start 0.610 \
  --no-end 0.650 \
  --levels 4 \
  --investment 120 \
  --take-profit 10 \
  --stop-loss 10 \
  --output btc-yes-no-neutral

supurr backtest -c btc-yes-no-neutral
supurr deploy -c btc-yes-no-neutral
```

## Sizing

| Input | Meaning |
| --- | --- |
| Price `0.50` | 50% implied probability on that side |
| Size `20` | 20 outcome tokens |
| Cost | `price * size`, paid in `USDH` |

Example: `20` tokens at `0.50` costs about `10 USDH` before fees/slippage.

## User-Intent Handoff

If the user has USDC in perps but needs USDH in spot, do not pretend the agent transferred it. Emit a user action intent from [User Action Intents](../references/user-action-intents.md), for example `perps_to_spot_transfer` or `spot_to_spot_token_transfer`.

## Troubleshooting

| Issue | Likely cause | Fix |
| --- | --- | --- |
| No mid price | Wrong `#<encoding>` | Recompute `10 * outcome_id + side` from current `outcomeMeta`. |
| Order rejected | Stale outcome or wrong asset ID | Refresh `outcomeMeta`; update `outcome_id`, side, and metadata. |
| Zero quote balance | No `USDH` in spot | Ask for a user action intent to fund spot `USDH`. |
| Price validation failure | Price outside `0..1` | Clamp or ask for a new range. |
| Rate limited `429` | Too many Hyperliquid calls | Wait and retry with backoff. |
