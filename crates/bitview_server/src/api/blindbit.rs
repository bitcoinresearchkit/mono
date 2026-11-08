//! BlindBit oracle v1's HTTP API, the one silent payments light wallets speak (spdk and Dana, Bull
//! Bitcoin, blindbitd), under `/blindbit/mainnet/` as on silentpayments.dev's public oracle: a
//! wallet switches by changing the host.

use aide::axum::{
    ApiRouter,
    routing::{get_with, post_with},
};
use axum::{
    Json,
    extract::{DefaultBodyLimit, Path, Query as QueryParams},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use bitview_query::{BlindbitBlock, Query, Result as QueryResult};
use bitview_types::{
    BlindbitBlockHash, BlindbitBlockHeight, BlindbitFilter, BlindbitInfo, BlindbitSpentIndex,
    BlindbitUtxo,
};
use brk_types::{Height, Tweak, Txid};
use serde::Serialize;
use serde_json::to_vec;
use tower_http::limit::RequestBodyLimitLayer;

use super::broadcast::{self, BroadcastPermit};
use crate::{
    AppState, CacheParams,
    api::tags,
    error::Result,
    extended::{HeaderMapExtended, TransformResponseExtended},
    params::{BlindbitFilterParam, DustLimitQuery, Empty, ForwardTx, HeightParam},
    request_state::RequestState,
};

/// The routes' prefix: BlindBit's paths, under the public oracle's mount point.
const PREFIX: &str = "/blindbit/mainnet";

/// How long a per-block answer stays true.
#[derive(Clone, Copy)]
enum Lifetime {
    /// As long as the block: it changes only if a reorg replaces the block.
    Block,
    /// Until one of the block's Taproot outputs is spent: it says what's spent as of the tip.
    Spends,
}

impl AppState {
    /// A per-block route: `route` names its cache tags (with what else selects the answer), and
    /// `build` reads it.
    async fn respond_blindbit<T: Serialize + 'static>(
        &self,
        headers: HeaderMap,
        height: Height,
        route: String,
        lifetime: Lifetime,
        build: impl FnOnce(&Query, &BlindbitBlock) -> QueryResult<T> + Clone + Send + 'static,
    ) -> Result<Response> {
        let cdn_cache_mode = self.cdn_cache_mode;
        Ok(self
            .respond_admitted(
                &self.chain_query,
                &self.chain_bodies,
                headers,
                "application/json",
                move |q| {
                    let block = q.blindbit_block(height)?;
                    let tag = format!("blindbit-v1-{route}-{height}-{}", block.hash());
                    let params = match lifetime {
                        Lifetime::Block => CacheParams::revalidate(tag)
                            .stable_if(block.is_settled(), cdn_cache_mode),
                        Lifetime::Spends => {
                            let spends = q.blindbit_spends(&block)?;
                            CacheParams::live(format!("{tag}-{}", spends.key()))
                                .stable_if(spends.is_final(), cdn_cache_mode)
                        }
                    };
                    Ok((block, params))
                },
                move |q, block| Ok(to_vec(&build(q, &block)?)?.into()),
            )
            .await?)
    }

    /// A route about the tip alone.
    async fn respond_blindbit_tip<T: Serialize + 'static>(
        &self,
        headers: HeaderMap,
        route: &'static str,
        build: fn(Height) -> T,
    ) -> Result<Response> {
        Ok(self
            .respond_admitted(
                &self.chain_query,
                &self.chain_bodies,
                headers,
                "application/json",
                move |q| {
                    let (height, hash) = q.blindbit_tip()?;
                    Ok((height, CacheParams::live(format!("blindbit-v1-{route}-{hash}"))))
                },
                move |_, height| Ok(to_vec(&build(height))?.into()),
            )
            .await?)
    }
}

fn info(height: Height) -> BlindbitInfo {
    BlindbitInfo {
        network: "main".into(),
        height,
        tweaks_only: false,
        tweaks_full_basic: false,
        tweaks_full_with_dust_filter: true,
        tweaks_cut_through_with_dust_filter: true,
    }
}

