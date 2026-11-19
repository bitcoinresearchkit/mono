use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    body::Bytes,
    extract::Path,
    http::{HeaderMap, Method, header},
    response::Response,
};
use bitview_query::{Error as QueryError, Query, ResolvedBlocks, Result as QueryResult};
use bitview_types::{BlockInfo, BlockInfoV1, BlockStatus, BlockTimestamp, BlockTxIndex};
use brk_types::{BlockHash, Height, Hex, Transaction, Txid};

use crate::{
    AppState,
    answer::{Answer, Content, Settle},
    api::tags,
    extended::TransformResponseExtended,
    gates::Gate,
    params::{
        BlockHashParam, BlockHashStartIndex, BlockHashTxIndex, Empty, HeightParam, TimestampParam,
    },
    request_state::RequestState,
};

const BLOCK_TXS_PAGE_SIZE: u32 = 25;

/// One way a block is sent by its hash: the name its tag carries, bumped when its format
/// changes, and its kind once settled. The hash commits to every byte of a header, of the raw
/// block and of a txid; the JSON forms are the server's.
#[derive(Clone, Copy)]
struct BlockForm {
    name: &'static str,
    settle: Settle,
}

impl BlockForm {
    const JSON: Self = Self::new("v3", Answer::settled);
    const HEADER: Self = Self::new("header-v2", Answer::committed);
    const RAW: Self = Self::new("raw-v2", Answer::committed);
    const TXID: Self = Self::new("txid-v2", Answer::committed);
    const TXIDS: Self = Self::new("txids-v2", Answer::settled);
    const TXS: Self = Self::new("txs-v2", Answer::settled);

    const fn new(name: &'static str, settle: Settle) -> Self {
        Self { name, settle }
    }
}

/// Covers the extended rows' formulas and bundled pool catalog: bumped when either changes.
pub const V1_BLOCK_SCHEMA: &str = "v1-7";

const MISSING_BLOCK: QueryError = QueryError::Internal("Missing resolved block");

#[derive(Clone, Copy)]
enum BlockTxSelection {
    Index(BlockTxIndex),
    Page(BlockTxIndex),
}

impl BlockTxSelection {
    fn validate(self, q: &Query, snapshot: &ResolvedBlocks) -> QueryResult<()> {
        let index = match self {
            Self::Index(index) | Self::Page(index) => index,
        };
        snapshot.validate_tx_index(q, index)?;
        // Keep range errors first, as on the ordinary Esplora response.
        if matches!(self, Self::Page(_)) && u32::from(index) % BLOCK_TXS_PAGE_SIZE != 0 {
            return Err(QueryError::InvalidParam(format!(
                "start index must be a multiple of {BLOCK_TXS_PAGE_SIZE}"
            )));
        }
        Ok(())
    }
}

impl AppState {
    async fn respond_block_tip(&self, headers: &HeaderMap, hash: bool) -> Response {
        self.respond(headers, self.chain(), move |q| {
            let snapshot = q.resolve_blocks(None, 1)?;
            // An empty index has no tip yet.
            let value = if hash {
                snapshot.anchor().map(|hash| hash.to_string())
            } else {
                snapshot.last_height().map(|height| height.to_string())
            }
            .ok_or_else(|| QueryError::updating())?;
            // The value is the whole representation. In particular, a same-height reorg
            // changes the hash endpoint but not the height.
            Ok(Answer::new(format!("block-tip-v2-{value}"), move |_| {
                Ok(Content::text(value))
            }))
        })
        .await
    }

    /// An answer about the block at `hash`, which `tx_selection` must fit: named by the block
    /// (its height and hash under the form's name) before `build` reads any of it.
    async fn respond_block(
        &self,
        headers: &HeaderMap,
        gate: Gate<'_>,
        hash: BlockHash,
        form: BlockForm,
        tx_selection: Option<BlockTxSelection>,
        build: impl FnOnce(&Query, ResolvedBlocks) -> QueryResult<Content> + Clone + Send + 'static,
    ) -> Response {
        self.respond(headers, gate, move |q| {
            let snapshot = q.resolve_block_snapshot(&hash)?;
            if let Some(selection) = tx_selection {
                selection.validate(q, &snapshot)?;
            }
            let height = snapshot.last_height().ok_or(MISSING_BLOCK)?;
            let hash = snapshot.anchor().ok_or(MISSING_BLOCK)?;
            let settled = snapshot.is_settled();
            let tag = format!("block-{}-{height}-{hash}", form.name);
            let answer = Answer::new(tag, move |q| build(q, snapshot));
            Ok((form.settle)(answer, settled))
        })
        .await
    }

