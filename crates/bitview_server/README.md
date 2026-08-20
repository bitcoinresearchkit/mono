# bitview_server

HTTP API server for Bitcoin on-chain analytics.

## Features

- **OpenAPI spec**: Auto-generated docs at `/api` with full spec at `/openapi.json`
- **LLM-optimized**: Compact spec at `/api.json` for AI tools
- **MCP-ready**: The same OpenAPI operations are available through the official
  stateless, read-only endpoint at [mcp.bitview.space](https://mcp.bitview.space/)
- **Response caching**: ETag-based with LRU cache (1000 entries by default, configurable via `ServerConfig::cache_size`)
- **Compression**: Brotli, gzip, deflate, zstd
- **Static files**: Optional web interface hosting

Plugin features mirror `bitview_query` and gate the routes that can use them.
`chain`, `series`, `urpd`, and `full` are convenience aggregators; the default
is `full`. Custom compositions can disable default features and enable only the
plugins and route families they provide. This crate does not depend on the
official Bitview composition.

## Usage

```rust,ignore
let server = Server::new(
    &async_query,
    ServerConfig {
        data_path,
        website: Website::Filesystem(files_path),
        ..Default::default()
    },
);
server.serve(None).await?;
```

## Endpoints

| Path | Description |
|------|-------------|
| `/api` | Interactive API documentation |
| `/openapi.json` | Full OpenAPI specification |
| `/api.json` | Compact OpenAPI for LLMs |
| `/api/address/{address}` | Address stats, transactions, UTXOs |
| `/api/block/{hash}` | Block info, transactions, status |
| `/api/block-height/{height}` | Block by height |
| `/api/tx/{txid}` | Transaction details, status, hex |
| `/api/mempool` | Fee estimates, mempool stats |
| `/api/series` | Hierarchical series catalog |
| `/api/series/{series}/{index}` | Series data and range queries |
| `/api/v1/mining/...` | Hashrate, difficulty, pools |

## Caching

ETag-based revalidation. Five strategies pick the etag scheme:

- **Tip**: chain-state, etag = tip hash prefix (invalidates per block + reorgs)
- **Immutable**: deeply-confirmed data, etag = format version
- **BlockBound**: data tied to a specific block hash (reorg-safe)
- **Deploy**: catalog/static data, etag = build version
- **MempoolHash**: mempool data, etag = projected next-block hash

Browser sees `Cache-Control: public, no-cache, stale-if-error=86400` (always
revalidate, ETag makes it cheap). CDN sees a separate `CDN-Cache-Control`
directive whose stable tier is selected by `CdnCacheMode` (`Live` revalidates
every request; `Aggressive` caches up to a year as `immutable` and requires a
purge on deploy).

## Configuration

Binds to port 3110, auto-incrementing up to 3210 if busy.

## Dependencies

- `bitview_query` - data access
- `aide` + `axum` - HTTP routing and OpenAPI
- `tower-http` - compression and tracing
