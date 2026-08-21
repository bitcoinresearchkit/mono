# brk_oracle

**Version 4**

Pure on-chain BTC/USD price oracle. No exchange feeds, no external APIs. Derives the bitcoin price from transaction data alone. Tracks block by block from height 340,000 (January 2015) onward.

Inspired by [UTXOracle](https://utxo.live/oracle/) by [@SteveSimple](https://x.com/SteveSimple), which proved the concept. brk_oracle takes the same core insight and redesigns the algorithm for per-block resolution and rolling operation. See [comparison](#comparison-with-utxoracle) below.

## The signal

People buy bitcoin in round dollar amounts. Each purchase creates a transaction output whose satoshi value depends on the current price:

```
  $100 at  $50,000/BTC  →  200,000 sats
  $100 at $100,000/BTC  →  100,000 sats
```

Thousands of these round-dollar purchases happen every day: $10, $20, $50, $100, $200, $500. Plot every transaction output in a block on a log-scale histogram and clear spikes emerge at each round-dollar amount:

```
       $5  $10 $20 $50 $100 $200 $500  $1k       $5k $10k
        ↓   ↓   ↓   ↓    ↓    ↓    ↓    ↓         ↓   ↓
   │            ▌        ▌         ▌                  ▌
   │    ▌   █   █   ▌    █    ▌    █    █         ▌   █
   │   ▐█▌ ▐█▌  █▌  █   ▐█▌  ▐█   ▐█▌  ▐█▌  █    ▐█  ▐█▌
   │▄▄████▄███████▄▐█▌▄█████▄██▌▄█████▄███▄▐█▌▄▄▄███▄████▄▄
   └─────────────────────────────────────────────────────────→
                          log₁₀(satoshis)
```

On a log scale, when the price changes **all spikes shift together** by the same number of bins. A 2x price move always shifts the pattern by ~60 bins, whether bitcoin moves from $1k to $2k or from $50k to $100k:

```
  price × 2  →  sats ÷ 2  →  shift left by log₁₀(2) × 200 ≈ 60 bins

  $50k:  ···· █ ···· █ ···· █ ···· █ ····
  $100k: ·· █ ···· █ ···· █ ···· █ ······
              ◄── 60 bins ──►
```

The spacing between spikes is constant (set by the ratios between dollar amounts). Only the position changes. The oracle detects this pattern and reads the price from where it lands.

## How it works

The oracle tracks the price incrementally, block by block, starting from a known seed price. Each new block nudges the estimate. The search window is narrow (about 12 bins, or +15% / -12% in price), so the oracle can only follow gradual movement, not jump to an arbitrary price from scratch. This is by design: it makes the algorithm resistant to noise.

For each new block:

### 1. Filter outputs

Skip the coinbase transaction, and skip every output of a transaction carrying an `OP_RETURN`: that transaction is protocol machinery, not a dollar-denominated payment, so its payout amounts are not price signal. Below height 630,000, also skip every output of a transaction with more than 100 outputs: a large fan-out is a batch payout (exchange sweep, mixer), not a round-dollar payment, and the thin early signal needs it removed. At and above height 630,000, the transaction fan-out cap relaxes to 250 outputs so dense-chain payment activity remains visible while very large fan-outs cannot dominate one EMA slot. Then exclude noisy outputs: script types dominated by protocol activity (P2TR by default), dust below 1,000 sats, and round BTC amounts (0.01, 0.1, 1.0 BTC, etc.) that create false spikes unrelated to dollar purchases.

### 2. Build a log-scale histogram

Each remaining output becomes a bin index in a 2,400-bin histogram spanning 12 decades (1 sat to 10¹² sats):

```
  bin = round(log₁₀(sats) × 200)       200 bins per decade
```

### 3. Smooth over recent blocks

A single block has too few outputs for a clean signal. The oracle keeps a ring buffer of the last 12 block histograms and combines them into an exponential moving average (EMA) that weights recent blocks more heavily:

```
  EMA[bin] = Σ  weight(age) × histogram[age][bin]
             age=0..11

  weight(age) = α × (1 − α)^age         default α = 2/7 (~6-block span)
```

The EMA is recomputed from the ring buffer each block. This makes the oracle deterministic: since only the last 12 histograms matter, any oracle started from a known price converges to the exact same state after 12 blocks, regardless of prior history. This is what makes checkpointing and restoring possible.

### 4. Score with a 19-point stencil

The fixed ratios between round-dollar amounts ($1, $2, $3, $5, ... $10,000) create a fingerprint: a pattern of 19 spikes with known spacing on the log scale. A stencil encodes this spacing as bin offsets from a $100 reference point:

```
   $1       $5     $10          $50  $100  $200        $1k          $10k
    ↓        ↓      ↓            ↓     ↓     ↓          ↓             ↓
    ·────────·──────·────────────·─────·─────·──────────·─────────────·
  -400     -260   -200          -60    0    +60       +200          +400
                      bin offsets from the $100 reference point
                                 (19 offsets total)
```

The oracle slides this stencil across the EMA histogram within the search window. At each candidate position:

1. **Read** the EMA value at all 19 expected spike locations
2. **Normalize** each value by dividing by that offset's peak within the search window: this gives rare amounts like $3 equal voting weight to common amounts like $100
3. **Sum** the 19 normalized values into a single score

The position with the highest score is where the fingerprint best matches the histogram.

### 5. Convert bin to price

A $100 purchase at price P produces `$100 / P × 10⁸` sats, which lands in bin:

```
  bin = log₁₀($100 / P × 10⁸) × 200
      = (2 + 8 − log₁₀(P)) × 200
      = (10 − log₁₀(P)) × 200
```

So the stencil's winning position, the bin where $100 purchases land, directly encodes the price:

```
  price = 10^(10 − bin / 200)  dollars
```

Parabolic interpolation between the best bin and its two neighbors refines the estimate to sub-bin precision.

## Pipeline

```
  block ──→ filter ──→ histogram ──→ ring buffer ──→ EMA ──→ stencil ──→ best bin ──→ $
             outputs     2,400 bins       ×12                  19-point    parabolic
                          log-scale                             scoring   interpolation
```

## Input

The oracle consumes one pre-built histogram per block via `process_histogram(&hist)`, a `[u32; 2400]` bin-count array, and returns the updated reference bin.

The caller filters as it builds the histogram, applying the [step 1](#1-filter-outputs) rules. `PaymentFilter::for_height(height).histogram(txs)` builds a fresh block histogram from non-coinbase transaction outputs. Incremental live callers use `PaymentFilter::MODERN.for_each_bin(outputs, emit)`, which applies the modern fan-out cap without requiring a height. `PaymentFilter::eligible_bin(sats, output_type)` returns an individual output's bin index, or `None` if filtered. The transaction-level rules include the OP_RETURN drop, the >100 transaction-output fan-out cap below height 630,000, and the >250 cap from height 630,000 onward.

The initial seed must be close to the real price at the starting height. The crate includes typed pre-oracle helpers for exchange prices at heights 0..340,000. `Oracle::from_seed()` uses the last baked price, height 339,999 (one below `START_HEIGHT_SLOW`), and the slow cold-start config to seed the oracle's first on-chain computation at height 340,000.

## Configuration

All parameters via `Config` with sensible defaults:

| Parameter | Default | Purpose |
|-----------|---------|---------|
| `alpha` | 2/7 | EMA decay rate (~6-block span) |
| `window_size` | 12 | Ring buffer depth in blocks |
| `search_below` / `search_above` | 12 / 11 | Search window around previous estimate (bins) |
| `shape_weight` | 0 | Shape-anchoring restoring-force weight. 0 disables it. `Config::slow()` sets 8 for the cold-start |

The output-filtering rules (1,000-sat dust floor, excluded P2TR, round-BTC exclusion) are not `Config` parameters: they are constants in the `filter` module so the indexer, per-request reconstruction, and mempool all bin identically. See [Input](#input).

Between heights 340,000 and 508,000 the oracle runs a slower cold-start configuration (`Config::slow()`: `alpha` = 0.10, ~19-block span, `window_size` = 40, `shape_weight` = 8). In the thin pre-2018 output mix the fast default octave-locks onto the round-dollar half-price pattern, so the slow EMA and the shape-anchoring restoring force resist that drift. At 508,000 `Oracle::reconfigure` switches to the defaults above (`shape_weight` back to 0), and `Config::for_height` returns the right one for any height.

## Comparison with UTXOracle

[UTXOracle](https://utxo.live/oracle/) by [@SteveSimple](https://x.com/SteveSimple) proved that BTC/USD can be derived purely from on-chain data. Both projects share the same core insight (round-dollar detection via log-scale histogram) but make different engineering choices:

| | brk_oracle | UTXOracle |
|---|---|---|
| Resolution | Per-block (~10 min); daily OHLC built downstream | Per-run consensus price + per-output intraday scatter |
| Operation | Rolling: EMA over ring buffer, updates each block | Batch: processes a full day from scratch, stateless |
| Algorithm | Single-pass stencil scoring with per-offset normalization | Multi-step: dual stencil → rough estimate → output-to-USD mapping → iterative convergence |
| Steps to compute price | 7 (filter+bin → ring insert → EMA → per-offset peaks → score → argmax+parabolic → bin→price) | 10 (filter+bin → clip → smooth round BTC → sum → normalize → cap extremes → dual-stencil slide → neighbor weight-avg → output-to-USD map → iterative central price) |
| Stencil | 19 round-USD offsets ($1 to $10k), each normalized to its own peak | 803-point Gaussian + weighted spike template targeting 17 round-USD amounts |
| Round BTC handling | Excluded from histogram entirely | Histogram bins smoothed by averaging neighbors |
| Output filtering | Per-tx OP_RETURN drop, then per-output: script type, dust threshold, round BTC | Per-tx: not coinbase, no OP_RETURN, exactly 2 outputs, ≤5 inputs, no same-day inputs, ≤500-byte witness |
| Validated from | Height 340,000 (January 2015) | Dec 15, 2023 |
| Language | Rust | Python |
| Dependencies | None (pure computation, caller provides block data) | bitcoin-cli + direct blk file reads |
| Bins per decade | 200 | 200 |

## Accuracy

Tested over 596,251 exchange-covered blocks after running the oracle from height 340,000 through height 952,314. Error is measured per block as distance from the oracle estimate to the exchange high/low range at that height. If the oracle falls within the range, the error is zero.

### Per-block

| Metric | Value |
|--------|-------|
| Median error | 0.15% |
| 95th percentile | 1.2% |
| 99th percentile | 3.4% |
| 99.9th percentile | 15.6% |
| RMSE | 0.97% |
| Max error | 33.8% |
| Bias | +0.06 bins (essentially zero) |
| Blocks > 5% error | 3,233 (0.542%) |
| Blocks > 10% error | 1,323 |
| Blocks > 20% error | 154 |

### Daily candles

Oracle daily OHLC built from per-block prices vs exchange daily OHLC:

| | Median | RMSE | Max |
|-------|--------|------|-----|
| Open | 0.24% | 1.07% | 29.1% |
| High | 0.58% | 1.47% | 27.3% |
| Low | 0.53% | 1.95% | 55.1% |
| Close | 0.27% | 1.18% | 29.2% |

### By year

| Year | Blocks | Median | RMSE | Max | >5% | >10% | >20% | Price range |
|------|--------|--------|------|-----|-----|------|------|-------------|
| 2015 | 51,249 | 0.26% | 1.67% | 33.8% | 916 | 449 | 25 | $198–$500 |
| 2016 | 54,753 | 0.33% | 0.80% | 16.9% | 150 | 33 | 0 | $351–$989 |
| 2017 | 55,959 | 0.45% | 2.05% | 28.6% | 1,527 | 606 | 67 | $0–$19,892 |
| 2018 | 54,531 | 0.18% | 1.31% | 31.6% | 411 | 207 | 62 | $3,129–$17,178 |
| 2019 | 54,272 | 0.16% | 0.59% | 17.4% | 100 | 16 | 0 | $3,338–$13,868 |
| 2020 | 53,102 | 0.10% | 0.42% | 11.6% | 61 | 3 | 0 | $3,858–$29,322 |
| 2021 | 52,733 | 0.07% | 0.47% | 14.4% | 42 | 9 | 0 | $27,678–$69,000 |
| 2022 | 53,230 | 0.07% | 0.32% | 6.8% | 10 | 0 | 0 | $15,460–$48,240 |
| 2023 | 54,032 | 0.10% | 0.25% | 6.6% | 5 | 0 | 0 | $16,490–$44,700 |
| 2024 | 53,367 | 0.10% | 0.28% | 6.7% | 7 | 0 | 0 | $38,555–$108,298 |
| 2025 | 53,113 | 0.11% | 0.25% | 5.8% | 4 | 0 | 0 | $74,409–$126,198 |
| 2026 | 5,910 | 0.10% | 0.27% | 3.2% | 0 | 0 | 0 | $60,000–$97,900 |

The oracle is only as good as the signal it reads. The largest errors cluster in the early cold-start, where thin 2015 on-chain volume gives a weaker round-dollar pattern: the 33.8% max error sits at height 341,498 (oracle ~$287 vs exchange ~$213) during the first weeks of warm-up. A second cluster sits just below the 508,000 regime switch, where the slow EMA lagged the fast early-2018 rally (~31.6% at height 507,278, oracle ~$6,685 vs exchange ~$8,800) before handing off to the fast default. The thin pre-2018 mix means 2015, 2017, and 2018 carry the bulk of the error (1.67%, 2.05%, and 1.31% RMSE). From 2019 the signal strengthens: by 2020 the oracle reaches 0.1% median accuracy, and since 2022 no block exceeds 10% error.

### Why no outlier smoothing?

Post-hoc smoothing, for example correcting any block whose price deviates more than 5% from both its neighbors, would improve the aggregate numbers. This is deliberately not done, for two reasons:

1. **Simplicity**: The oracle is a single forward pass with no lookback corrections. Adding smoothing means defining thresholds, neighbor windows, and replacement strategies, all of which add complexity for marginal gain.
2. **Finality**: Each block's price is produced once and never revised (unless the block itself is reorged). Downstream consumers can treat the oracle output as append-only. Smoothing would require retroactively changing already-published prices, breaking that property.

## Changelog

### v4

Changes from v3:

- **Modern fan-out cap**: below height 630,000 the oracle keeps the strict >100-output transaction drop introduced in v3. At and above 630,000 the cap now relaxes to 250 outputs instead of being fully lifted. This preserves dense-chain payment signal while preventing very large modern fan-outs from dominating a single EMA slot and creating a transient false round-dollar ladder.

`VERSION` is bumped to 4 so downstream consumers invalidate prices computed by an earlier algorithm.

### v3

Changes from v2:

- **Earlier start with a cold-start regime**: on-chain tracking begins at height 340,000 (January 2015) instead of 525,000, adding about 185,000 blocks of history. Below height 508,000 the oracle runs a slower EMA (`Config::slow()`, ~19-block span, window 40) paired with a shape-anchoring restoring force (`shape_weight` 8) that pulls candidate scores toward a slowly-adapted profile of the round-dollar arm shape, resisting the half-price octave drift the fast default locks onto in the thinner pre-2018 output mix. At height 508,000 it switches to the fast default via `Oracle::reconfigure`, which restores `shape_weight` to 0 and turns the force off.
- **Max-outputs filter**: a transaction with more than 100 outputs is dropped from the histogram below height 630,000. Large fan-outs (exchange sweeps, mixer payouts) are batch machinery, not round-dollar payments, and the thin 2018-2020 signal needs them removed to stay locked onto the pattern. Above 630,000 on-chain volume is dense enough that the cap removes more genuine signal than noise, so it is lifted.
- **Wider up-reach**: `search_below` raised from 9 to 12 bins. The sharp 2018 reversal candles need extra room to follow a fast move upward in price.

`VERSION` is bumped to 3 so downstream consumers invalidate prices computed by an earlier algorithm.

### v2

Changes from v1:

- **OP_RETURN filter**: every output of a transaction carrying an `OP_RETURN` is now dropped from the histogram. Such transactions are protocol machinery (cross-chain swaps, anchoring) whose payout amounts can form false round-dollar patterns. This was the trigger for the worst price glitches in v1.
- **P2WSH reactivated**: once the OP_RETURN filter removes the protocol noise, P2WSH outputs are usable round-dollar signal again, so they are no longer excluded. P2TR stays excluded.
- **Earlier start**: on-chain tracking begins at height 525,000 (May 2018) instead of 550,000, adding about 25,000 blocks of history.

`VERSION` is exposed as a crate constant so downstream consumers can invalidate prices computed by an earlier algorithm.
