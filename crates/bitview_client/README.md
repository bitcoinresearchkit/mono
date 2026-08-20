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

## Configuration

```rust,ignore
use bitview_client::{BitviewClient, BitviewClientOptions};

let client = BitviewClient::with_options(BitviewClientOptions {
    base_url: "https://bitview.space".to_string(),
    timeout_secs: 60,
});
```
