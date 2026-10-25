// https://docs.rs/schemars/latest/schemars/derive.JsonSchema.html

mod compact;

use aide::openapi::{Contact, Info, License, OpenApi, Tag};
use axum::body::Bytes;
use serde_json::{to_value, to_vec};

use crate::{VERSION, prepared_json::PreparedJson};

/// The full spec and its compact form for LLMs, pre-serialized at startup with their validators.
pub struct OpenApiBodies {
    pub full: PreparedJson,
    pub compact: PreparedJson,
}

impl OpenApiBodies {
    pub fn new(openapi: &OpenApi) -> Self {
        let json = |bytes: Vec<u8>| PreparedJson::from_bytes(Bytes::from(bytes));
        Self {
            full: json(to_vec(openapi).unwrap()),
            compact: json(compact::compact_json(to_value(openapi).unwrap())),
        }
    }
}

pub mod tags {
    pub const SERVER: &str = "Server";
    pub const SERIES: &str = "Series";
    pub const GENERAL: &str = "General";
    pub const ADDRESSES: &str = "Addresses";
    pub const BLOCKS: &str = "Blocks";
    pub const MINING: &str = "Mining";
    pub const FEES: &str = "Fees";
    pub const MEMPOOL: &str = "Mempool";
    pub const TRANSACTIONS: &str = "Transactions";
    pub const ORACLE: &str = "Oracle";
    pub const UTXO_SET: &str = "UTXO Set";
    pub const URPD: &str = "URPD";
}

/// The tags in the spec's order, each with its description.
const TAGS: [(&str, &str); 12] = [
    (
        tags::SERVER,
        "API metadata, health monitoring, and OpenAPI specifications.",
    ),
    (
        tags::SERIES,
        "Thousands of historical Bitcoin series across calendar, block and entity indexes, as JSON \
        or CSV. Series are renamed only in releases, each rename listed in the changelog.",
    ),
    (
        tags::GENERAL,
        "Difficulty adjustments and BTC/USD price data.\n\n\
        *[Mempool.space](https://mempool.space/docs/api/rest) compatible.*",
    ),
    (
        tags::ADDRESSES,
        "Address balances, transaction history and UTXOs. Supports P2PKH, P2SH, P2WPKH, P2WSH, \
        P2TR and P2A addresses, and P2PK public keys in hex (33 or 65 bytes).\n\n\
        *[Mempool.space](https://mempool.space/docs/api/rest) compatible.*",
    ),
    (
        tags::BLOCKS,
        "Blocks by hash or height: headers, transaction lists and raw block bytes.\n\n\
        *[Mempool.space](https://mempool.space/docs/api/rest) compatible.*",
    ),
    (
        tags::MINING,
        "Mining statistics including pool distribution, hashrate, difficulty adjustments, \
        block rewards, and fee rates across configurable time periods.\n\n\
        *[Mempool.space](https://mempool.space/docs/api/rest) compatible.*",
    ),
    (
        tags::FEES,
        "Fee estimation and projected mempool blocks.\n\n\
        *[Mempool.space](https://mempool.space/docs/api/rest) compatible.*",
    ),
    (
        tags::MEMPOOL,
        "Mempool statistics, transaction IDs, fee histogram and recent transactions.\n\n\
        *[Mempool.space](https://mempool.space/docs/api/rest) compatible.*",
    ),
    (
        tags::TRANSACTIONS,
        "Transactions by txid: details, confirmation status, raw hex, merkle proofs and output \
        spends.\n\n\
        *[Mempool.space](https://mempool.space/docs/api/rest) compatible.*",
    ),
    (
        tags::ORACLE,
        "On-chain BTC/USD price derived purely from round-dollar payment patterns in \
        transaction outputs, with no external price feed. Payment activity is binned on a \
        log scale, and a smoothed EMA over recent blocks locates the price.\n\n\
        The live price is at `/api/oracle/price`, confirmed per-height price history at \
        `/api/series/price/height`. How it read each block (what it counted, where it looked, what \
        it found) is at `/api/oracle/blocks/{height}`, and now at `/api/oracle/live`; the payments \
        behind its prices, 144 blocks at a time, at `/api/oracle/payments/{start}`.",
    ),
    (
        tags::UTXO_SET,
        "The UTXO set by creation height: what remains unspent of each block's outputs \
        after a block or a UTC day, and what a block or a day changed. Reconstructed from \
        the UTXO set's history on demand. Supplies are in BTC; a set plus the next diff \
        gives the next set, exact when each supply is rounded to 8 decimals. A date \
        covers the day's published blocks, by each block's monotonic timestamp (the \
        running maximum of block times), like URPD.\n\n\
        The set excludes `OP_RETURN` outputs and the genesis coinbase (unspendable), and \
        the coinbases of blocks 91812 and 91722 once duplicates at 91842 and 91880 \
        overwrite them; it includes zero-value outputs.",
    ),
    (
        tags::URPD,
        "UTXO Realized Price Distribution. For each (cohort, block) pair, supply is \
        grouped by the price at which each UTXO was last moved. Distributions are \
        reconstructed from the UTXO set's history on demand.\n\n\
        The buckets are columns, one entry per bucket in ascending `price`: `supply` (BTC), \
        `realized_cap` (USD, the sum of each coin's creation price times its supply, so not \
        `price * supply` once aggregated), and `unrealized_pnl` (USD, = `close * supply - \
        realized_cap`, can be negative).\n\n\
        Aggregate with the `agg` query parameter:\n\
        - `raw`: one bucket per rounded price (default).\n\
        - `lin200` / `lin500` / `lin1000`: linear buckets, $200 / $500 / $1000 wide.\n\
        - `log10` / `log50` / `log100` / `log200` / `log500` / `log1000` / `log2000`: logarithmic \
        buckets, N bins per price decade.\n\n\
        Weight supply with the `weight` query parameter:\n\
        - `none`: unweighted supply (default).\n\
        - `wakefulness`: each age band weighted by its wakefulness (Cointime), the distribution behind \
        the `*_awake_*` series.\n\
        - `mobility`: each age band weighted by its mobility (Coinflow), behind the `*_mobile_*` series.\n\n\
        Unweighted and weighted age-range, `all`, `sth`, and `lth` distributions share \
        the same block state. Weights are evaluated at the requested block.\n\n\
        Discovery flow: `GET /api/urpd` (cohorts), `GET /api/urpd/{cohort}` (latest), \
        `GET /api/urpd/{cohort}/{point}` (any block height or UTC day with a block).",
    ),
];

