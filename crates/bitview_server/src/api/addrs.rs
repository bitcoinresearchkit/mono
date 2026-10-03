use aide::axum::{ApiRouter, routing::get_with};
use axum::{body::Bytes, extract::Path, http::HeaderMap, response::Response};
use bitview_query::RepresentationId;
use bitview_types::{AddrHashPrefixMatches, AddrStats, AddrValidation, Utxo};
use brk_types::{Addr, BlockHashPrefix, Transaction, Txid, Version};
use serde_json::to_vec;

use crate::{
    AppState, CacheParams, CacheStrategy, CdnCacheMode,
    error::Result,
    extended::{HeaderMapExtended, ResponseExtended, TransformResponseExtended},
    params::{AddrAfterTxidParam, AddrHashPrefixParam, AddrParam, Empty, ValidateAddrParam},
    request_state::RequestState,
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

impl AppState {
    async fn respond_addr_chain_txs(
        &self,
        headers: HeaderMap,
        addr: Addr,
        after_txid: Option<Txid>,
    ) -> Response {
        let mode = self.cdn_cache_mode;
        self.respond_read(
            headers,
            "application/json",
            move |q| {
                let source = q.resolve_addr_chain_txs(&addr, after_txid, CHAIN_PAGE)?;
                let strategy = CacheStrategy::ActivityBound(
                    Version::ONE,
                    BlockHashPrefix::from(&source.activity_anchor()),
                );
                Ok((source, CacheParams::resolve(&strategy, mode)))
            },
            |q, source| Ok(Bytes::from(to_vec(&q.addr_txs_chain_resolved(source)?)?)),
        )
        .await
    }
}

impl AddrRoutes for ApiRouter<AppState> {
    fn add_addr_routes(self) -> Self {
        self.api_route(
            "/api/address/hash-prefix/{addr_type}/{prefix}",
            get_with(async |
                headers: HeaderMap,
                Path(path): Path<AddrHashPrefixParam>,
                _: Empty,
                RequestState(state): RequestState
            | {
                state.respond_json_content(&headers, move |q| {
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
                headers: HeaderMap,
                Path(path): Path<AddrParam>,
                _: Empty,
                RequestState(state): RequestState
            | -> Result<Response> {
                Ok(state
                    .respond_json_content(&headers, move |q| q.addr(path.addr))
                    .await)
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
                headers: HeaderMap,
                Path(path): Path<AddrParam>,
                _: Empty,
                RequestState(state): RequestState
            | -> Result<Response> {
                serve_txs(state, headers, path.addr).await
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
                headers: HeaderMap,
                Path(path): Path<AddrParam>,
                _: Empty,
                RequestState(state): RequestState
            | -> Result<Response> {
                Ok(state.respond_addr_chain_txs(
                    headers,
                    path.addr,
                    None,
                ).await)
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
                headers: HeaderMap,
                Path(path): Path<AddrAfterTxidParam>,
                _: Empty,
                RequestState(state): RequestState
            | -> Result<Response> {
                Ok(state.respond_addr_chain_txs(
                    headers,
                    path.addr,
                    Some(path.after_txid),
                ).await)
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
                headers: HeaderMap,
                Path(path): Path<AddrParam>,
                _: Empty,
                RequestState(state): RequestState
            | -> Result<Response> {
                Ok(state.respond_json_content(&headers, move |q| {
                    q.addr_mempool_txs(&path.addr, MEMPOOL_PAGE)
                }).await)
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
                headers: HeaderMap,
                Path(path): Path<AddrParam>,
                _: Empty,
                RequestState(state): RequestState
            | -> Result<Response> {
                serve_utxos(state, headers, path.addr).await
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
                headers: HeaderMap,
                Path(path): Path<ValidateAddrParam>,
                _: Empty,
                RequestState(state): RequestState
            | {
                state.respond_json_immediate(&headers, CacheStrategy::Deploy, move || {
                    AddrValidation::from_addr(&path.addr)
                })
            }, |op| op
                .id("validate_address")
                .addrs_tag()
                .summary("Validate address")
                .description("Validate a Bitcoin address and get information about its type and scriptPubKey. Returns `isvalid: false` with an error message for invalid addresses.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-validate)*")
                .json_response::<AddrValidation>()
                .not_modified()
            ),
        )
    }
}

async fn serve_txs(state: AppState, headers: HeaderMap, addr: Addr) -> Result<Response> {
    Ok(state
        .read_admitted(move |q| {
            let resolved = q.resolve_addr_txs(&addr, MEMPOOL_PAGE, CHAIN_PAGE, TXS_TOTAL_TARGET)?;
            // The confirmed selection is fixed by its full activity anchor. Only
            // the bounded, already-captured mempool bodies contribute dynamic JSON.
            // Revalidation never loads or serializes confirmed transaction bodies.
            let params = {
                let source = to_vec(&(
                    "addr-txs-v2",
                    resolved.chain_anchor(),
                    resolved.mempool_transactions(),
                ))?;
                CacheParams::resolve(
                    &AppState::representation_strategy(
                        Version::ONE,
                        RepresentationId::content(&source),
                    ),
                    CdnCacheMode::Live,
                )
            };
            if params.matches_etag(&headers) {
                return Ok(Response::new_not_modified(&params));
            }
            let transactions = q.addr_txs_resolved(resolved)?;
            let bytes = Bytes::from(to_vec(&transactions)?);
            Ok(AppState::assemble_response(
                params,
                bytes,
                HeaderMap::insert_content_type_application_json,
            ))
        })
        .await?)
}

/// Validate policy and select the representation in one admitted worker. Do not
/// carry a publication guard across a second admission wait.
async fn serve_utxos(state: AppState, headers: HeaderMap, addr: Addr) -> Result<Response> {
    let max_utxos = state.max_utxos;
    Ok(state
        .read_admitted(move |q| {
            let resolved = q.resolve_addr_utxos(&addr, max_utxos)?;
            let params = CacheParams::resolve(
                &CacheStrategy::Live(format!("addr-utxos-v2-{}", resolved.block_hash()).into()),
                CdnCacheMode::Live,
            );
            if params.matches_etag(&headers) {
                return Ok(Response::new_not_modified(&params));
            }
            let (utxos, _) = q.addr_utxos_resolved(resolved, max_utxos)?;
            let bytes = Bytes::from(to_vec(&utxos)?);
            Ok(AppState::assemble_response(
                params,
                bytes,
                HeaderMap::insert_content_type_application_json,
            ))
        })
        .await?)
}