    async fn respond_block_raw(
        &self,
        headers: &HeaderMap,
        hash: BlockHash,
        head: bool,
    ) -> Response {
        const RAW: &str = "application/octet-stream";
        let gate = self.raw_block();
        // A `HEAD` reads the block's length alone: it keeps no body in memory.
        let gate = if head { gate.without_bodies() } else { gate };
        self.respond_block(headers, gate, hash, BlockForm::RAW, None, move |q, block| {
            Ok(if head {
                let length = block.anchor_raw_size(q)?;
                Content::new(RAW, Bytes::new()).with(header::CONTENT_LENGTH, length.into())
            } else {
                Content::new(RAW, block.anchor_raw(q)?)
            })
        })
        .await
    }

    /// A page of the block's transactions from `start`.
    async fn respond_block_txs(
        &self,
        headers: &HeaderMap,
        hash: BlockHash,
        start: BlockTxIndex,
    ) -> Response {
        self.respond_block(
            headers,
            self.chain(),
            hash,
            BlockForm::TXS,
            Some(BlockTxSelection::Page(start)),
            move |q, block| Content::json(&block.anchor_txs(q, start, BLOCK_TXS_PAGE_SIZE)?),
        )
        .await
    }

    async fn respond_block_v1(&self, headers: &HeaderMap, hash: BlockHash) -> Response {
        self.respond(headers, self.chain(), move |q| {
            let snapshot = q.resolve_block_v1(&hash)?;
            let height = snapshot.last_height().ok_or(MISSING_BLOCK)?;
            let tag = format!(
                "block-{V1_BLOCK_SCHEMA}-{height}-{:x}",
                snapshot.identity()
            );
            let settled = snapshot.is_settled();
            Ok(
                Answer::json(tag, move |q| snapshot.build(q)?.pop().ok_or(MISSING_BLOCK))
                    .settled(settled),
            )
        })
        .await
    }

    async fn respond_blocks_v1(
        &self,
        headers: &HeaderMap,
        start_height: Option<Height>,
    ) -> Response {
        self.respond(headers, self.chain(), move |q| {
            let snapshot = q.resolve_blocks_v1(start_height, 15)?;
            let tag = format!("blocks-{V1_BLOCK_SCHEMA}-{:x}", snapshot.identity());
            let settled = snapshot.is_settled();
            Ok(Answer::json(tag, move |q| snapshot.build(q)).settled(settled))
        })
        .await
    }