pub fn create_openapi() -> OpenApi {
    let info = Info {
        title: "Bitview".to_string(),
        description: Some(
            r#"API for querying Bitcoin blockchain data, mempool state, and on-chain series.

### Features

- **[Mempool.space](https://mempool.space/docs/api/rest) compatible**: Blocks, transactions, addresses, mining, fees, and mempool endpoints match the mempool.space REST API
- **Series**: Thousands of on-chain time-series across multiple indexes (date, block height, etc.)
- **Multiple formats**: JSON and CSV output
- **For LLMs**: [`/llms.txt`](/llms.txt) for discovery, [`/api.json`](/api.json) a compact custom summary of this spec for reading (not OpenAPI); build tools from [`/openapi.json`](/openapi.json)
- **MCP**: Stateless, read-only access to these operations at [mcp.bitview.space](https://mcp.bitview.space/), with no authentication required

### Quick start

```bash
curl -s https://bitview.space/api/blocks/tip/height
curl -s https://bitview.space/api/v1/fees/recommended
curl -s https://bitview.space/api/mempool
curl -s https://bitview.space/api/series/search?q=price
```

### Errors

Errors use `application/problem+json`:

```json
{
  "error": {
    "type": "not_found",
    "code": "series_not_found",
    "message": "'foo' not found, did you mean 'bar'?",
    "doc_url": "/api"
  }
}
```

- **`type`**: Error category, following the status — `invalid_request` (4xx other than 404), `not_found` (404), `unavailable` (503), `timeout` (504), or `internal` (other 5xx); see the `ErrorType` schema
- **`code`**: Machine-readable error code (e.g. `invalid_address`, `series_not_found`, `weight_exceeded`); see the `ErrorCode` schema

### Client Libraries

- [JavaScript](https://www.npmjs.com/package/bitview-client)
- [Python](https://pypi.org/project/bitview-client/)
- [Rust](https://crates.io/crates/bitview_client)
- [MCP](https://mcp.bitview.space/)

### Links

- [GitHub](https://github.com/bitcoinresearchkit/mono)
- [Bitview Studio](https://bitview.space/studio/) - Charts of any series, built on this API"#
                .to_string(),
        ),
        version: format!("v{VERSION}"),
        contact: Some(Contact {
            name: Some("Bitview".to_string()),
            url: Some("https://github.com/bitcoinresearchkit/mono".to_string()),
            email: Some("support@bitview.space".to_string()),
            ..Contact::default()
        }),
        license: Some(License {
            name: "MIT".to_string(),
            url: Some(
                "https://github.com/bitcoinresearchkit/mono/blob/main/LICENSE".to_string(),
            ),
            ..License::default()
        }),
        ..Info::default()
    };

    let tags = TAGS
        .iter()
        .map(|&(name, description)| Tag {
            name: name.to_owned(),
            description: Some(description.to_owned()),
            ..Default::default()
        })
        .collect();

    OpenApi {
        info,
        tags,
        ..OpenApi::default()
    }
}
