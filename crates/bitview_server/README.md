# bitview_server

HTTP API server for Bitcoin on-chain analytics.

Responses are sent uncompressed, for a CDN in front to compress. The server also serves the website, always: embedded, or in debug builds read live from the repository.

`series` serves the generic series routes and `full-api`, the default, adds every other route.
Custom compositions can disable default features and enable `series` alone, which needs only the
mappings plugin; `full-api` needs every plugin its endpoints read. This crate does not
depend on the official Bitview composition.

## Usage

```rust,ignore
let server = Server::bind(
    &async_query,
    ServerConfig {
        data_path,
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
| `/mcp` | MCP server (with `full-api`) |

## MCP

`/mcp` is a stateless, read-only MCP server over Streamable HTTP, for MCP
revisions `2025-06-18` through `2026-07-28`. Its tools are the MCP-visible API
operations, from the catalog `cargo bindgen` writes to `mcp/manifest.json`
(embedded at compile time, with the MCP Registry's `mcp/server.json` beside it;
see [`bitview_bindgen`](../bitview_bindgen/README.md#regenerating)). A tool call
is the API request it names, sent to this server's own router without a socket:
the same handler, admission and answer as over HTTP, logged as that one request.

A `GET` of `/mcp` is the server's page; `/mcp/` redirects to `/mcp`. Beside it
are `/mcp/logo.png`, `/mcp/server-card`, `/mcp/.well-known/ai-catalog.json`, and
`/mcp/privacy`, `/mcp/terms` and `/mcp/support`, which redirect to the site's
pages. The server names itself by `ServerConfig::site_url` (bitviewd's
`--siteurl`, `http://localhost:<port>` by default).

## Caching

Every read is answered one way (`AppState::respond`). Its route resolves an
answer: the tag naming what it would send, how long its bytes stay what they
are, and how to build it. The `304`, the body budget and the cache class follow
from those; no route sets a cache header.

**The tag.** Every API ETag is weak and starts with a build hash. Bitviewd hashes
its source inputs and dependency lockfile at build time, so independent builds
of the same inputs can share validators. Applications embedding the server can
supply their own build tag (`set_build`) and otherwise use their executable's
hash. The rest is made of what the body is read from, under the same pin the
body is read under: the block that fixes it (a block by hash, a confirmed
transaction, a point of the UTXO set), the tip (whole-history statistics,
tip-keyed series), the block that last changed it (an address), or a hash of the
bytes where nothing cheaper names them or building costs no more than naming. A request holding the tag gets `304`
before any of the body is built, except for a content hash, which is the body's.
A tag is resolved against what is published now; a matching one is not
permission to serve an unpublished or displaced result.

**The class.** It follows from one thing the route declares: how long the bytes
at the URL stay what they are.

| the answer | browser (`Cache-Control`) | CDN (`CDN-Cache-Control`) |
|---|---|---|
| live: changes at any time | 1 s, then revalidate | 1 s, then revalidate |
| settled: the server shapes it (JSON, CSV, the hash at a height) from chain data the URL pins wholly below the six-block reorg window | 1 s, then revalidate | immutable under `--cdn`, else 1 s |
| committed: the hash in the URL commits to its every byte, in a block below the reorg window | immutable | immutable |

One second is `public, max-age=1, must-revalidate`, which lets readers polling
together share one answer; immutable is `public, max-age=31536000, immutable`.
Neither allows stale-on-error reuse.

Browsers are never purged, so only what no release changes is immutable for
them: a raw block, a block's header, the txid at an index of a block, and a
settled transaction's hex and raw bytes (a txid commits to its witness only
through its block). Its proofs are settled: the txid in their address doesn't
say which block holds the transaction, and the JSON one is the server's to
shape. A settled answer the server shapes can change
shape with a code change. A browser revalidates it against the build-tagged
validator and gets `304` while the tag matches. The CDN
of a server run with `--cdn` (`CdnCacheMode::Aggressive`) is purged by its
operator on every release and on a reorg deeper than six blocks, so it keeps
every settled answer until then; `CdnCacheMode::Live` gives it the one-second
policy instead. No address carries a version: the paths stay mempool.space's
and Esplora's.

Settled once below the reorg window: blocks by hash or height, by timestamp, and
fixed block windows; confirmed transactions (bodies, proofs, CPFP, status, by
index) and their outspends once every output is spent; an address's history
page after a cursor; a pool's blocks before a height; height or date points
(UTXO set and its diff, URPD, oracle histograms and payments), settled by their
block, for a date by the next day's first block; a timestamp's historical price
by the first block of the timestamp's four-hour bucket, which closes every
earlier one. Latest, tip and mempool answers never are. Documents only a
release changes (the series catalog, its count and indexes, the API
specifications, pool metadata) are settled from startup.

Series ranges are settled when the URL pins them (no bound counted from the end,
left open, past the rows or decided by a block inside the reorg window, as a
timestamp bound can be) wholly below the reorg window. Their tag is then the
schema version, bounds and the hash of the last block they depend on (stable
across appends, changed by any reorg replacing them): the last block they cover,
or for calendar indexes the first block of the following period, whose timestamp
ends the range's last period. Other ranges, and every range of a series that
rewrites existing rows, are named by the current tip hash.

Errors deliberately have no ETag: a conditional request must receive the error
status again rather than `304`. Client errors (`400`, `404`), invalid addresses
included, use a one-second, must-revalidate policy, since a release can change
even a permanent verdict and browsers are never purged; service-unavailable and
server errors use `no-store`. The 404 for an address no route or website file names is
`no-store` too: the website answers a browser with its not-found page there, so the answer depends on `Accept`,
which a CDN's cache key leaves out; the error layer keeps a cache policy its handler marked (`CachePolicy::mark`). Server errors carry a generic message; the detail goes to the log.

`POST /api/tx` is an action: every response is `no-store`, with no validator.
It accepts at most 800,000 bytes of hexadecimal text including surrounding
whitespace (the largest standard transaction; the node takes no larger one), and admits at most four buffered/pending submissions, each sent on
its own connection to the node; excess admission returns 503. Submission is not automatically replayed after a lost
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

Binds exactly to `127.0.0.1:3110` by default: this machine only, since the server has no authentication. Set
`ServerConfig::bind` (`0.0.0.0` to serve the network) and `ServerConfig::port` to use another listener.
