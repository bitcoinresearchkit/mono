use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    body::Bytes,
    extract::{Path, Query},
    http::HeaderMap,
    response::Response,
};
use bitview_primitives::BlockHashPrefix;
use bitview_query::RepresentationId;
use bitview_types::{AddressHashPrefixMatches, AddressStats, AddressValidation, Utxo};
use brk_types::{Address, Transaction, Txid, Version};
use serde_json::to_vec;

use crate::{
    AppState, CacheParams,
    api::tags,
    extended::{ResponseExtended, TransformResponseExtended},
    params::{
        AddressAfterTxidParam, AddressChainTxsQuery, AddressHashPrefixParam,
        AddressMempoolTxsQuery, AddressParam, AddressTxsQuery, Empty, ValidateAddressParam,
        page_size,
    },
    request_state::RequestState,
};

/// Esplora's page sizes, defaults and maxima of `max_txs`, as mempool.space's: `/txs` and
/// `/txs/mempool` return up to `TXS_PAGE` (on `/txs` mempool entries first, the chain filling
/// the rest), `/txs/chain` up to `CHAIN_PAGE`.
const TXS_PAGE: usize = 50;
const CHAIN_PAGE: usize = 25;

impl AppState {
    async fn respond_addr_chain_txs(
        &self,
        headers: HeaderMap,
        addr: Address,
        after_txid: Option<Txid>,
        max_txs: Option<usize>,
    ) -> Response {
        self.respond_read(
            headers,
            "application/json",
            move |q| {
                let source =
                    q.resolve_addr_chain_txs(&addr, after_txid, page_size(max_txs, CHAIN_PAGE))?;
                let params = CacheParams::activity_bound(
                    Version::ONE,
                    BlockHashPrefix::from(&source.activity_anchor()),
                );
                Ok((source, params))
            },
            |q, source| Ok(Bytes::from(to_vec(&q.addr_txs_chain_resolved(source)?)?)),
        )
        .await
    }
}

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router.api_route(
        "/api/address/hash-prefix/{address_type}/{prefix}",
        get_with(async |
            headers: HeaderMap,
            Path(path): Path<AddressHashPrefixParam>,
            _: Empty,
            RequestState(state): RequestState
        | {
            state.respond_json_content(&headers, move |q| {
                q.addr_hash_prefix_matches(path.address_type, &path.prefix)
            }).await
        }, |op| op
            .id("get_address_hash_prefix_matches")
            .tag(tags::ADDRESSES)
            .summary("Address hash-prefix matches")
            .description("Find addresses by address type and by the first 1-16 hex nibbles of RapidHash v3 over the raw address payload bytes. Intended for privacy-preserving client-side wallet discovery without sending raw addresses or xpubs. Fetch metadata with `GET /api/address/{address}`.")
            .json_response::<AddressHashPrefixMatches>()
            .not_modified()
        ),
    )
    .api_route(
        "/api/address/{address}",
        get_with(async |
            headers: HeaderMap,
            Path(path): Path<AddressParam>,
            _: Empty,
            RequestState(state): RequestState
        | {
            state
                .respond_json_content(&headers, move |q| q.addr(path.addr))
                .await
        }, |op| op
            .id("get_address")
            .tag(tags::ADDRESSES)
            .summary("Address information")
            .description("Balance and transaction counts. Supports P2PKH, P2SH, P2WPKH, P2WSH, P2TR and P2A addresses, and P2PK public keys in hex (33 or 65 bytes).\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address)*")
            .json_response::<AddressStats>()
            .not_modified()
        ),
    )
    .api_route(
        "/api/address/{address}/txs",
        get_with(async |
            headers: HeaderMap,
            Path(path): Path<AddressParam>,
            Query(query): Query<AddressTxsQuery>,
            RequestState(state): RequestState
        | {
            serve_txs(state, headers, path.addr, query).await
        }, |op| op
            .id("get_address_txs")
            .tag(tags::ADDRESSES)
            .summary("Address transactions")
            .description("Get transaction history for an address, newest first: up to `max_txs` (50 by default and at most), mempool transactions first and confirmed ones filling the rest. To continue, pass the last returned txid as `after_txid`, or request `GET /api/address/{address}/txs/chain/{after_txid}` for confirmed history.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions)*")
            .json_response::<Vec<Transaction>>()
            .not_modified()
            .unknown_cursor()
        ),
    )
    .api_route(
        "/api/address/{address}/txs/chain",
        get_with(async |
            headers: HeaderMap,
            Path(path): Path<AddressParam>,
            Query(query): Query<AddressChainTxsQuery>,
            RequestState(state): RequestState
        | {
            state.respond_addr_chain_txs(headers, path.addr, None, query.max_txs).await
        }, |op| op
            .id("get_address_confirmed_txs")
            .tag(tags::ADDRESSES)
            .summary("Address confirmed transactions")
            .description("Get the newest confirmed transactions for an address, up to `max_txs` (25 by default and at most). For pagination, request `GET /api/address/{address}/txs/chain/{after_txid}` with the last returned txid.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*")
            .json_response::<Vec<Transaction>>()
            .not_modified()
        ),
    )
    .api_route(
        "/api/address/{address}/txs/chain/{after_txid}",
        get_with(async |
            headers: HeaderMap,
            Path(path): Path<AddressAfterTxidParam>,
            Query(query): Query<AddressChainTxsQuery>,
            RequestState(state): RequestState
        | {
            state
                .respond_addr_chain_txs(headers, path.addr, Some(path.after_txid), query.max_txs)
                .await
        }, |op| op
            .id("get_address_confirmed_txs_after")
            .tag(tags::ADDRESSES)
            .summary("Address confirmed transactions (paginated)")
            .description("Get the next confirmed transactions strictly older than `after_txid`, up to `max_txs` (25 by default and at most); a txid outside the address's history answers `[]` (Esplora-canonical pagination form, matches mempool.space).\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*")
            .json_response::<Vec<Transaction>>()
            .not_modified()
        ),
    )
    .api_route(
        "/api/address/{address}/txs/mempool",
        get_with(async |
            headers: HeaderMap,
            Path(path): Path<AddressParam>,
            Query(query): Query<AddressMempoolTxsQuery>,
            RequestState(state): RequestState
        | {
            state.respond_read(
                headers,
                "application/json",
                move |q| {
                    let resolved = q
                        .resolve_addr_mempool_txs(&path.addr, page_size(query.max_txs, TXS_PAGE))?;
                    let params = live_params(resolved.identity());
                    Ok((resolved, params))
                },
                |_, resolved| Ok(Bytes::from(resolved.json())),
            ).await
        }, |op| op
            .id("get_address_mempool_txs")
            .tag(tags::ADDRESSES)
            .summary("Address mempool transactions")
            .description("Get unconfirmed transactions for an address from the mempool, newest first, up to `max_txs` (50 by default and at most).\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-mempool)*")
            .json_response::<Vec<Transaction>>()
            .not_modified()
        ),
    )
    .api_route(
        "/api/address/{address}/utxo",
        get_with(async |
            headers: HeaderMap,
            Path(path): Path<AddressParam>,
            _: Empty,
            RequestState(state): RequestState
        | {
            serve_utxos(state, headers, path.addr).await
        }, |op| op
            .id("get_address_utxos")
            .tag(tags::ADDRESSES)
            .summary("Address UTXOs")
            .description("Get unspent transaction outputs (UTXOs) for an address, including unconfirmed ones; outputs that mempool transactions spend are left out.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-utxo)*")
            .json_response::<Vec<Utxo>>()
            .not_modified()
        ),
    )
    .api_route(
        "/api/v1/validate-address/{address}",
        get_with(async |
            headers: HeaderMap,
            Path(path): Path<ValidateAddressParam>,
            _: Empty
        | -> crate::error::Result<Response> {
            let validation = AddressValidation::from_addr(path.checked()?);
            Ok(Response::cached_json(&headers, &CacheParams::deploy(), move || validation))
        }, |op| op
            .id("validate_address")
            .tag(tags::ADDRESSES)
            .summary("Validate address")
            .description("Validate a Bitcoin address and get information about its type and scriptPubKey. Returns `isvalid: false` with an error message for invalid addresses; a string that isn't 2 to 130 letters and digits answers 400.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-validate)*")
            .json_response::<AddressValidation>()
            .not_modified()
        ),
    )
}

