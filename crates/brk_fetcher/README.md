# brk_fetcher

Bitcoin price data fetcher with multi-source fallback.

## What It Enables

Fetch OHLC (Open/High/Low/Close) price data from Binance, Kraken, or BRK's own API. Automatically falls back between sources on failure, with 12-hour retry persistence for transient network issues.

## Key Features

- **Multi-source fallback**: Binance → Kraken → Bitview API
- **Health tracking**: Temporarily disables failing sources
- **Two resolution modes**: Per-date (daily) or per-block (1-minute interpolated)
- **HAR file support**: Import Binance 1mn data from browser network captures for historical fills
- **Permanent block detection**: Stops retrying on DNS/TLS failures

## Core API

```rust,ignore
let mut fetcher = Fetcher::import(Some(&hars_path))?;

// Daily price
let ohlc = fetcher.get_date(Date::new(2024, 4, 20))?;

// Block-level price (uses 1mn data when available)
let ohlc = fetcher.get_height(height, block_timestamp, prev_timestamp)?;
```

## Sources

| Source | Resolution | Lookback | Notes |
|--------|------------|----------|-------|
| Binance | 1mn | ~16 hours | Best for recent blocks |
| Kraken | 1mn | ~10 hours | Fallback for recent |
| Bitview API | Daily | Full history | Fallback for older data |

## HAR Import

For historical 1-minute data beyond API limits, export network requests from Binance's web interface and place the HAR file in the imports directory.

## Built On

- `brk_error` for error handling
- `brk_logger` for retry logging
- `brk_types` for `Date`, `Height`, `Timestamp`, `OHLCCents`
