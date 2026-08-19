# bitview-client

Python client for the [Bitcoin Research Kit](https://github.com/bitcoinresearchkit/brk) API.

Requires Python 3.9+. Zero dependencies.

[PyPI](https://pypi.org/project/bitview-client/) | [API Reference](https://github.com/bitcoinresearchkit/brk/blob/main/packages/bitview_client/DOCS.md)

AI clients can use the same API through the official stateless, read-only MCP
endpoint at [mcp.bitview.space](https://mcp.bitview.space/). No authentication
is required.

## Installation

```bash
pip install bitview-client
# or
uv add bitview-client
```

Or just copy [`bitview_client/__init__.py`](./bitview_client/__init__.py) into your project - it's a single file with no dependencies.

## Quick Start

```python
from bitview_client import BitviewClient

# Use the free public API or your own instance
# Has optional `, timeout=60.0` argument
client = BitviewClient("https://bitview.space")

# Blockchain data (mempool.space compatible)
block = client.get_block_by_height(800000)
tx = client.get_tx("abc123...")
address = client.get_address("bc1q...")

# Metrics API - typed, chainable
prices = client.metrics.price.usd.split.close \
    .by.dateindex() \
    .tail(30) \
    .fetch()  # Last 30 items

# Generic metric fetching
data = client.get_metric("price_close", "dateindex", -30)
```
