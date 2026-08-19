# brk_computer

Derived metrics computation engine for Bitcoin on-chain analytics.

## What It Enables

Compute 1000+ on-chain metrics from indexed blockchain data: supply breakdowns, realized/unrealized P&L, SOPR, MVRV, cohort analysis (by age, amount, address type), cointime economics, mining pool attribution, and price-weighted valuations.

## Key Features

- **Cohort metrics**: Filter by UTXO age (STH/LTH, age bands), amount ranges, address types
- **Stateful computation**: Track per-UTXO cost basis, realized/unrealized states
- **Multi-index support**: Metrics available by height, date, week, month, year, decade
- **Price integration**: USD-denominated metrics when price data available
- **Mining pool attribution**: Tag blocks/rewards to known pools
- **Cointime economics**: Liveliness, vaultedness, activity-weighted metrics
- **Incremental updates**: Resume from checkpoints, compute only new blocks

## Core API

```rust,ignore
let mut computer = Computer::forced_import(&outputs_path, &indexer)?;

// Compute all metrics for new blocks
computer.compute(&indexer, &exit)?;

// Access computed data via traversable vecs
let supply = computer.distribution.utxo_cohorts.all.metrics.supply.total.sats.height;
let realized_cap = computer.distribution.utxo_cohorts.all.metrics.realized.unwrap().realized_cap.height;
```

## Metric Categories

| Module | Examples |
|--------|----------|
| `blocks` | Block count, interval, size, mining metrics, rewards |
| `transactions` | Transaction count, fee, size, volume |
| `scripts` | Output type counts |
| `distribution` | Realized cap, MVRV, SOPR, unrealized P&L, supply |
| `cointime` | Liveliness, vaultedness, true market mean |
| `pools` | Per-pool block counts, rewards, fees |
| `market` | Market cap, NVT, Puell multiple |
| `price` | Height-to-price mapping from on-chain oracle |

## Cohort System

UTXO and address cohorts support filtering by:
- **Age**: STH (<150d), LTH (≥150d), 23 age bands (<1h, 1h-1d, 1d-1w, 1w-1m, 1m-2m, ..., 6m-9m, 9m-1y, 1y-18m, 18m-2y, ..., 12y-15y, 15y+)
- **Amount**: 0-0.001 BTC, 0.001-0.01, ..., 10k+ BTC
- **Type**: P2PK, P2PKH, P2MS, P2SH, P2WPKH, P2WSH, P2TR, P2A
- **Epoch**: By halving epoch

```rust,ignore
// Access metrics for a specific UTXO cohort (e.g. long-term holders)
let lth_supply = computer.distribution.utxo_cohorts.lth.metrics.supply.total.sats.height;

// Access metrics for an address cohort (e.g. 1-10 BTC holders)
let whale_count = computer.distribution.address_cohorts.from_1_to_10.metrics.address_count.height;

// Access metrics for all UTXOs combined
let sopr = computer.distribution.utxo_cohorts.all.metrics.realized.unwrap().sopr.height;
```

## Performance

### End-to-End

Full pipeline benchmarks (indexer + computer):

| Machine | Time | Disk | Peak Disk | Memory | Peak Memory |
|---------|------|------|-----------|--------|-------------|
| MBP M3 Pro (36GB, internal SSD) | 4.4h | 345 GB | 348 GB | 3.3 GB | 11 GB |
| Mac Mini M4 (16GB, external SSD) | 7h | 344 GB | 346 GB | 4 GB | 10 GB |

Full benchmark data: [bitcoinresearchkit/benches](https://github.com/bitcoinresearchkit/benches/tree/main/brk)

## Recommended: mimalloc v3

Use [mimalloc v3](https://crates.io/crates/mimalloc) as the global allocator to reduce memory usage.

## Built On

- `brk_indexer` for indexed blockchain data
- `brk_cohort` for cohort filtering
- `brk_oracle` for on-chain price data
- `brk_reader` for raw block access
- `bitview_traversable` for data export
