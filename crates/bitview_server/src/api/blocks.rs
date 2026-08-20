use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
};
use brk_types::{
    BlockHash, BlockInfo, BlockInfoV1, BlockStatus, BlockTimestamp, BlockTxIndex, Height, Hex,
    Transaction, Txid, Version,
};

use crate::{
    AppState, CacheStrategy,
    extended::TransformResponseExtended,
    params::{
        BlockHashParam, BlockHashStartIndex, BlockHashTxIndex, Empty, HeightParam, TimestampParam,
    },
};

const BLOCK_TXS_PAGE_SIZE: u32 = 25;

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
                           _: Empty, State(state): State<AppState>| {
                        let strategy = state.block_strategy(Version::ONE, &path.hash);
                        state.respond_json(&headers, strategy, move |q| q.block(&path.hash)).await
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
                            .server_error()
                    },
                ),
            )
            .api_route(
                "/api/v1/block/{hash}",
                get_with(
                    async |headers: HeaderMap, Path(path): Path<BlockHashParam>, _: Empty, State(state): State<AppState>| {
                        let strategy = state.block_strategy(Version::ONE, &path.hash);
                        state.respond_json(&headers, strategy, move |q| {
                            let height = q.height_by_hash(&path.hash)?;
                            q.block_by_height_v1(height)
                        }).await
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
                            .server_error()
                    },
                ),
            )
            .api_route(
                "/api/block/{hash}/header",
                get_with(
                    async |headers: HeaderMap, Path(path): Path<BlockHashParam>, _: Empty, State(state): State<AppState>| {
                        let strategy = state.block_strategy(Version::ONE, &path.hash);
                        state.respond_text(&headers, strategy, move |q| q.block_header_hex(&path.hash)).await
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
                            .server_error()
                    },
                ),
            )
            .api_route(
                "/api/block-height/{height}",
                get_with(
                    async |headers: HeaderMap,
                           Path(path): Path<HeightParam>,
                           _: Empty, State(state): State<AppState>| {
                        state.respond_text(&headers, state.height_strategy(Version::ONE, path.height), move |q| q.block_hash_by_height(path.height).map(|h| h.to_string())).await
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
                            .server_error()
                    },
                ),
            )
            .api_route(
                "/api/v1/mining/blocks/timestamp/{timestamp}",
                get_with(
                    async |headers: HeaderMap,
                           Path(path): Path<TimestampParam>,
                           _: Empty, State(state): State<AppState>| {
                        state.respond_json(&headers, state.timestamp_strategy(Version::ONE, path.timestamp), move |q| q.block_by_timestamp(path.timestamp)).await
                    },
                    |op| {
                        op.id("get_block_by_timestamp")
                            .blocks_tag()
                            .summary("Block by timestamp")
                            .description("Find the block closest to a given UNIX timestamp.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-timestamp)*")
                            .json_response::<BlockTimestamp>()
                            .not_modified()
                            .bad_request()
                            .not_found()
                            .server_error()
                    },
                ),
            )
            .api_route(
                "/api/block/{hash}/raw",
                get_with(
                    async |headers: HeaderMap,
                           Path(path): Path<BlockHashParam>,
                           _: Empty, State(state): State<AppState>| {
                        let strategy = state.block_strategy(Version::ONE, &path.hash);
                        state.respond_bytes(&headers, strategy, move |q| q.block_raw(&path.hash)).await
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
                            .server_error()
                    },
                ),
            )
            .api_route(
                "/api/block/{hash}/status",
                get_with(
                    async |headers: HeaderMap,
                           Path(path): Path<BlockHashParam>,
                           _: Empty, State(state): State<AppState>| {
                        state.respond_json(&headers, state.block_status_strategy(Version::ONE, &path.hash), move |q| q.block_status(&path.hash)).await
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
                    async |headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                        state.respond_text(&headers, CacheStrategy::Tip, |q| Ok(q.height().to_string())).await
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
                    async |headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                        state.respond_text(&headers, CacheStrategy::Tip, |q| Ok(q.tip_blockhash().to_string())).await
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
                           _: Empty, State(state): State<AppState>| {
                        let strategy = state.block_strategy(Version::ONE, &path.hash);
                        state.respond_text(&headers, strategy, move |q| q.block_txid_at_index(&path.hash, path.index).map(|t| t.to_string())).await
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
                           _: Empty, State(state): State<AppState>| {
                        let strategy = state.block_strategy(Version::ONE, &path.hash);
                        state.respond_json(&headers, strategy, move |q| q.block_txids(&path.hash)).await
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
                           _: Empty, State(state): State<AppState>| {
                        let strategy = state.block_strategy(Version::ONE, &path.hash);
                        state.respond_json(&headers, strategy, move |q| q.block_txs(&path.hash, BlockTxIndex::default(), BLOCK_TXS_PAGE_SIZE)).await
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
                           _: Empty, State(state): State<AppState>| {
                        let strategy = state.block_strategy(Version::ONE, &path.hash);
                        state.respond_json(&headers, strategy, move |q| q.block_txs(&path.hash, path.start_index, BLOCK_TXS_PAGE_SIZE)).await
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
                    async |headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                        state
                            .respond_json(&headers, CacheStrategy::Tip, move |q| q.blocks(None, 10))
                            .await
                    },
                    |op| {
                        op.id("get_blocks")
                            .blocks_tag()
                            .summary("Recent blocks")
                            .description("Retrieve the last 10 blocks. Returns block metadata for each block.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*")
                            .json_response::<Vec<BlockInfo>>()
                            .not_modified()
                            .server_error()
                    },
                ),
            )
            .api_route(
                "/api/blocks/{height}",
                get_with(
                    async |headers: HeaderMap,
                           Path(path): Path<HeightParam>,
                           _: Empty, State(state): State<AppState>| {
                        state.respond_json(&headers, state.height_strategy(Version::ONE, path.height), move |q| q.blocks(Some(path.height), 10)).await
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
                    async |headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                        state
                            .respond_json(&headers, CacheStrategy::Tip, move |q| q.blocks_v1(None, 15))
                            .await
                    },
                    |op| {
                        op.id("get_blocks_v1")
                            .blocks_tag()
                            .summary("Recent blocks with extras")
                            .description("Retrieve the last 15 blocks with extended data including pool identification and fee statistics.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*")
                            .json_response::<Vec<BlockInfoV1>>()
                            .not_modified()
                            .server_error()
                    },
                ),
            )
            .api_route(
                "/api/v1/blocks/{height}",
                get_with(
                    async |headers: HeaderMap,
                           Path(path): Path<HeightParam>,
                           _: Empty, State(state): State<AppState>| {
                        state.respond_json(&headers, state.height_strategy(Version::ONE, path.height), move |q| q.blocks_v1(Some(path.height), 15)).await
                    },
                    |op| {
                        op.id("get_blocks_v1_from_height")
                            .blocks_tag()
                            .summary("Blocks from height with extras")
                            .description("Retrieve up to 15 blocks with extended data going backwards from the given height.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*")
                            .json_response::<Vec<BlockInfoV1>>()
                            .not_modified()
                            .bad_request()
                            .server_error()
                    },
                ),
            )
    }
}
