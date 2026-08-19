use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, Uri},
};
use brk_types::{
    BlockFeeRatesEntry, BlockFeesEntry, BlockInfoV1, BlockRewardsEntry, BlockSizesWeights,
    DifficultyAdjustmentEntry, HashrateSummary, PoolDetail, PoolHashrateEntry, PoolInfo,
    PoolsSummary, RewardStats, Version,
};

use crate::{
    AppState, CacheStrategy,
    extended::TransformResponseExtended,
    params::{BlockCountParam, Empty, PoolSlugAndHeightParam, PoolSlugParam, TimePeriodParam},
};

const HASHRATE_MAX_POINTS: usize = 200;
const POOL_BLOCKS_LIMIT: usize = 100;

pub trait MiningRoutes {
    fn add_mining_routes(self) -> Self;
}

impl MiningRoutes for ApiRouter<AppState> {
    fn add_mining_routes(self) -> Self {
        self.api_route(
            "/api/v1/mining/pools",
            get_with(
                async |uri: Uri, headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    // Pool list is compiled-in, only changes on deploy
                    state.respond_json(&headers, CacheStrategy::Deploy, &uri, |q| Ok(q.all_pools())).await
                },
                |op| {
                    op.id("get_pools")
                        .mining_tag()
                        .summary("List all mining pools")
                        .description("Get list of all known mining pools with their identifiers.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*")
                        .json_response::<Vec<PoolInfo>>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/mining/pools/{time_period}",
            get_with(
                async |uri: Uri, headers: HeaderMap, Path(path): Path<TimePeriodParam>, _: Empty, State(state): State<AppState>| {
                    state.respond_json(&headers, CacheStrategy::Tip, &uri, move |q| q.mining_pools(path.time_period)).await
                },
                |op| {
                    op.id("get_pool_stats")
                        .mining_tag()
                        .summary("Mining pool statistics")
                        .description("Get mining pool statistics for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*")
                        .json_response::<PoolsSummary>()
                        .not_modified()
                        .bad_request()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/mining/pool/{slug}",
            get_with(
                async |uri: Uri, headers: HeaderMap, Path(path): Path<PoolSlugParam>, _: Empty, State(state): State<AppState>| {
                    state.respond_json(&headers, CacheStrategy::Tip, &uri, move |q| q.pool_detail(path.slug)).await
                },
                |op| {
                    op.id("get_pool")
                        .mining_tag()
                        .summary("Mining pool details")
                        .description("Get detailed information about a specific mining pool including block counts and shares for different time periods.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool)*")
                        .json_response::<PoolDetail>()
                        .not_modified()
                        .not_found()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/mining/hashrate/pools",
            get_with(
                async |uri: Uri, headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state.respond_json(&headers, CacheStrategy::Tip, &uri, |q| q.pools_hashrate(None)).await
                },
                |op| {
                    op.id("get_pools_hashrate")
                        .mining_tag()
                        .summary("All pools hashrate (all time)")
                        .description("Get hashrate data for all mining pools.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*")
                        .json_response::<Vec<PoolHashrateEntry>>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/mining/hashrate/pools/{time_period}",
            get_with(
                async |uri: Uri, headers: HeaderMap, Path(path): Path<TimePeriodParam>, _: Empty, State(state): State<AppState>| {
                    state.respond_json(&headers, CacheStrategy::Tip, &uri, move |q| q.pools_hashrate(Some(path.time_period))).await
                },
                |op| {
                    op.id("get_pools_hashrate_by_period")
                        .mining_tag()
                        .summary("All pools hashrate")
                        .description("Get hashrate data for all mining pools for a time period. Valid periods: `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*")
                        .json_response::<Vec<PoolHashrateEntry>>()
                        .not_modified()
                        .bad_request()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/mining/pool/{slug}/hashrate",
            get_with(
                async |uri: Uri, headers: HeaderMap, Path(path): Path<PoolSlugParam>, _: Empty, State(state): State<AppState>| {
                    state.respond_json(&headers, CacheStrategy::Tip, &uri, move |q| q.pool_hashrate(path.slug)).await
                },
                |op| {
                    op.id("get_pool_hashrate")
                        .mining_tag()
                        .summary("Mining pool hashrate")
                        .description("Get hashrate history for a specific mining pool.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrate)*")
                        .json_response::<Vec<PoolHashrateEntry>>()
                        .not_modified()
                        .not_found()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/mining/pool/{slug}/blocks",
            get_with(
                async |uri: Uri, headers: HeaderMap, Path(path): Path<PoolSlugParam>, _: Empty, State(state): State<AppState>| {
                    let strategy = state.pool_blocks_strategy(Version::ONE, path.slug);
                    state.respond_json(&headers, strategy, &uri, move |q| q.pool_blocks(path.slug, None, POOL_BLOCKS_LIMIT)).await
                },
                |op| {
                    op.id("get_pool_blocks")
                        .mining_tag()
                        .summary("Mining pool blocks")
                        .description("Get the 10 most recent blocks mined by a specific pool.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*")
                        .json_response::<Vec<BlockInfoV1>>()
                        .not_modified()
                        .not_found()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/mining/pool/{slug}/blocks/{height}",
            get_with(
                async |uri: Uri, headers: HeaderMap, Path(PoolSlugAndHeightParam {slug, height}): Path<PoolSlugAndHeightParam>, _: Empty, State(state): State<AppState>| {
                    state.respond_json(&headers, state.height_strategy(Version::ONE, height), &uri, move |q| q.pool_blocks(slug, Some(height), POOL_BLOCKS_LIMIT)).await
                },
                |op| {
                    op.id("get_pool_blocks_from")
                        .mining_tag()
                        .summary("Mining pool blocks from height")
                        .description("Get 10 blocks mined by a specific pool before (and including) the given height.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*")
                        .json_response::<Vec<BlockInfoV1>>()
                        .not_modified()
                        .not_found()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/mining/hashrate",
            get_with(
                async |uri: Uri, headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state.respond_json(&headers, CacheStrategy::Tip, &uri, |q| q.hashrate(None, HASHRATE_MAX_POINTS)).await
                },
                |op| {
                    op.id("get_hashrate")
                        .mining_tag()
                        .summary("Network hashrate (all time)")
                        .description("Get network hashrate and difficulty data for all time.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*")
                        .json_response::<HashrateSummary>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/mining/hashrate/{time_period}",
            get_with(
                async |uri: Uri, headers: HeaderMap, Path(path): Path<TimePeriodParam>, _: Empty, State(state): State<AppState>| {
                    state.respond_json(&headers, CacheStrategy::Tip, &uri, move |q| q.hashrate(Some(path.time_period), HASHRATE_MAX_POINTS)).await
                },
                |op| {
                    op.id("get_hashrate_by_period")
                        .mining_tag()
                        .summary("Network hashrate")
                        .description("Get network hashrate and difficulty data for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*")
                        .json_response::<HashrateSummary>()
                        .not_modified()
                        .bad_request()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/mining/difficulty-adjustments",
            get_with(
                async |uri: Uri, headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state.respond_json(&headers, CacheStrategy::Tip, &uri, |q| q.difficulty_adjustments(None)).await
                },
                |op| {
                    op.id("get_difficulty_adjustments")
                        .mining_tag()
                        .summary("Difficulty adjustments (all time)")
                        .description("Get historical difficulty adjustments including timestamp, block height, difficulty value, and percentage change.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*")
                        .json_response::<Vec<DifficultyAdjustmentEntry>>()
                        .not_modified()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/mining/difficulty-adjustments/{time_period}",
            get_with(
                async |uri: Uri, headers: HeaderMap, Path(path): Path<TimePeriodParam>, _: Empty, State(state): State<AppState>| {
                    state.respond_json(&headers, CacheStrategy::Tip, &uri, move |q| q.difficulty_adjustments(Some(path.time_period))).await
                },
                |op| {
                    op.id("get_difficulty_adjustments_by_period")
                        .mining_tag()
                        .summary("Difficulty adjustments")
                        .description("Get historical difficulty adjustments for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*")
                        .json_response::<Vec<DifficultyAdjustmentEntry>>()
                        .not_modified()
                        .bad_request()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/mining/reward-stats/{block_count}",
            get_with(
                async |uri: Uri, headers: HeaderMap, Path(path): Path<BlockCountParam>, _: Empty, State(state): State<AppState>| {
                    state.respond_json(&headers, CacheStrategy::Tip, &uri, move |q| q.reward_stats(path.block_count)).await
                },
                |op| {
                    op.id("get_reward_stats")
                        .mining_tag()
                        .summary("Mining reward statistics")
                        .description("Get mining reward statistics for the last N blocks including total rewards, fees, and transaction count.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-reward-stats)*")
                        .json_response::<RewardStats>()
                        .not_modified()
                        .bad_request()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/mining/blocks/fees/{time_period}",
            get_with(
                async |uri: Uri, headers: HeaderMap, Path(path): Path<TimePeriodParam>, _: Empty, State(state): State<AppState>| {
                    state.respond_json(&headers, CacheStrategy::Tip, &uri, move |q| q.block_fees(path.time_period)).await
                },
                |op| {
                    op.id("get_block_fees")
                        .mining_tag()
                        .summary("Block fees")
                        .description("Get average total fees per block for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-fees)*")
                        .json_response::<Vec<BlockFeesEntry>>()
                        .not_modified()
                        .bad_request()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/mining/blocks/rewards/{time_period}",
            get_with(
                async |uri: Uri, headers: HeaderMap, Path(path): Path<TimePeriodParam>, _: Empty, State(state): State<AppState>| {
                    state.respond_json(&headers, CacheStrategy::Tip, &uri, move |q| q.block_rewards(path.time_period)).await
                },
                |op| {
                    op.id("get_block_rewards")
                        .mining_tag()
                        .summary("Block rewards")
                        .description("Get average coinbase reward (subsidy + fees) per block for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-rewards)*")
                        .json_response::<Vec<BlockRewardsEntry>>()
                        .not_modified()
                        .bad_request()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/mining/blocks/fee-rates/{time_period}",
            get_with(
                async |uri: Uri, headers: HeaderMap, Path(path): Path<TimePeriodParam>, _: Empty, State(state): State<AppState>| {
                    state.respond_json(&headers, CacheStrategy::Tip, &uri, move |q| q.block_fee_rates(path.time_period)).await
                },
                |op| {
                    op.id("get_block_fee_rates")
                        .mining_tag()
                        .summary("Block fee rates")
                        .description("Get block fee rate percentiles (min, 10th, 25th, median, 75th, 90th, max) for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-feerates)*")
                        .json_response::<Vec<BlockFeeRatesEntry>>()
                        .not_modified()
                        .bad_request()
                        .server_error()
                },
            ),
        )
        .api_route(
            "/api/v1/mining/blocks/sizes-weights/{time_period}",
            get_with(
                async |uri: Uri, headers: HeaderMap, Path(path): Path<TimePeriodParam>, _: Empty, State(state): State<AppState>| {
                    state.respond_json(&headers, CacheStrategy::Tip, &uri, move |q| q.block_sizes_weights(path.time_period)).await
                },
                |op| {
                    op.id("get_block_sizes_weights")
                        .mining_tag()
                        .summary("Block sizes and weights")
                        .description("Get average block sizes and weights for a time period. Valid periods: `24h`, `3d`, `1w`, `1m`, `3m`, `6m`, `1y`, `2y`, `3y`.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-sizes-weights)*")
                        .json_response::<BlockSizesWeights>()
                        .not_modified()
                        .bad_request()
                        .server_error()
                },
            ),
        )
    }
}
