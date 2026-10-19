# bitview_server

HTTP API server for Bitcoin on-chain analytics.

Brotli, gzip and zstd compression skips small responses; a request refusing every encoding
(identity included) gets 406 before any handler runs. The server can also host a website.

Route-family features (`chain`, `series`, `urpd`, and `price`, which `chain` and
`urpd` imply) gate the routes; `full-api` enables all of them and is the default.
Custom compositions can disable default features and enable only the route
families they provide; each family pulls the plugins its endpoints read. This crate does not
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
| `/api.json` | Compact API summary for LLMs (custom format, not OpenAPI) |
| `/api/address/{address}` | Address statistics |
| `/api/block/{hash}` | Block metadata |
| `/api/block-height/{height}` | Block hash as plain text |
| `/api/tx/{txid}` | Transaction details |
| `/api/mempool` | Mempool statistics and fee histogram |
| `/api/series` | Hierarchical series catalog |
| `/api/series/{series}/{index}` | Series data and range queries |
| `/api/v1/mining/...` | Hashrate, difficulty, pools |

## Caching

Validators are selected with their source data before conditional matching:

- **Live**: exact, versioned representation tag
- **LiveHash**: mutable data, etag = a representation-specific content hash
- **Immutable**: data fixed for the release (the empty search), etag = format version
- **ActivityBound**: mutable state anchored to its latest relevant block
- **Deploy**: the release prefix alone, for responses only a release changes
- **Revalidate**: exact tags a second's reuse cannot mislead (blocks by hash, the tip, catalogs)

Catalogs, pool metadata and specifications use content-based validators, not
the package version alone. Dynamic transaction, block and mempool reads validate
current availability before returning 304; a matching tag is not permission to
serve an unpublished or displaced result.

Series responses use a separate range-aware scheme: ranges the URL pins (no
bound counted from the end, left open or past the rows) wholly below the
six-block reorg window are keyed by schema version, bounds and the hash of the
last block they depend on (stable across appends, changed by any reorg replacing
them). That is the last block they cover, or for calendar indexes the first
block of the following period, whose timestamp ends the range's last period.
Other ranges, and every range of a series that rewrites existing rows, are keyed
by the current tip hash.

Every ETag starts with the release, so a release changing a response's format
never validates an older body. Live responses (exact tags, content hashes,
activity anchors, tip-keyed series) use
`Cache-Control: public, no-cache, must-revalidate` for browsers. Responses a
second's reuse cannot mislead (blocks by hash, the tip, catalogs, specifications
and other deployment-bound responses) allow one second of browser and edge
freshness (`max-age=1, must-revalidate`). The separate
`CDN-Cache-Control` live tier allows one second of edge freshness, then requires
revalidation; neither policy allows stale-on-error reuse.
`CdnCacheMode::Live` applies that same edge policy to stable responses.
`Aggressive` instead caches the stable tier for up to a year as `immutable` and
requires an operator purge on deploy. Mutable and deployment-bound URLs retain
their short freshness under either mode.

Errors deliberately have no ETag: a conditional request must receive the error
status again rather than `304`. Unknown-resource and other recoverable client
errors use a one-second, must-revalidate policy; permanently invalid address and
network inputs are immutable; service-unavailable and server errors use
`no-store`. Server errors carry a generic message; the detail goes to the log.

`POST /api/tx` is an action: every response is `no-store`, with no validator.
It accepts at most 8,000,000 bytes of hexadecimal text including surrounding
whitespace, and admits at most four buffered/pending submissions. Requests wait
asynchronously for the shared node connection within the HTTP deadline; excess
admission returns 503. Submission is not automatically replayed after a lost
response. A timeout or cancellation after dispatch can leave the outcome unknown
and cannot undo a transaction already received by the node.

## Read availability

See [docs/read-availability.md](docs/read-availability.md) for deadlines, admission,
publication pins, response capacity and transient errors.

Every `bitview_query::Error` maps to a status in one exhaustive match (`error.rs`), so a new variant must
choose one. Failures from below the query (storage, RPC, IO) are internal errors (500, `no-store`), never a
404 or 400, except the conditions `bitview_query` translates (invalid address or network input, an
unindexable date, an updating state) and a transaction the node rejects (400).

## Configuration

Binds exactly to `0.0.0.0:3110` by default. Set `ServerConfig::bind` and
`ServerConfig::port` to use another listener.