    async fn respond_blocks(&self, headers: &HeaderMap, start_height: Option<Height>) -> Response {
        self.respond(headers, self.chain(), move |q| {
            let snapshot = q.resolve_blocks(start_height, 10)?;
            let tag = match snapshot.anchor() {
                Some(hash) => format!("blocks2-{hash}"),
                None => "blocks2-empty".to_owned(),
            };
            let settled = snapshot.is_settled();
            Ok(Answer::json(tag, move |q| snapshot.build(q)).settled(settled))
        })
        .await
    }
}

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router.api_route(
            "/api/block/{hash}",
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<BlockHashParam>,
                       _: Empty,
                       RequestState(state): RequestState| {
                    state
                        .respond_block(
                            &headers,
                            state.chain(),
                            path.hash,
                            BlockForm::JSON,
                            None,
                            |q, block| Content::json(&block.build(q)?.pop().ok_or(MISSING_BLOCK)?),
                        )
                        .await
                },
                |op| {
                    op.id("get_block")
                        .tag(tags::BLOCKS)
                        .summary("Block information")
                        .description(
                            "Details of a block by hash: height, time, transaction count, size, weight, and difficulty.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block)*",
                        )
                        .json_response::<BlockInfo>()
                        .not_modified()
                        .not_found()
                },
            ),
        )
        .api_route(
            "/api/v1/block/{hash}",
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<BlockHashParam>,
                       _: Empty,
                       RequestState(state): RequestState| {
                    state.respond_block_v1(&headers, path.hash).await
                },
                |op| {
                    op.id("get_block_v1")
                        .tag(tags::BLOCKS)
                        .summary("Block (v1)")
                        .description("Returns block details with extras by hash.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-v1)*")
                        .json_response::<BlockInfoV1>()
                        .not_modified()
                        .not_found()
                },
            ),
        )
        .api_route(
            "/api/block/{hash}/header",
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<BlockHashParam>,
                       _: Empty,
                       RequestState(state): RequestState| {
                    state
                        .respond_block(
                            &headers,
                            state.chain(),
                            path.hash,
                            BlockForm::HEADER,
                            None,
                            |q, block| Ok(Content::text(block.anchor_header_hex(q)?)),
                        )
                        .await
                },
                |op| {
                    op.id("get_block_header")
                        .tag(tags::BLOCKS)
                        .summary("Block header")
                        .description("Returns the hex-encoded 80-byte block header.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-header)*")
                        .text_response::<Hex>()
                        .not_modified()
                        .not_found()
                },
            ),
        )
        .api_route(
            "/api/block-height/{height}",
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<HeightParam>,
                       _: Empty, RequestState(state): RequestState| {
                    // The hash is the whole representation.
                    state
                        .respond(&headers, state.chain(), move |q| {
                            let (hash, settled) = q.resolve_block_hash(path.height)?;
                            let tag = format!("block-height-v2-{hash}");
                            let content = Content::text(hash.to_string());
                            Ok(Answer::ready(tag, content).settled(settled))
                        })
                        .await
                },
                |op| {
                    op.id("get_block_hash_by_height")
                        .tag(tags::BLOCKS)
                        .summary("Block hash by height")
                        .description(
                            "Block hash as plain text.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-height)*",
                        )
                        .text_response::<BlockHash>()
                        .not_modified()
                        .not_found()
                },
            ),
        )
        .api_route(
            "/api/v1/mining/blocks/timestamp/{timestamp}",
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<TimestampParam>,
                       _: Empty, RequestState(state): RequestState| {
                    state
                        .respond(&headers, state.chain(), move |q| {
                            let block = q.resolve_block_by_timestamp(path.timestamp)?;
                            let settled = block.is_settled();
                            let tag = format!("block-timestamp-v2-{}", block.hash());
                            Ok(Answer::json(tag, move |_| Ok(block.into_value())).settled(settled))
                        })
                        .await
                },
                |op| {
                    op.id("get_block_by_timestamp")
                        .tag(tags::BLOCKS)
                        .summary("Block by timestamp")
                        .description("Find the block with the greatest header timestamp at or before the given UNIX timestamp, choosing the earliest height on ties.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-timestamp)*")
                        .json_response::<BlockTimestamp>()
                        .not_modified()
                        .not_found()
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
                       RequestState(state): RequestState| {
                    state
                        .respond_block_raw(&headers, path.hash, method == Method::HEAD)
                        .await
                },
                |op| {
                    op.id("get_block_raw")
                        .tag(tags::BLOCKS)
                        .mcp_ignore()
                        .summary("Raw block")
                        .description(
                            "Returns the raw block data in binary format.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-raw)*",
                        )
                        .binary_response()
                        .not_modified()
                        .not_found()
                },
            ),
        )
        .api_route(
            "/api/block/{hash}/status",
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<BlockHashParam>,
                       _: Empty,
                       RequestState(state): RequestState| {
                    state
                        .respond(&headers, state.chain(), move |q| {
                            Ok(Answer::built(Content::json(&q.block_status(&path.hash)?)?))
                        })
                        .await
                },
                |op| {
                    op.id("get_block_status")
                        .tag(tags::BLOCKS)
                        .summary("Block status")
                        .description(
                            "Whether the block is in the best chain and, if so, its height and the hash of the next block. A block outside the best chain (orphaned or unknown) reports `in_best_chain: false`.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-status)*",
                        )
                        .json_response::<BlockStatus>()
                        .not_modified()
                },
            ),
        )
        .api_route(
            "/api/blocks/tip/height",
            get_with(
                async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                    state.respond_block_tip(&headers, false).await
                },
                |op| {
                    op.id("get_block_tip_height")
                        .tag(tags::BLOCKS)
                        .summary("Block tip height")
                        .description("Height of the latest block.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-height)*")
                        .text_response::<Height>()
                        .not_modified()
                },
            ),
        )
        .api_route(
            "/api/blocks/tip/hash",
            get_with(
                async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                    state.respond_block_tip(&headers, true).await
                },
                |op| {
                    op.id("get_block_tip_hash")
                        .tag(tags::BLOCKS)
                        .summary("Block tip hash")
                        .description("Hash of the latest block.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-hash)*")
                        .text_response::<BlockHash>()
                        .not_modified()
                },
            ),
        )
        .api_route(
            "/api/block/{hash}/txid/{index}",
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<BlockHashTxIndex>,
                       _: Empty,
                       RequestState(state): RequestState| {
                    state
                        .respond_block(
                            &headers,
                            state.chain(),
                            path.hash,
                            BlockForm::TXID,
                            Some(BlockTxSelection::Index(path.index)),
                            move |q, block| {
                                Ok(Content::text(block.anchor_txid(q, path.index)?.to_string()))
                            },
                        )
                        .await
                },
                |op| {
                    op.id("get_block_txid")
                        .tag(tags::BLOCKS)
                        .summary("Transaction ID at index")
                        .description(
                            "Retrieve a single transaction ID at a specific index within a block. Returns plain text txid.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transaction-id)*",
                        )
                        .text_response::<Txid>()
                        .not_modified()
                        .not_found()
                },
            ),
        )
        .api_route(
            "/api/block/{hash}/txids",
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<BlockHashParam>,
                       _: Empty,
                       RequestState(state): RequestState| {
                    state
                        .respond_block(
                            &headers,
                            state.chain(),
                            path.hash,
                            BlockForm::TXIDS,
                            None,
                            |q, block| Content::json(&block.anchor_txids(q)?),
                        )
                        .await
                },
                |op| {
                    op.id("get_block_txids")
                        .tag(tags::BLOCKS)
                        .mcp_ignore()
                        .summary("Block transaction IDs")
                        .description(
                            "Transaction IDs in block order.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transaction-ids)*",
                        )
                        .json_response::<Vec<Txid>>()
                        .not_modified()
                        .not_found()
                },
            ),
        )
        .api_route(
            "/api/block/{hash}/txs",
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<BlockHashParam>,
                       _: Empty,
                       RequestState(state): RequestState| {
                    state
                        .respond_block_txs(&headers, path.hash, BlockTxIndex::default())
                        .await
                },
                |op| {
                    op.id("get_block_txs")
                        .tag(tags::BLOCKS)
                        .mcp_ignore()
                        .summary("Block transactions")
                        .description(&format!(
                            "Up to {} transactions starting at index 0.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*",
                            BLOCK_TXS_PAGE_SIZE
                        ))
                        .json_response::<Vec<Transaction>>()
                        .not_modified()
                        .not_found()
                },
            ),
        )
        .api_route(
            "/api/block/{hash}/txs/{start_index}",
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<BlockHashStartIndex>,
                       _: Empty,
                       RequestState(state): RequestState| {
                    state
                        .respond_block_txs(&headers, path.hash, path.start_index)
                        .await
                },
                |op| {
                    op.id("get_block_txs_from_index")
                        .tag(tags::BLOCKS)
                        .summary("Block transactions (paginated)")
                        .description(&format!(
                            "Up to {} transactions starting at `start_index`, a multiple of {}.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*",
                            BLOCK_TXS_PAGE_SIZE, BLOCK_TXS_PAGE_SIZE
                        ))
                        .json_response::<Vec<Transaction>>()
                        .not_modified()
                        .not_found()
                },
            ),
        )
        .api_route(
            "/api/blocks",
            get_with(
                async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                    state.respond_blocks(&headers, None).await
                },
                |op| {
                    op.id("get_blocks")
                        .tag(tags::BLOCKS)
                        .summary("Recent blocks")
                        .description("The last 10 blocks.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*")
                        .json_response::<Vec<BlockInfo>>()
                        .not_modified()
                },
            ),
        )
        .api_route(
            "/api/blocks/{height}",
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<HeightParam>,
                       _: Empty, RequestState(state): RequestState| {
                    state.respond_blocks(&headers, Some(path.height)).await
                },
                |op| {
                    op.id("get_blocks_from_height")
                        .tag(tags::BLOCKS)
                        .summary("Blocks from height")
                        .description(
                            "Retrieve up to 10 blocks going backwards from the given height. For example, height=100 returns blocks 100, 99, 98, ..., 91. Height=0 returns only block 0. A height past the tip is not found.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*",
                        )
                        .json_response::<Vec<BlockInfo>>()
                        .not_modified()
                        .not_found()
                },
            ),
        )
        .api_route(
            "/api/v1/blocks",
            get_with(
                async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                    state.respond_blocks_v1(&headers, None).await
                },
                |op| {
                    op.id("get_blocks_v1")
                        .tag(tags::BLOCKS)
                        .summary("Recent blocks with extras")
                        .description("Retrieve the last 15 blocks with extended data including pool identification and fee statistics.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*")
                        .json_response::<Vec<BlockInfoV1>>()
                        .not_modified()
                },
            ),
        )
        .api_route(
            "/api/v1/blocks/{height}",
            get_with(
                async |headers: HeaderMap,
                       Path(path): Path<HeightParam>,
                       _: Empty, RequestState(state): RequestState| {
                    state.respond_blocks_v1(&headers, Some(path.height)).await
                },
                |op| {
                    op.id("get_blocks_v1_from_height")
                        .tag(tags::BLOCKS)
                        .summary("Blocks from height with extras")
                        .description("Retrieve up to 15 blocks with extended data going backwards from the given height: the published blocks among the 15 heights ending at it, so fewer (or none) past the tip.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*")
                        .json_response::<Vec<BlockInfoV1>>()
                        .not_modified()
                },
            ),
        )
}
