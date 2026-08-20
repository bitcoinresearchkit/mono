# bitview_cohort

UTXO and address cohort filtering for on-chain analytics.

## What It Enables

Slice the UTXO set and address population by age, amount, output type, halving epoch, or holder classification (STH/LTH). Build complex cohorts by combining filters for metrics like "realized cap of 1+ BTC UTXOs older than 150 days."

## Key Features

- **Age-based**: `TimeFilter::GreaterOrEqual(hours)`, `TimeFilter::Range(hours..hours)`, `TimeFilter::LowerThan(hours)`
- **Amount-based**: `AmountFilter::GreaterOrEqual(Sats::_1BTC)`, `AmountFilter::Range(Sats::_100K..Sats::_1M)`
- **Term classification**: `Term::Sth` (short-term holders, <150 days), `Term::Lth` (long-term holders)
- **Epoch filters**: Group by halving epoch
- **Type filters**: Segment by output type (P2PKH, P2TR, etc.)
- **Context-aware naming**: Automatic prefix generation (`utxos_`, `addrs_`) based on cohort context
- **Inclusion logic**: Filter hierarchy for aggregation (`Filter::includes`)

## Filter Types

```rust,ignore
pub enum Filter {
    All,
    Term(Term),           // STH/LTH
    Time(TimeFilter),     // Age-based
    Amount(AmountFilter), // Value-based
    Epoch(Halving),  // Halving epoch
    Year(Year),           // Calendar year
    Type(OutputType),     // P2PKH, P2TR, etc.
}
```

## Core API

```rust,ignore
// TimeFilter values are in hours (e.g., 3600 hours = 150 days)
let filter = Filter::Time(TimeFilter::GreaterOrEqual(3600));

// Check membership
filter.contains_time(4000);  // true (4000 hours > 3600 hours)
filter.contains_amount(sats);

// Generate metric names (via CohortContext)
let ctx = CohortContext::Utxo;
ctx.full_name(&filter, "min_age_150d");  // "utxos_min_age_150d"
```

## Built On

- `brk_error` for error handling
- `brk_types` for `Sats`, `Halving`, `OutputType`
- `bitview_traversable` for data structure traversal
