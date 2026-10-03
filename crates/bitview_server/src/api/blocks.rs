use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    body::Bytes,
    extract::Path,
    http::{HeaderMap, HeaderValue, Method, header},
    response::Response,
};
use bitcoin::hashes::{Hash, HashEngine, sha256};
use bitview_query::{Query, ResolvedBlocks, ResolvedBlocksV1};
use bitview_types::{BlockInfo, BlockInfoV1, BlockStatus, BlockTimestamp, BlockTxIndex, Hex};
use brk_error::{Error as QueryError, Result as QueryResult};
use brk_types::{BlockHash, Dollars, Height, Timestamp, Transaction, Txid};
use serde::Serialize;
use serde_json::to_vec;

use crate::{
    AppState, CacheParams, CacheStrategy, CdnCacheMode, Error,
    extended::{ResponseExtended, TransformResponseExtended},
    params::{
        BlockHashParam, BlockHashStartIndex, BlockHashTxIndex, Empty, HeightParam, TimestampParam,
    },
    raw_body::RawBodyPermit,
    request_state::RequestState,
};

const BLOCK_TXS_PAGE_SIZE: u32 = 25;
const BASE_BLOCK_SCHEMA: &str = "v3";
const HEADER_SCHEMA: &str = "header-v2";
const RAW_SCHEMA: &str = "raw-v2";
pub const V1_BLOCK_SCHEMA: &str = "v1-4";

fn recent_blocks_params(tip: Option<BlockHash>) -> CacheParams {
    let tag = match tip {
        Some(hash) => format!("blocks2-{hash}"),
        None => "blocks2-empty".to_owned(),
    };
    CacheParams::resolve(&CacheStrategy::Live(tag.into()), CdnCacheMode::Live)
}

// The namespace covers the row formulas and bundled pool catalog. Bump it when
// either changes. Prices also participate because oracle checkpoints can produce
// different prices for the same chain. Hash exactly the values used by the body.
pub fn blocks_v1_identity(anchor: Option<BlockHash>, prices: &[Dollars]) -> sha256::Hash {
    let mut engine = sha256::Hash::engine();
    engine.input(&[u8::from(anchor.is_some())]);
    if let Some(hash) = anchor {
        engine.input(&*hash);
    }
    for price in prices {
        engine.input(&f64::from(*price).to_bits().to_le_bytes());
    }
    sha256::Hash::from_engine(engine)
}

fn blocks_v1_params(anchor: Option<BlockHash>, prices: &[Dollars]) -> CacheParams {
    let identity = blocks_v1_identity(anchor, prices);
    let tag = format!("blocks-{V1_BLOCK_SCHEMA}-{identity}");
    CacheParams::resolve(&CacheStrategy::Live(tag.into()), CdnCacheMode::Live)
}

fn block_v1_params(snapshot: &ResolvedBlocksV1) -> QueryResult<CacheParams> {
    let height = snapshot
        .last_height()
        .ok_or(QueryError::Internal("Missing resolved block"))?;
    let identity = blocks_v1_identity(snapshot.anchor(), snapshot.prices());
    Ok(CacheParams::resolve(
        &CacheStrategy::Live(format!("block-{V1_BLOCK_SCHEMA}-{height}-{identity}").into()),
        CdnCacheMode::Live,
    ))
}

fn block_params(snapshot: &ResolvedBlocks, schema: &str) -> QueryResult<CacheParams> {
    let height = snapshot
        .last_height()
        .ok_or(QueryError::Internal("Missing resolved block"))?;
    let hash = snapshot
        .anchor()
        .ok_or(QueryError::Internal("Missing resolved block hash"))?;
    // Availability is best-chain-only, even at historical heights. A reorg can
    // remove the resource, so depth alone cannot justify immutable freshness.
    Ok(CacheParams::revalidate(
        format!("block-{schema}-{height}-{hash}").into(),
    ))
}

