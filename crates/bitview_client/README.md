# bitview_client

Synchronous Rust client for the [Bitview](https://bitview.space) Bitcoin analytics API.

[crates.io](https://crates.io/crates/bitview_client) | [docs.rs](https://docs.rs/bitview_client)

AI clients can use the same API through the official stateless, read-only MCP
endpoint at [mcp.bitview.space](https://mcp.bitview.space/). No authentication
is required.

## Installation

```toml
[dependencies]
bitview_client = "0.11"
```

## Quick start

```rust,ignore
use bitview_client::{BitviewClient, Height, Index};

fn main() -> bitview_client::Result<()> {
    // Use the public API or point the client at a self-hosted Bitview server.
    let client = BitviewClient::new("https://bitview.space");

    let block_hash = client.get_block_by_height(Height::new(800_000))?;

    // Typed, chainable series access.
    let prices = client
        .series()
        .price
        .split
        .close
        .usd
        .by
        .day1()
        .last(30)
        .fetch()?;

    // Programmatic access when the series name is only known at runtime.
    let same_prices = client
        .series_endpoint("price_close", Index::Day1)
        .last(30)
        .fetch()?;

    Ok(())
}
```

## Missing and undefined values

A value is `None` where it is missing (mostly a date period without blocks) or
undefined (a NaN, or a ratio over zero). The types say exactly where:
`Option<T>` at the indexes where a value can be missing, and everywhere for value
types that can be undefined (floats, `Cents`, ratios, parts per million).

## Date and timestamp selectors

Date and timestamp selectors return `Result`: use
`endpoint.get_date(date)?.fetch()` (and likewise for `date_range`,
`get_timestamp`, and `timestamp_range`). Invalid or unsupported selectors fail
locally instead of silently selecting index zero. Numeric selectors are unchanged.

## Errors

An error answer from the server keeps its HTTP `status` and machine-readable `code` (an
`ErrorCode`) on `BitviewError`, next to the server's message; client-side failures (connection,
decoding, invalid selectors) have neither.

## Configuration

```rust,ignore
use bitview_client::{BitviewClient, BitviewClientOptions};

let client = BitviewClient::with_options(BitviewClientOptions {
    base_url: "https://bitview.space".to_string(),
    timeout_secs: 60,
});
```
