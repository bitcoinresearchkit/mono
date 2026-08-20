# bitview-client

Zero-dependency Python client for the [Bitview](https://bitview.space) Bitcoin analytics API.

Requires Python 3.9+.

[PyPI](https://pypi.org/project/bitview-client/) | [API Reference](https://github.com/bitcoinresearchkit/brk/blob/main/packages/bitview_client/DOCS.md) | [Source](https://github.com/bitcoinresearchkit/brk/tree/main/packages/bitview_client)

AI clients can use the same API through the official stateless, read-only MCP
endpoint at [mcp.bitview.space](https://mcp.bitview.space/). No authentication
is required.

## Installation

```bash
pip install bitview-client
# or
uv add bitview-client
```

You can also copy [`bitview_client/__init__.py`](./bitview_client/__init__.py)
into a project. The client has no runtime dependencies; Pandas and Polars are
only needed for their optional conversion helpers.

## Quick start

```python
from bitview_client import BitviewClient

# Use the public API or point the client at a self-hosted Bitview server.
client = BitviewClient("https://bitview.space")

# Mempool.space-compatible blockchain endpoints.
block_hash = client.get_block_by_height(800000)
tx = client.get_tx(
    "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d"
)

# Typed, chainable series access.
prices = (
    client.series.price.split.close.usd.by.day1()
    .tail(30)
    .fetch()
)

# Programmatic access when the series name is only known at runtime.
same_prices = client.series_endpoint("price_close", "day1").tail(30).fetch()
```

Series endpoints support integer and date slices, `head(n)`, `tail(n)`,
`skip(n).take(m)`, `fetch_csv()`, `len()`, and `version()`.