fn live_params(identity: RepresentationId) -> CacheParams {
    CacheParams::representation(Version::ONE, identity)
}

async fn serve_txs(
    state: AppState,
    headers: HeaderMap,
    addr: Address,
    query: AddressTxsQuery,
) -> Response {
    state
        .respond_read(
            headers,
            "application/json",
            move |q| {
                let resolved = q.resolve_addr_txs(
                    &addr,
                    query.after_txid,
                    page_size(query.max_txs, TXS_PAGE),
                )?;
                let params = live_params(resolved.identity());
                Ok((resolved, params))
            },
            |q, resolved| Ok(Bytes::from(to_vec(&q.addr_txs_resolved(resolved)?)?)),
        )
        .await
}

/// Validate policy and select the representation in one admitted worker.
async fn serve_utxos(state: AppState, headers: HeaderMap, addr: Address) -> Response {
    let max_utxos = state.max_utxos;
    state
        .respond_read(
            headers,
            "application/json",
            move |q| {
                let resolved = q.resolve_addr_utxos(&addr, max_utxos)?;
                let params =
                    CacheParams::live(format!("addr-utxos-v3-{}", resolved.representation()));
                Ok((resolved, params))
            },
            move |q, resolved| {
                Ok(Bytes::from(to_vec(
                    &q.addr_utxos_resolved(resolved, max_utxos)?,
                )?))
            },
        )
        .await
}