// A hint only: at most one candidate is checked against the full canonical hash.
// Actual conditional matching still uses the complete computed ETag afterward.
fn block_height_hint(headers: &HeaderMap, schema: &str) -> Option<Height> {
    headers
        .get_all(header::IF_NONE_MATCH)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .find_map(|tag| {
            let tag = tag.trim();
            let tag = tag.strip_prefix("W/").unwrap_or(tag);
            let token = tag.strip_prefix('"')?.strip_suffix('"')?;
            let suffix = token
                .strip_prefix("block-")?
                .strip_prefix(schema)?
                .strip_prefix('-')?;
            let (height, _) = suffix.split_once('-')?;
            height.parse::<u32>().ok().map(Height::from)
        })
}

fn block_json_response(params: CacheParams, value: &impl Serialize) -> QueryResult<Response> {
    let bytes = Bytes::from(to_vec(value)?);
    Ok(AppState::assemble_response(params, bytes, |headers| {
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
    }))
}

impl AppState {
    async fn respond_block_tip(&self, headers: HeaderMap, hash: bool) -> Result<Response, Error> {
        self.read_block_response(move |q| {
            let snapshot = q.resolve_blocks(None, 1)?;
            let value = if hash {
                snapshot
                    .anchor()
                    .ok_or(QueryError::Internal("Missing chain tip"))?
                    .to_string()
            } else {
                snapshot
                    .last_height()
                    .ok_or(QueryError::Internal("Missing chain tip"))?
                    .to_string()
            };
            // The owned value is the whole representation. In particular, a
            // same-height reorg changes the hash endpoint but not the height.
            let params = CacheParams::revalidate(format!("block-tip-v2-{value}").into());
            if params.matches_etag(&headers) {
                return Ok(ResponseExtended::new_not_modified(&params));
            }
            Ok(AppState::assemble_response(
                params,
                Bytes::from(value),
                |headers| {
                    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/plain"));
                },
            ))
        })
        .await
    }

    pub async fn respond_block_raw(
        &self,
        headers: HeaderMap,
        hash: BlockHash,
        method: Method,
    ) -> Result<Response, Error> {
        if let Some(height) = block_height_hint(&headers, RAW_SCHEMA)
            && let Some(snapshot) =
                self.preflight(|q| q.try_resolve_block_snapshot(&hash, height))?
        {
            let params = block_params(&snapshot, RAW_SCHEMA)?;
            if params.matches_etag(&headers) {
                return Ok(ResponseExtended::new_not_modified(&params));
            }
        }
        let budget = self.raw_block_bodies.clone();
        Ok(self
            .read_body(
                &self.sync_query,
                &self.raw_block_bodies,
                move |q, permit| {
                    let snapshot = q.resolve_block_snapshot(&hash)?;
                    let params = block_params(&snapshot, RAW_SCHEMA)?;
                    if params.matches_etag(&headers) {
                        return Ok(Some(ResponseExtended::new_not_modified(&params)));
                    }
                    let content_headers = |headers: &mut HeaderMap| {
                        headers.insert(
                            header::CONTENT_TYPE,
                            HeaderValue::from_static("application/octet-stream"),
                        );
                    };
                    if method == Method::HEAD {
                        let length = snapshot.anchor_raw_size(q)?;
                        return Ok(Some(AppState::assemble_response(
                            params,
                            Bytes::new(),
                            |headers| {
                                content_headers(headers);
                                headers.insert(header::CONTENT_LENGTH, length.into());
                            },
                        )));
                    }
                    let Some(permit) = permit.or_else(|| RawBodyPermit::try_acquire(&budget))
                    else {
                        return Ok(None);
                    };
                    let bytes = Bytes::from(snapshot.anchor_raw(q)?);
                    Ok(Some(permit.response(params, bytes, content_headers)))
                },
            )
            .await?)
    }

