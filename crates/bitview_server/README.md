# bitview_server

HTTP API server for Bitcoin on-chain analytics.

## Features

- **OpenAPI spec**: Auto-generated docs at `/api` with full spec at `/openapi.json`
- **LLM-optimized**: Compact spec at `/api.json` for AI tools
- **MCP-ready**: The same OpenAPI operations are available through the official
  stateless, read-only endpoint at [mcp.bitview.space](https://mcp.bitview.space/)
- **HTTP caching**: ETag revalidation with separate browser and CDN policies
- **Compression**: Brotli, gzip, deflate, zstd
- **Static files**: Optional web interface hosting

Plugin features mirror `bitview_query` and gate the routes that can use them.
`chain`, `series`, `urpd`, and `full-api` are convenience aggregators; the
default is `full-api`. Custom compositions can disable default features and
enable only the plugins and route families they provide. This crate does not
depend on the official Bitview composition.

## Usage

```rust,ignore
let server = Server::bind(
    &async_query,
    ServerConfig {
        data_path,
        website: Website::Filesystem(files_path),
        ..Default::default()
    },
)
.await?;
server.serve().await?;
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

ETag-based revalidation. Six strategies pick the etag scheme:

- **Tip**: chain-state, etag = tip hash prefix (invalidates per block + reorgs)
- **Immutable**: deeply-confirmed data, etag = format version
- **BlockBound**: immutable content tied to a specific block hash
- **ActivityBound**: mutable state anchored to its latest relevant block
- **Deploy**: catalog/static data, etag = build version
- **MempoolHash**: mempool data, etag = the relevant mempool-state hash

Series responses use a separate range-aware scheme: immutable historical
ranges are keyed by schema version and bounds, while mutable tails are keyed by
the current tip hash.

Browser sees `Cache-Control: public, no-cache, stale-if-error=86400` (always
revalidate, ETag makes it cheap). CDN sees a separate `CDN-Cache-Control`
directive whose stable tier is selected by `CdnCacheMode` (`Live` revalidates
every request; `Aggressive` caches up to a year as `immutable` and requires a
purge on deploy).

Errors deliberately have no ETag: a conditional request must receive the error
status again rather than `304`. Unknown-resource and other recoverable client
errors use a one-second, must-revalidate policy; permanently invalid address,
network, and transaction-ID inputs are immutable; authorization,
service-unavailable, and server errors use `no-store`.

## Configuration

Binds exactly to `0.0.0.0:3110` by default. Set `ServerConfig::bind` and
`ServerConfig::port` to use another listener.

## Dependencies

- `bitview_query` - data access
- `aide` + `axum` - HTTP routing and OpenAPI
- `tower-http` - compression and tracing