/// Submits a transaction to the node, BlindBit's way: its hex as `data` in a JSON body. Answers
/// its txid as a JSON string (BlindBit answers an empty 200, which spdk can't read).
async fn forward_tx(
    permit: BroadcastPermit,
    _: Empty,
    RequestState(state): RequestState,
    Json(body): Json<ForwardTx>,
) -> Result<Response> {
    let txid = broadcast::submit(permit, &state, &body.data).await?;
    let mut response = Json(txid).into_response();
    let headers = response.headers_mut();
    headers.insert_cache_control("no-store");
    headers.insert_cdn_cache_control("no-store");
    Ok(response)
}

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router
        .api_route(
            &format!("{PREFIX}/info"),
            get_with(
                async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                    state.respond_blindbit_tip(headers, "info", info).await
                },
                |op| {
                    op.id("get_blindbit_info")
                        .tag(tags::SILENT_PAYMENTS)
                        .mcp_ignore()
                        .summary("Server info")
                        .description("The network, the last indexed block, and which tweak modes the server serves: the full index and cut-through, both taking a dust limit.")
                        .json_response::<BlindbitInfo>()
                        .not_modified()
                },
            ),
        )
        .api_route(
            &format!("{PREFIX}/block-height"),
            get_with(
                async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                    state
                        .respond_blindbit_tip(headers, "block-height", |block_height| {
                            BlindbitBlockHeight { block_height }
                        })
                        .await
                },
                |op| {
                    op.id("get_blindbit_block_height")
                        .tag(tags::SILENT_PAYMENTS)
                        .mcp_ignore()
                        .summary("Last indexed block")
                        .description("The height of the last indexed block.")
                        .json_response::<BlindbitBlockHeight>()
                        .not_modified()
                },
            ),
        )
        .api_route(
            &format!("{PREFIX}/block-hash/{{height}}"),
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<HeightParam>,
                       _: Empty,
                       RequestState(state): RequestState| {
                    state
                        .respond_blindbit(
                            headers,
                            path.height,
                            "block-hash".into(),
                            Lifetime::Block,
                            |_, block| Ok(BlindbitBlockHash { block_hash: *block.hash() }),
                        )
                        .await
                },
                |op| {
                    op.id("get_blindbit_block_hash")
                        .tag(tags::SILENT_PAYMENTS)
                        .mcp_ignore()
                        .summary("Block hash")
                        .description("The hash of the block at a height.")
                        .json_response::<BlindbitBlockHash>()
                        .not_modified()
                        .not_found()
                },
            ),
        )
        .api_route(
            &format!("{PREFIX}/tweak-index/{{height}}"),
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<HeightParam>,
                       QueryParams(query): QueryParams<DustLimitQuery>,
                       RequestState(state): RequestState| {
                    let dust_limit = query.limit()?;
                    state
                        .respond_blindbit(
                            headers,
                            path.height,
                            format!("tweak-index-{dust_limit}"),
                            Lifetime::Block,
                            move |q, block| q.blindbit_tweak_index(block, dust_limit),
                        )
                        .await
                },
                |op| {
                    op.id("get_blindbit_tweak_index")
                        .tag(tags::SILENT_PAYMENTS)
                        .mcp_ignore()
                        .summary("Tweak index")
                        .description("The tweaks of a block's eligible transactions with a Taproot output of at least `dustLimit` sats, as compressed points in hex, in block order. A light wallet multiplies each by its scan key and tests the resulting output keys against the block's `filter/new-utxos`. `dustLimit` is required and must be at least 600, the least value held (BlindBit's is optional, and 0 by default).")
                        .json_response::<Vec<Tweak>>()
                        .not_modified()
                        .not_found()
                },
            ),
        )
        .api_route(
            &format!("{PREFIX}/tweaks/{{height}}"),
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<HeightParam>,
                       QueryParams(query): QueryParams<DustLimitQuery>,
                       RequestState(state): RequestState| {
                    let dust_limit = query.limit()?;
                    state
                        .respond_blindbit(
                            headers,
                            path.height,
                            format!("tweaks-{dust_limit}"),
                            Lifetime::Spends,
                            move |q, block| q.blindbit_tweaks(block, dust_limit),
                        )
                        .await
                },
                |op| {
                    op.id("get_blindbit_tweaks")
                        .tag(tags::SILENT_PAYMENTS)
                        .mcp_ignore()
                        .summary("Cut-through tweaks")
                        .description("`tweak-index`'s tweaks of the transactions that still have an unspent Taproot output of at least `dustLimit` sats, as of the last indexed block: what a wallet restoring its balance alone scans. `dustLimit` as in `tweak-index`.")
                        .json_response::<Vec<Tweak>>()
                        .not_modified()
                        .not_found()
                },
            ),
        )
        .api_route(
            &format!("{PREFIX}/filter/{{kind}}/{{height}}"),
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<BlindbitFilterParam>,
                       _: Empty,
                       RequestState(state): RequestState| {
                    let kind = path.kind;
                    state
                        .respond_blindbit(
                            headers,
                            path.height,
                            format!("filter-{kind}"),
                            Lifetime::Block,
                            move |q, block| q.blindbit_filter(block, kind),
                        )
                        .await
                },
                |op| {
                    op.id("get_blindbit_filter")
                        .tag(tags::SILENT_PAYMENTS)
                        .mcp_ignore()
                        .summary("Block filter")
                        .description("A BIP158 Golomb-coded set (P = 19, M = 784,931, keyed by the block hash) of the block's new Taproot output keys (`new-utxos`, the x-only keys) or of its spends of Taproot outputs (`spent`, hashed as in `spent-index`). Repeated elements count once, as in BIP158 (BlindBit repeats them), which leaves every membership test's answer the same.")
                        .json_response::<BlindbitFilter>()
                        .not_modified()
                        .not_found()
                },
            ),
        )
        .api_route(
            &format!("{PREFIX}/spent-index/{{height}}"),
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<HeightParam>,
                       _: Empty,
                       RequestState(state): RequestState| {
                    state
                        .respond_blindbit(
                            headers,
                            path.height,
                            "spent-index".into(),
                            Lifetime::Block,
                            |q, block| q.blindbit_spent_index(block),
                        )
                        .await
                },
                |op| {
                    op.id("get_blindbit_spent_index")
                        .tag(tags::SILENT_PAYMENTS)
                        .mcp_ignore()
                        .summary("Spent index")
                        .description("The block's spends of Taproot outputs, each the first 8 bytes of `sha256(txid ‖ vout ‖ block hash)` in hex (the txid and this block's hash in internal byte order, the vout little-endian): a wallet hashes its own outpoints to find which this block spends. Always an object, even when empty (BlindBit then answers `[]`, which spdk can't read).")
                        .json_response::<BlindbitSpentIndex>()
                        .not_modified()
                        .not_found()
                },
            ),
        )
        .api_route(
            &format!("{PREFIX}/utxos/{{height}}"),
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<HeightParam>,
                       _: Empty,
                       RequestState(state): RequestState| {
                    state
                        .respond_blindbit(
                            headers,
                            path.height,
                            "utxos".into(),
                            Lifetime::Spends,
                            |q, block| q.blindbit_utxos(block),
                        )
                        .await
                },
                |op| {
                    op.id("get_blindbit_utxos")
                        .tag(tags::SILENT_PAYMENTS)
                        .mcp_ignore()
                        .summary("Taproot outputs")
                        .description("Every Taproot output of the block's eligible transactions (those `tweak-index` lists), in block order, each marked spent or not as of the last indexed block. `block_height` and `timestamp` are the block's (BlindBit sends 0 for the height, and for the time of spent outputs).")
                        .json_response::<Vec<BlindbitUtxo>>()
                        .not_modified()
                        .not_found()
                },
            ),
        )
        .api_route(
            &format!("{PREFIX}/forward-tx"),
            post_with(forward_tx, |op| {
                op.id("post_blindbit_forward_tx")
                    .tag(tags::SILENT_PAYMENTS)
                    .mcp_ignore()
                    .summary("Broadcast transaction")
                    .description("Submits a raw transaction, `{\"data\": \"<hex>\"}`, to the node, as `POST /api/tx` does. Returns its txid as a JSON string. No responses are cached.")
                    .json_response::<Txid>()
                    .error_response::<413>("Request body too large")
            })
            .layer((
                RequestBodyLimitLayer::new(broadcast::MAX_BODY_BYTES + 64),
                DefaultBodyLimit::disable(),
            )),
        )
}