    pub async fn respond_block_timestamp(
        &self,
        headers: HeaderMap,
        timestamp: Timestamp,
    ) -> Result<Response, Error> {
        self.read_block_response(move |q| {
            let block = q.resolve_block_by_timestamp(timestamp)?;
            let params =
                CacheParams::revalidate(format!("block-timestamp-v2-{}", block.hash()).into());
            if params.matches_etag(&headers) {
                return Ok(ResponseExtended::new_not_modified(&params));
            }
            block_json_response(params, &block.into_value())
        })
        .await
    }

    pub async fn respond_block_height(
        &self,
        headers: HeaderMap,
        height: Height,
    ) -> Result<Response, Error> {
        // The owned hash is the entire representation: no second source read,
        // retained snapshot or body cache is needed after resolution.
        let respond = move |hash: BlockHash| {
            let body = hash.to_string();
            let params = CacheParams::revalidate(format!("block-height-v2-{body}").into());
            if params.matches_etag(&headers) {
                return Ok(ResponseExtended::new_not_modified(&params));
            }
            Ok(AppState::assemble_response(
                params,
                Bytes::from(body),
                |headers| {
                    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/plain"));
                },
            ))
        };
        if let Some(hash) = self.preflight(|q| q.try_resolve_block_hash(height))? {
            return Ok(respond(hash)?);
        }
        self.read_block_response(move |q| respond(q.resolve_block_hash(height)?))
            .await
    }

    pub async fn respond_block(
        &self,
        headers: HeaderMap,
        hash: BlockHash,
    ) -> Result<Response, Error> {
        self.respond_exact_block(
            headers,
            hash,
            BASE_BLOCK_SCHEMA,
            None,
            |q, snapshot, params| {
                let block = snapshot
                    .build(q)?
                    .pop()
                    .ok_or(QueryError::Internal("Missing resolved block"))?;
                block_json_response(params, &block)
            },
        )
        .await
    }

    pub async fn respond_block_header(
        &self,
        headers: HeaderMap,
        hash: BlockHash,
    ) -> Result<Response, Error> {
        self.respond_exact_block(headers, hash, HEADER_SCHEMA, None, |q, snapshot, params| {
            let bytes = Bytes::from(snapshot.anchor_header_hex(q)?);
            Ok(AppState::assemble_response(params, bytes, |headers| {
                headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/plain"));
            }))
        })
        .await
    }

    /// Shared exact-block selection and admission; the callback consumes a
    /// stable snapshot and builds the response only after conditional checks.
    async fn respond_exact_block(
        &self,
        headers: HeaderMap,
        hash: BlockHash,
        schema: &'static str,
        tx_index: Option<BlockTxIndex>,
        build: impl FnOnce(&Query, ResolvedBlocks, CacheParams) -> QueryResult<Response>
        + Clone
        + Send
        + 'static,
    ) -> Result<Response, Error> {
        if let Some(height) = block_height_hint(&headers, schema)
            && let Some(snapshot) =
                self.preflight(|q| q.try_resolve_block_snapshot(&hash, height))?
        {
            if let Some(index) = tx_index {
                self.sync(|q| snapshot.validate_tx_index(q, index))?;
            }
            let params = block_params(&snapshot, schema)?;
            if params.matches_etag(&headers) {
                return Ok(ResponseExtended::new_not_modified(&params));
            }
        }

        self.read_block_response(move |q| {
            let snapshot = q.resolve_block_snapshot(&hash)?;
            if let Some(index) = tx_index {
                snapshot.validate_tx_index(q, index)?;
            }
            let params = block_params(&snapshot, schema)?;
            if params.matches_etag(&headers) {
                return Ok(ResponseExtended::new_not_modified(&params));
            }
            build(q, snapshot, params)
        })
        .await
    }

