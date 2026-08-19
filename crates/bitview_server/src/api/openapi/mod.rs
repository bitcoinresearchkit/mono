//
// https://docs.rs/schemars/latest/schemars/derive.JsonSchema.html
//
// Scalar:
// - Documentation: https://guides.scalar.com/scalar/scalar-api-references
// - Configuration: https://guides.scalar.com/scalar/scalar-api-references/configuration
// - Examples:
//   - https://docs.machines.dev/
//   - https://tailscale.com/api
//   - https://api.supabase.com/api/v1
//

mod compact;
mod full;

pub use compact::ApiJson;
pub use full::OpenApiJson;

use aide::openapi::{Contact, Info, License, OpenApi, Tag};

use crate::VERSION;

pub fn create_openapi() -> OpenApi {
    let info = Info {
        title: "Bitcoin Research Kit".to_string(),
        description: Some(
            r#"API for querying Bitcoin blockchain data, mempool state, and on-chain series.

### Features

- **[Mempool.space](https://mempool.space/docs/api/rest) compatible**: Blocks, transactions, addresses, mining, fees, and mempool endpoints match the mempool.space REST API
- **Series**: Thousands of on-chain time-series across multiple indexes (date, block height, etc.)
- **Multiple formats**: JSON and CSV output
- **LLM-optimized**: [`/llms.txt`](/llms.txt) for discovery, [`/api.json`](/api.json) compact OpenAPI spec for tool use (full spec at [`/openapi.json`](/openapi.json))
- **MCP**: Stateless, read-only access to these operations at [mcp.bitview.space](https://mcp.bitview.space/), with no authentication required

### Quick start

```bash
curl -s https://bitview.space/api/blocks/tip/height
curl -s https://bitview.space/api/v1/fees/recommended
curl -s https://bitview.space/api/mempool
curl -s https://bitview.space/api/series/search?q=price
```

### Errors

All errors return structured JSON with a consistent format:

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

- **`type`**: Error category — `invalid_request` (400), `forbidden` (403), `not_found` (404), `unavailable` (503), or `internal` (500)
- **`code`**: Machine-readable error code (e.g. `invalid_address`, `series_not_found`, `weight_exceeded`)
- **`message`**: Human-readable description
- **`doc_url`**: Link to API documentation

### Client Libraries

- [JavaScript](https://www.npmjs.com/package/bitview-client)
- [Python](https://pypi.org/project/bitview-client/)
- [Rust](https://crates.io/crates/bitview_client)
- [MCP](https://mcp.bitview.space/)

### Links

- [GitHub](https://github.com/bitcoinresearchkit/brk)
- [Bitview](https://bitview.space) - Web app built on this API"#
                .to_string(),
        ),
        version: format!("v{VERSION}"),
        contact: Some(Contact {
            name: Some("Bitcoin Research Kit".to_string()),
            url: Some("https://github.com/bitcoinresearchkit/brk".to_string()),
            email: Some("hello@bitcoinresearchkit.org".to_string()),
            ..Contact::default()
        }),
        license: Some(License {
            name: "MIT".to_string(),
            url: Some(
                "https://github.com/bitcoinresearchkit/brk/blob/main/docs/LICENSE.md".to_string(),
            ),
            ..License::default()
        }),
        ..Info::default()
    };

    let tags = vec![
        Tag {
            name: "Server".to_string(),
            description: Some(
                "API metadata, health monitoring, and OpenAPI specifications.".to_string(),
            ),
            ..Default::default()
        },
        Tag {
            name: "Series".to_string(),
            description: Some(
                "Access thousands of Bitcoin network time-series data. Query historical statistics \
                across various indexes (date, week, month, block height) with JSON or CSV output.\n\n\
                **Note:** Series names are subject to change while the project is in active development."
                    .to_string(),
            ),
            ..Default::default()
        },
        Tag {
            name: "General".to_string(),
            description: Some(
                "General Bitcoin network information including difficulty adjustments and price data.\n\n\
                *[Mempool.space](https://mempool.space/docs/api/rest) compatible.*"
                    .to_string(),
            ),
            ..Default::default()
        },
        Tag {
            name: "Addresses".to_string(),
            description: Some(
                "Query Bitcoin address data including balances, transaction history, and UTXOs. \
                Supports all address types: P2PKH, P2SH, P2WPKH, P2WSH, and P2TR.\n\n\
                *[Mempool.space](https://mempool.space/docs/api/rest) compatible.*"
                    .to_string(),
            ),
            ..Default::default()
        },
        Tag {
            name: "Blocks".to_string(),
            description: Some(
                "Retrieve block data by hash or height. Access block headers, transaction lists, \
                and raw block bytes.\n\n\
                *[Mempool.space](https://mempool.space/docs/api/rest) compatible.*"
                    .to_string(),
            ),
            ..Default::default()
        },
        Tag {
            name: "Mining".to_string(),
            description: Some(
                "Mining statistics including pool distribution, hashrate, difficulty adjustments, \
                block rewards, and fee rates across configurable time periods.\n\n\
                *[Mempool.space](https://mempool.space/docs/api/rest) compatible.*"
                    .to_string(),
            ),
            ..Default::default()
        },
        Tag {
            name: "Fees".to_string(),
            description: Some(
                "Fee estimation and projected mempool blocks.\n\n\
                *[Mempool.space](https://mempool.space/docs/api/rest) compatible.*"
                    .to_string(),
            ),
            ..Default::default()
        },
        Tag {
            name: "Mempool".to_string(),
            description: Some(
                "Monitor unconfirmed transactions. Get mempool statistics, \
                transaction IDs, fee histogram, and recent transactions.\n\n\
                *[Mempool.space](https://mempool.space/docs/api/rest) compatible.*"
                    .to_string(),
            ),
            ..Default::default()
        },
        Tag {
            name: "Transactions".to_string(),
            description: Some(
                "Retrieve transaction data by txid. Access full transaction details, confirmation \
                status, raw hex, merkle proofs, and output spend information.\n\n\
                *[Mempool.space](https://mempool.space/docs/api/rest) compatible.*"
                    .to_string(),
            ),
            ..Default::default()
        },
        Tag {
            name: "Oracle".to_string(),
            description: Some(
                "On-chain BTC/USD price derived purely from round-dollar payment patterns in \
                transaction outputs, with no external price feed. Payment activity is binned on a \
                log scale, and a smoothed EMA over recent blocks locates the price.\n\n\
                Histograms come in two flavors, each available at the live tip (mempool-blended) \
                or at any confirmed height: `raw` bins every output by value with no filtering, \
                while `ema` is the smoothed round-dollar window the price is read from. The live \
                price is also at `/api/mempool/price`. Confirmed per-height price history is at \
                `/api/series/price/height`."
                    .to_string(),
            ),
            ..Default::default()
        },
        Tag {
            name: "URPD".to_string(),
            description: Some(
                "UTXO Realized Price Distribution. For each (cohort, date) pair, supply is \
                grouped by the close price at which each UTXO was last moved. One snapshot is \
                emitted per UTC day.\n\n\
                Each bucket carries `supply` (BTC), `realized_cap` (USD, = `price_floor * supply`), \
                and `unrealized_pnl` (USD, = `(close - price_floor) * supply`, can be negative).\n\n\
                Aggregate with the `agg` query parameter (alias `bucket`):\n\
                - `raw`: one bucket per rounded price (default).\n\
                - `lin200` / `lin500` / `lin1000`: linear buckets, $200 / $500 / $1000 wide.\n\
                - `log10` / `log50` / `log100` / `log200`: logarithmic buckets, N bins per price decade.\n\n\
                Weight supply with the `weight` query parameter:\n\
                - `raw`: unweighted supply (default).\n\
                - `cointime`: cointime-weighted supply.\n\
                - `coinflow`: coinflow-weighted supply.\n\n\
                Weighted `all`, `sth`, and `lth` snapshots are persisted. Weighted age-range \
                cohorts are derived from their raw snapshot and daily cohort weight.\n\n\
                Discovery flow: `GET /api/urpd` (cohorts), `GET /api/urpd/{cohort}` (latest), \
                `GET /api/urpd/{cohort}/dates` (history), `GET /api/urpd/{cohort}/{date}` (specific)."
                    .to_string(),
            ),
            ..Default::default()
        },
        Tag {
            name: "Metrics".to_string(),
            description: Some("Deprecated - use Series".to_string()),
            extensions: [("deprecated".to_string(), serde_json::Value::Bool(true))].into(),
            ..Default::default()
        },
    ];

    OpenApi {
        info,
        tags,
        ..OpenApi::default()
    }
}
