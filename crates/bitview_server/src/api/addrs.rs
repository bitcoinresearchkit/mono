use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, Uri},
};
use brk_types::{AddrHashPrefixMatches, AddrStats, AddrValidation, Transaction, Utxo, Version};

use crate::{
    AppState, CacheStrategy,
    extended::TransformResponseExtended,
    params::{AddrAfterTxidParam, AddrHashPrefixParam, AddrParam, Empty, ValidateAddrParam},
};

/// Esplora `/txs` and `/txs/chain` page sizes. Wire-protocol constants from
/// mempool.space/esplora, not deployment policy. `/txs` returns up to
/// `MEMPOOL_PAGE` mempool entries plus a chain page sized to reach
/// `TXS_TOTAL_TARGET` total, floored at `CHAIN_PAGE`.
const MEMPOOL_PAGE: usize = 50;
const CHAIN_PAGE: usize = 25;
const TXS_TOTAL_TARGET: usize = 50;

pub trait AddrRoutes {
    fn add_addr_routes(self) -> Self;
}

impl AddrRoutes for ApiRouter<AppState> {
    fn add_addr_routes(self) -> Self {
        self.api_route(
            "/api/address/hash-prefix/{addr_type}/{prefix}",
            get_with(async |
                uri: Uri,
                headers: HeaderMap,
                Path(path): Path<AddrHashPrefixParam>,
                _: Empty,
                State(state): State<AppState>
            | {
                state.respond_json(&headers, CacheStrategy::Tip, &uri, move |q| {
                    q.addr_hash_prefix_matches(path.addr_type, &path.prefix)
                }).await
            }, |op| op
                .id("get_address_hash_prefix_matches")
                .addrs_tag()
                .summary("Address hash-prefix matches")
                .description("Find addresses by address type and by the first 1-16 hex nibbles of RapidHash v3 over the raw address payload bytes. Intended for privacy-preserving client-side wallet discovery without sending raw addresses or xpubs. Fetch metadata with `GET /api/address/{address}`.")
                .json_response::<AddrHashPrefixMatches>()
                .not_modified()
                .bad_request()
                .server_error()
            ),
        )
        .api_route(
            "/api/address/{address}",
            get_with(async |
                uri: Uri,
                headers: HeaderMap,
                Path(path): Path<AddrParam>,
                _: Empty,
                State(state): State<AppState>
            | {
                let strategy = state.addr_strategy(Version::ONE, &path.addr, false, None);
                state.respond_json(&headers, strategy, &uri, move |q| q.addr(path.addr)).await
            }, |op| op
                .id("get_address")
                .addrs_tag()
                .summary("Address information")
                .description("Retrieve address information including current balance and transaction counts. Supports all standard Bitcoin address types (P2PKH, P2SH, P2WPKH, P2WSH, P2TR).\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address)*")
                .json_response::<AddrStats>()
                .not_modified()
                .bad_request()
                .not_found()
                .server_error()
            ),
        )
        .api_route(
            "/api/address/{address}/txs",
            get_with(async |
                uri: Uri,
                headers: HeaderMap,
                Path(path): Path<AddrParam>,
                _: Empty,
                State(state): State<AppState>
            | {
                let strategy = state.addr_strategy(Version::ONE, &path.addr, false, None);
                state.respond_json(&headers, strategy, &uri, move |q| {
                    let mempool_txs = if q.mempool().is_some() {
                        q.addr_mempool_txs(&path.addr, MEMPOOL_PAGE)?
                    } else {
                        Vec::new()
                    };
                    let chain_limit = TXS_TOTAL_TARGET.saturating_sub(mempool_txs.len()).max(CHAIN_PAGE);
                    let chain_txs = q.addr_txs_chain(&path.addr, None, chain_limit)?;
                    let mut out = mempool_txs;
                    out.extend(chain_txs);
                    Ok(out)
                }).await
            }, |op| op
                .id("get_address_txs")
                .addrs_tag()
                .summary("Address transactions")
                .description("Get transaction history for an address, newest first. Returns up to 50 mempool transactions plus a confirmed page sized to fill the response to 50 total (chain floor of 25, so 25-50 confirmed depending on mempool weight). To paginate further confirmed history, request `GET /api/address/{address}/txs/chain/{after_txid}` with the last returned txid.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions)*")
                .json_response::<Vec<Transaction>>()
                .not_modified()
                .bad_request()
                .not_found()
                .server_error()
            ),
        )
        .api_route(
            "/api/address/{address}/txs/chain",
            get_with(async |
                uri: Uri,
                headers: HeaderMap,
                Path(path): Path<AddrParam>,
                _: Empty,
                State(state): State<AppState>
            | {
                let strategy = state.addr_strategy(Version::ONE, &path.addr, true, None);
                state.respond_json(&headers, strategy, &uri, move |q| q.addr_txs_chain(&path.addr, None, CHAIN_PAGE)).await
            }, |op| op
                .id("get_address_confirmed_txs")
                .addrs_tag()
                .summary("Address confirmed transactions")
                .description("Get the first 25 confirmed transactions for an address. For pagination, request `GET /api/address/{address}/txs/chain/{after_txid}` with the last returned txid.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*")
                .json_response::<Vec<Transaction>>()
                .not_modified()
                .bad_request()
                .not_found()
                .server_error()
            ),
        )
        .api_route(
            "/api/address/{address}/txs/chain/{after_txid}",
            get_with(async |
                uri: Uri,
                headers: HeaderMap,
                Path(path): Path<AddrAfterTxidParam>,
                _: Empty,
                State(state): State<AppState>
            | {
                let strategy = state.addr_strategy(Version::ONE, &path.addr, true, Some(&path.after_txid));
                state.respond_json(&headers, strategy, &uri, move |q| q.addr_txs_chain(&path.addr, Some(path.after_txid), CHAIN_PAGE)).await
            }, |op| op
                .id("get_address_confirmed_txs_after")
                .addrs_tag()
                .summary("Address confirmed transactions (paginated)")
                .description("Get the next 25 confirmed transactions strictly older than `after_txid` (Esplora-canonical pagination form, matches mempool.space).\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*")
                .json_response::<Vec<Transaction>>()
                .not_modified()
                .bad_request()
                .not_found()
                .server_error()
            ),
        )
        .api_route(
            "/api/address/{address}/txs/mempool",
            get_with(async |
                uri: Uri,
                headers: HeaderMap,
                Path(path): Path<AddrParam>,
                _: Empty,
                State(state): State<AppState>
            | {
                let hash = state.sync(|q| q.addr_mempool_hash(&path.addr)).unwrap_or(0);
                state.respond_json(&headers, CacheStrategy::MempoolHash(hash), &uri, move |q| q.addr_mempool_txs(&path.addr, MEMPOOL_PAGE)).await
            }, |op| op
                .id("get_address_mempool_txs")
                .addrs_tag()
                .summary("Address mempool transactions")
                .description("Get unconfirmed transactions for an address from the mempool, newest first (up to 50).\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-mempool)*")
                .json_response::<Vec<Transaction>>()
                .not_modified()
                .bad_request()
                .not_found()
                .server_error()
            ),
        )
        .api_route(
            "/api/address/{address}/utxo",
            get_with(async |
                uri: Uri,
                headers: HeaderMap,
                Path(path): Path<AddrParam>,
                _: Empty,
                State(state): State<AppState>
            | {
                let strategy = state.addr_strategy(Version::ONE, &path.addr, false, None);
                let max_utxos = state.max_utxos;
                state.respond_json(&headers, strategy, &uri, move |q| q.addr_utxos(path.addr, max_utxos)).await
            }, |op| op
                .id("get_address_utxos")
                .addrs_tag()
                .summary("Address UTXOs")
                .description("Get unspent transaction outputs (UTXOs) for an address. Returns txid, vout, value, and confirmation status for each UTXO.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-utxo)*")
                .json_response::<Vec<Utxo>>()
                .not_modified()
                .bad_request()
                .not_found()
                .server_error()
            ),
        )
        .api_route(
            "/api/v1/validate-address/{address}",
            get_with(async |
                uri: Uri,
                headers: HeaderMap,
                Path(path): Path<ValidateAddrParam>,
                _: Empty,
                State(state): State<AppState>
            | {
                state.respond_json(&headers, CacheStrategy::Deploy, &uri, move |_q| Ok(AddrValidation::from_addr(&path.addr))).await
            }, |op| op
                .id("validate_address")
                .addrs_tag()
                .summary("Validate address")
                .description("Validate a Bitcoin address and get information about its type and scriptPubKey. Returns `isvalid: false` with an error message for invalid addresses.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-validate)*")
                .json_response::<AddrValidation>()
                .not_modified()
                .server_error()
            ),
        )
    }
}