    pub async fn respond_block_v1(
        &self,
        headers: HeaderMap,
        hash: BlockHash,
    ) -> Result<Response, Error> {
        if let Some(height) = block_height_hint(&headers, V1_BLOCK_SCHEMA)
            && let Some(snapshot) = self.preflight(|q| q.try_resolve_block_v1(&hash, height))?
        {
            let params = block_v1_params(&snapshot)?;
            if params.matches_etag(&headers) {
                return Ok(ResponseExtended::new_not_modified(&params));
            }
        }

        self.read_block_response(move |q| {
            let snapshot = q.resolve_block_v1(&hash)?;
            let params = block_v1_params(&snapshot)?;
            if params.matches_etag(&headers) {
                return Ok(ResponseExtended::new_not_modified(&params));
            }
            let block = snapshot
                .build(q)?
                .pop()
                .ok_or(QueryError::Internal("Missing resolved block"))?;
            block_json_response(params, &block)
        })
        .await
    }

    /// Retry only the read, releasing guards and worker admission while waiting
    /// for publication. Resolve snapshots inside `build` on every attempt.
    /// Actual running work retains admission even after request cancellation.
    async fn read_block_response(
        &self,
        build: impl FnOnce(&Query) -> QueryResult<Response> + Clone + Send + 'static,
    ) -> Result<Response, Error> {
        Ok(self.read_admitted(build).await?)
    }

    pub async fn respond_blocks_v1(
        &self,
        headers: HeaderMap,
        start_height: Option<Height>,
    ) -> Result<Response, Error> {
        if headers.contains_key(header::IF_NONE_MATCH)
            && let Some(snapshot) = self.preflight(|q| q.try_resolve_blocks_v1(start_height, 15))?
        {
            let params = blocks_v1_params(snapshot.anchor(), snapshot.prices());
            if params.matches_etag(&headers) {
                return Ok(ResponseExtended::new_not_modified(&params));
            }
        }

        // The optimistic snapshot has been dropped before admission.
        self.read_block_response(move |q| {
            let snapshot = q.resolve_blocks_v1(start_height, 15)?;
            let params = blocks_v1_params(snapshot.anchor(), snapshot.prices());
            if params.matches_etag(&headers) {
                return Ok(ResponseExtended::new_not_modified(&params));
            }
            block_json_response(params, &snapshot.build(q)?)
        })
        .await
    }

    pub async fn respond_blocks(
        &self,
        headers: HeaderMap,
        start_height: Option<Height>,
    ) -> Result<Response, Error> {
        if headers.contains_key(header::IF_NONE_MATCH)
            && let Some(snapshot) = self.preflight(|q| q.try_resolve_blocks(start_height, 10))?
        {
            let params = recent_blocks_params(snapshot.anchor());
            if params.matches_etag(&headers) {
                return Ok(ResponseExtended::new_not_modified(&params));
            }
        }

        // Do not retain a publication guard while queued for admission. Re-resolve
        // after admission so a concurrent update cannot attach an old tag to new rows.
        self.read_block_response(move |q| {
            let snapshot = q.resolve_blocks(start_height, 10)?;
            let params = recent_blocks_params(snapshot.anchor());
            if params.matches_etag(&headers) {
                return Ok(ResponseExtended::new_not_modified(&params));
            }
            block_json_response(params, &snapshot.build(q)?)
        })
        .await
    }
}

pub trait BlockRoutes {
    fn add_block_routes(self) -> Self;
}

impl BlockRoutes for ApiRouter<AppState> {
    fn add_block_routes(self) -> Self {
        self.api_route(
                "/api/block/{hash}",
                get_with(
                    async |headers: HeaderMap,
                           Path(path): Path<BlockHashParam>,
                           _: Empty,
                           RequestState(state): RequestState|
                           -> Result<Response, Error> {
                        state.respond_block(headers, path.hash).await
                    },
                    |op| {
                        op.id("get_block")
                            .blocks_tag()
                            .summary("Block information")
                            .description(
                                "Retrieve block information by block hash. Returns block metadata including height, timestamp, difficulty, size, weight, and transaction count.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block)*",
                            )
                            .json_response::<BlockInfo>()
                            .not_modified()
                            .bad_request()
                            .not_found()
                            .server_errors()
                    },
                ),
            )
            .api_route(
                "/api/v1/block/{hash}",
                get_with(
                    async |headers: HeaderMap,
                           Path(path): Path<BlockHashParam>,
                           _: Empty,
                           RequestState(state): RequestState|
                           -> Result<Response, Error> {
                        state.respond_block_v1(headers, path.hash).await
                    },
                    |op| {
                        op.id("get_block_v1")
                            .blocks_tag()
                            .summary("Block (v1)")
                            .description("Returns block details with extras by hash.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-v1)*")
                            .json_response::<BlockInfoV1>()
                            .not_modified()
                            .bad_request()
                            .not_found()
                            .server_errors()
                    },
                ),
            )
            .api_route(
                "/api/block/{hash}/header",
                get_with(
                    async |headers: HeaderMap,
                           Path(path): Path<BlockHashParam>,
                           _: Empty,
                           RequestState(state): RequestState|
                           -> Result<Response, Error> {
                        state.respond_block_header(headers, path.hash).await
                    },
                    |op| {
                        op.id("get_block_header")
                            .blocks_tag()
                            .summary("Block header")
                            .description("Returns the hex-encoded 80-byte block header.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-header)*")
                            .text_response::<Hex>()
                            .not_modified()
                            .bad_request()
                            .not_found()
                            .server_errors()
                    },
                ),
            )
            .api_route(
                "/api/block-height/{height}",
                get_with(
                    async |headers: HeaderMap,
                           Path(path): Path<HeightParam>,
                           _: Empty, RequestState(state): RequestState| {
                        state.respond_block_height(headers, path.height).await
                    },
                    |op| {
                        op.id("get_block_by_height")
                            .blocks_tag()
                            .summary("Block hash by height")
                            .description(
                                "Retrieve the block hash at a given height. Returns the hash as plain text.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-height)*",
                            )
                            .text_response::<BlockHash>()
                            .not_modified()
                            .bad_request()
                            .not_found()
                            .server_errors()
                    },
                ),
            )
            .api_route(
                "/api/v1/mining/blocks/timestamp/{timestamp}",
                get_with(
                    async |headers: HeaderMap,
                           Path(path): Path<TimestampParam>,
                           _: Empty, RequestState(state): RequestState| {
                        state.respond_block_timestamp(headers, path.timestamp).await
                    },
                    |op| {
                        op.id("get_block_by_timestamp")
                            .blocks_tag()
                            .summary("Block by timestamp")
                            .description("Find the block with the greatest header timestamp at or before the given UNIX timestamp, choosing the earliest height on ties.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-timestamp)*")
                            .json_response::<BlockTimestamp>()
                            .not_modified()
                            .bad_request()
                            .not_found()
                            .server_errors()
                    },
                ),
            )
            .api_route(
                "/api/block/{hash}/raw",
                get_with(
                    async |method: Method,
                           headers: HeaderMap,
                           Path(path): Path<BlockHashParam>,
                           _: Empty,
                           RequestState(state): RequestState|
                           -> Result<Response, Error> {
                        state.respond_block_raw(headers, path.hash, method).await
                    },
                    |op| {
                        op.id("get_block_raw")
                            .blocks_tag()
                            .mcp_ignore()
                            .summary("Raw block")
                            .description(
                                "Returns the raw block data in binary format.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-raw)*",
                            )
                            .binary_response()
                            .not_modified()
                            .bad_request()
                            .not_found()
                            .server_errors()
                    },
                ),
            )
            .api_route(
                "/api/block/{hash}/status",
                get_with(
                    async |headers: HeaderMap,
                           Path(path): Path<BlockHashParam>,
                           _: Empty,
                           RequestState(state): RequestState|
                           -> Result<Response, Error> {
                        let status = state.read_admitted(move |q| q.block_status(&path.hash)).await?;
                        Ok(state.respond_json_content_value(&headers, status))
                    },
                    |op| {
                        op.id("get_block_status")
                            .blocks_tag()
                            .summary("Block status")
                            .description(
                                "Retrieve the status of a block. Returns whether the block is in the best chain and, if so, its height and the hash of the next block.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-status)*",
                            )
                            .json_response::<BlockStatus>()
                            .not_modified()
                            .bad_request()
                            .not_found()
                            .server_error()
                    },
                ),
            )
            .api_route(
                "/api/blocks/tip/height",
                get_with(
                    async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                        state.respond_block_tip(headers, false).await
                    },
                    |op| {
                        op.id("get_block_tip_height")
                            .blocks_tag()
                            .summary("Block tip height")
                            .description("Returns the height of the last block.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-height)*")
                            .text_response::<Height>()
                            .not_modified()
                            .server_error()
                    },
                ),
            )
            .api_route(
                "/api/blocks/tip/hash",
                get_with(
                    async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                        state.respond_block_tip(headers, true).await
                    },
                    |op| {
                        op.id("get_block_tip_hash")
                            .blocks_tag()
                            .summary("Block tip hash")
                            .description("Returns the hash of the last block.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-hash)*")
                            .text_response::<BlockHash>()
                            .not_modified()
                            .server_error()
                    },
                ),
            )
            .api_route(
                "/api/block/{hash}/txid/{index}",
                get_with(
                    async |headers: HeaderMap,
                           Path(path): Path<BlockHashTxIndex>,
                           _: Empty,
                           RequestState(state): RequestState|
                           -> Result<Response, Error> {
                        state.respond_exact_block(headers, path.hash, "txid-v2", Some(path.index), move |q, snapshot, params| {
                            let bytes = Bytes::from(snapshot.anchor_txid(q, path.index)?.to_string());
                            Ok(AppState::assemble_response(params, bytes, |headers| {
                                headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/plain"));
                            }))
                        }).await
                    },
                    |op| {
                        op.id("get_block_txid")
                            .blocks_tag()
                            .summary("Transaction ID at index")
                            .description(
                                "Retrieve a single transaction ID at a specific index within a block. Returns plain text txid.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transaction-id)*",
                            )
                            .text_response::<Txid>()
                            .not_modified()
                            .bad_request()
                            .not_found()
                            .server_error()
                    },
                ),
            )
            .api_route(
                "/api/block/{hash}/txids",
                get_with(
                    async |headers: HeaderMap,
                           Path(path): Path<BlockHashParam>,
                           _: Empty,
                           RequestState(state): RequestState|
                           -> Result<Response, Error> {
                        state.respond_exact_block(headers, path.hash, "txids-v2", None, |q, snapshot, params| {
                            block_json_response(params, &snapshot.anchor_txids(q)?)
                        }).await
                    },
                    |op| {
                        op.id("get_block_txids")
                            .blocks_tag()
                            .mcp_ignore()
                            .summary("Block transaction IDs")
                            .description(
                                "Retrieve all transaction IDs in a block. Returns an array of txids in block order.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transaction-ids)*",
                            )
                            .json_response::<Vec<Txid>>()
                            .not_modified()
                            .bad_request()
                            .not_found()
                            .server_error()
                    },
                ),
            )
            .api_route(
                "/api/block/{hash}/txs",
                get_with(
                    async |headers: HeaderMap,
                           Path(path): Path<BlockHashParam>,
                           _: Empty,
                           RequestState(state): RequestState|
                           -> Result<Response, Error> {
                        state.respond_exact_block(headers, path.hash, "txs-v2", Some(BlockTxIndex::default()), |q, snapshot, params| {
                            block_json_response(params, &snapshot.anchor_txs(q, BlockTxIndex::default(), BLOCK_TXS_PAGE_SIZE)?)
                        }).await
                    },
                    |op| {
                        op.id("get_block_txs")
                            .blocks_tag()
                            .mcp_ignore()
                            .summary("Block transactions")
                            .description(&format!(
                                "Retrieve transactions in a block by block hash. Returns up to {} transactions starting from index 0.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*",
                                BLOCK_TXS_PAGE_SIZE
                            ))
                            .json_response::<Vec<Transaction>>()
                            .not_modified()
                            .bad_request()
                            .not_found()
                            .server_error()
                    },
                ),
            )
            .api_route(
                "/api/block/{hash}/txs/{start_index}",
                get_with(
                    async |headers: HeaderMap,
                           Path(path): Path<BlockHashStartIndex>,
                           _: Empty,
                           RequestState(state): RequestState|
                           -> Result<Response, Error> {
                        state.respond_exact_block(headers, path.hash, "txs-v2", Some(path.start_index), move |q, snapshot, params| {
                            block_json_response(params, &snapshot.anchor_txs(q, path.start_index, BLOCK_TXS_PAGE_SIZE)?)
                        }).await
                    },
                    |op| {
                        op.id("get_block_txs_from_index")
                            .blocks_tag()
                            .summary("Block transactions (paginated)")
                            .description(&format!(
                                "Retrieve transactions in a block by block hash, starting from the specified index. Returns up to {} transactions at a time.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*",
                                BLOCK_TXS_PAGE_SIZE
                            ))
                            .json_response::<Vec<Transaction>>()
                            .not_modified()
                            .bad_request()
                            .not_found()
                            .server_error()
                    },
                ),
            )
            .api_route(
                "/api/blocks",
                get_with(
                    async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                        state.respond_blocks(headers, None).await
                    },
                    |op| {
                        op.id("get_blocks")
                            .blocks_tag()
                            .summary("Recent blocks")
                            .description("Retrieve the last 10 blocks. Returns block metadata for each block.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*")
                            .json_response::<Vec<BlockInfo>>()
                            .not_modified()
                            .bad_request()
                            .server_errors()
                    },
                ),
            )
            .api_route(
                "/api/blocks/{height}",
                get_with(
                    async |headers: HeaderMap,
                           Path(path): Path<HeightParam>,
                           _: Empty, RequestState(state): RequestState| {
                        state.respond_blocks(headers, Some(path.height)).await
                    },
                    |op| {
                        op.id("get_blocks_from_height")
                            .blocks_tag()
                            .summary("Blocks from height")
                            .description(
                                "Retrieve up to 10 blocks going backwards from the given height. For example, height=100 returns blocks 100, 99, 98, ..., 91. Height=0 returns only block 0.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*",
                            )
                            .json_response::<Vec<BlockInfo>>()
                            .not_modified()
                            .bad_request()
                            .server_error()
                    },
                ),
            )
            .api_route(
                "/api/v1/blocks",
                get_with(
                    async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                        state.respond_blocks_v1(headers, None).await
                    },
                    |op| {
                        op.id("get_blocks_v1")
                            .blocks_tag()
                            .summary("Recent blocks with extras")
                            .description("Retrieve the last 15 blocks with extended data including pool identification and fee statistics.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*")
                            .json_response::<Vec<BlockInfoV1>>()
                            .not_modified()
                            .bad_request()
                            .server_errors()
                    },
                ),
            )
            .api_route(
                "/api/v1/blocks/{height}",
                get_with(
                    async |headers: HeaderMap,
                           Path(path): Path<HeightParam>,
                           _: Empty, RequestState(state): RequestState| {
                        state.respond_blocks_v1(headers, Some(path.height)).await
                    },
                    |op| {
                        op.id("get_blocks_v1_from_height")
                            .blocks_tag()
                            .summary("Blocks from height with extras")
                            .description("Retrieve up to 15 blocks with extended data going backwards from the given height.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*")
                            .json_response::<Vec<BlockInfoV1>>()
                            .not_modified()
                            .bad_request()
                            .server_errors()
                    },
                ),
            )
    }
}
