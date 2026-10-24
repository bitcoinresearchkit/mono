use aide::axum::{ApiRouter, routing::get_with};
use axum::{body::Bytes, extract::Path, http::HeaderMap, response::Response};
use bitcoin::hashes::{Hash, HashEngine, sha256};
use bitview_primitives::PoolSlug;
use bitview_types::{
    BlockFeeRatesEntry, BlockFeeRatesPreciseEntry, BlockFeesEntry, BlockInfoV1, BlockRewardsEntry,
    BlockSizesWeights, DifficultyAdjustmentEntry, HashrateSummary, PoolDetail, PoolHashrateEntry,
    PoolInfo, PoolsSummary, RewardStats,
};
use brk_types::{BlockHash, Dollars, Height};
use serde_json::to_vec;

use super::blocks;
use crate::{
    AppState, CacheParams,
    api::tags,
    extended::TransformResponseExtended,
    params::{BlockCountParam, Empty, PoolSlugAndHeightParam, PoolSlugParam, TimePeriodParam},
    request_state::RequestState,
};
const POOL_BLOCKS_LIMIT: usize = 100;

fn pool_blocks_params(
    anchor: Option<BlockHash>,
    heights: &[Height],
    prices: &[Dollars],
) -> CacheParams {
    // Reuse the V1 row/catalog schema and price identity. Sparse selection also
    // participates: reclassification can change older rows at the same anchor.
    let identity = blocks::blocks_v1_identity(anchor, prices);
    let mut engine = sha256::Hash::engine();
    engine.input(identity.as_byte_array());
    for height in heights {
        engine.input(&u32::from(*height).to_le_bytes());
    }
    let identity = sha256::Hash::from_engine(engine);
    CacheParams::live(format!(
        "pool-blocks-{}-{identity}",
        blocks::V1_BLOCK_SCHEMA
    ))
}

async fn serve_pool_blocks(
    state: AppState,
    headers: HeaderMap,
    slug: PoolSlug,
    before_height: Option<Height>,
) -> Response {
    state
        .respond_read(
            headers,
            "application/json",
            move |q| {
                let blocks = q.resolve_pool_blocks(slug, before_height, POOL_BLOCKS_LIMIT)?;
                let params =
                    pool_blocks_params(blocks.activity_anchor(), blocks.heights(), blocks.prices());
                Ok((blocks, params))
            },
            |q, blocks| Ok(Bytes::from(to_vec(&q.pool_blocks_resolved(blocks)?)?)),
        )
        .await
}

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router.api_route(
        "/api/v1/mining/pools",
        get_with(
            async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                state.mining_pools_body.respond(&headers)
            },
            |op| {
                op.id("get_pools")
                    .tag(tags::MINING)
                    .summary("List all mining pools")
                    .description("*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*")
                    .json_response::<Vec<PoolInfo>>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/v1/mining/pools/{time_period}",
        get_with(
            async |headers: HeaderMap, Path(path): Path<TimePeriodParam>, _: Empty, RequestState(state): RequestState| {
                let time_period = path.time_period;
                state.respond_json_content(&headers, move |q| q.mining_pools(time_period)).await
            },
            |op| {
                op.id("get_pool_stats")
                    .tag(tags::MINING)
                    .summary("Mining pool statistics")
                    .description("Get mining pool statistics for a time period.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*")
                    .json_response::<PoolsSummary>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/v1/mining/pool/{slug}",
        get_with(
            async |headers: HeaderMap, Path(path): Path<PoolSlugParam>, _: Empty, RequestState(state): RequestState| {
                state.respond_json_content(&headers, move |q| q.pool_detail(path.slug)).await
            },
            |op| {
                op.id("get_pool")
                    .tag(tags::MINING)
                    .summary("Mining pool details")
                    .description("Block counts and mining shares by time period for one pool.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool)*")
                    .json_response::<PoolDetail>()
                    .not_modified()
                    .not_found()
            },
        ),
    )
    .api_route(
        "/api/v1/mining/hashrate/pools",
        get_with(
            async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                state.respond_json_content_admitted(headers, |q| q.pools_hashrate(None)).await
            },
            |op| {
                op.id("get_pools_hashrate")
                    .tag(tags::MINING)
                    .summary("All pools hashrate (all time)")
                    .description("Get weekly hashrate data, over the whole chain, of the mining pools that mined in the last week, by time and then their weekly rank.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*")
                    .json_response::<Vec<PoolHashrateEntry>>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/v1/mining/hashrate/pools/{time_period}",
        get_with(
            async |headers: HeaderMap, Path(path): Path<TimePeriodParam>, _: Empty, RequestState(state): RequestState| {
                let time_period = Some(path.time_period);
                state.respond_json_content_admitted(headers, move |q| q.pools_hashrate(time_period)).await
            },
            |op| {
                op.id("get_pools_hashrate_by_period")
                    .tag(tags::MINING)
                    .summary("All pools hashrate")
                    .description("Get weekly hashrate data, for a time period, of the mining pools that mined in the last week, by time and then their weekly rank.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*")
                    .json_response::<Vec<PoolHashrateEntry>>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/v1/mining/pool/{slug}/hashrate",
        get_with(
            async |headers: HeaderMap, Path(path): Path<PoolSlugParam>, _: Empty, RequestState(state): RequestState| {
                let slug = path.slug;
                state.respond_json_content(&headers, move |q| q.pool_hashrate(slug)).await
            },
            |op| {
                op.id("get_pool_hashrate")
                    .tag(tags::MINING)
                    .summary("Mining pool hashrate")
                    .description("Get hashrate history for a specific mining pool.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrate)*")
                    .json_response::<Vec<PoolHashrateEntry>>()
                    .not_modified()
                    .not_found()
            },
        ),
    )
    .api_route(
        "/api/v1/mining/pool/{slug}/blocks",
        get_with(
            async |headers: HeaderMap, Path(path): Path<PoolSlugParam>, _: Empty, RequestState(state): RequestState| {
                serve_pool_blocks(state, headers, path.slug, None).await
            },
            |op| {
                op.id("get_pool_blocks")
                    .tag(tags::MINING)
                    .summary("Mining pool blocks")
                    .description("Get up to 100 recent blocks mined by a specific pool.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*")
                    .json_response::<Vec<BlockInfoV1>>()
                    .not_modified()
                    .not_found()
            },
        ),
    )
    .api_route(
        "/api/v1/mining/pool/{slug}/blocks/{height}",
        get_with(
            async |headers: HeaderMap, Path(PoolSlugAndHeightParam {slug, height}): Path<PoolSlugAndHeightParam>, _: Empty, RequestState(state): RequestState| {
                serve_pool_blocks(state, headers, slug, Some(height)).await
            },
            |op| {
                op.id("get_pool_blocks_from")
                    .tag(tags::MINING)
                    .summary("Mining pool blocks before height")
                    .description("Get up to 100 blocks mined by a specific pool strictly below the given height (pass the last height of a page to get the next one).\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*")
                    .json_response::<Vec<BlockInfoV1>>()
                    .not_modified()
                    .not_found()
            },
        ),
    )
    .api_route(
        "/api/v1/mining/hashrate",
        get_with(
            async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                state.respond_json_content(&headers, |q| q.hashrate(None)).await
            },
            |op| {
                op.id("get_hashrate")
                    .tag(tags::MINING)
                    .summary("Network hashrate (all time)")
                    .description("Get network hashrate and difficulty data for all time.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*")
                    .json_response::<HashrateSummary>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/v1/mining/hashrate/{time_period}",
        get_with(
            async |headers: HeaderMap, Path(path): Path<TimePeriodParam>, _: Empty, RequestState(state): RequestState| {
                let time_period = Some(path.time_period);
                state.respond_json_content(&headers, move |q| q.hashrate(time_period)).await
            },
            |op| {
                op.id("get_hashrate_by_period")
                    .tag(tags::MINING)
                    .summary("Network hashrate")
                    .description("Get network hashrate and difficulty data for a time period.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*")
                    .json_response::<HashrateSummary>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/v1/mining/difficulty-adjustments",
        get_with(
            async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                state.respond_json_content(&headers, |q| q.difficulty_adjustments(None)).await
            },
            |op| {
                op.id("get_difficulty_adjustments")
                    .tag(tags::MINING)
                    .summary("Difficulty adjustments (all time)")
                    .description("*[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*")
                    .json_response::<Vec<DifficultyAdjustmentEntry>>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/v1/mining/difficulty-adjustments/{time_period}",
        get_with(
            async |headers: HeaderMap, Path(path): Path<TimePeriodParam>, _: Empty, RequestState(state): RequestState| {
                let time_period = Some(path.time_period);
                state.respond_json_content(&headers, move |q| q.difficulty_adjustments(time_period)).await
            },
            |op| {
                op.id("get_difficulty_adjustments_by_period")
                    .tag(tags::MINING)
                    .summary("Difficulty adjustments")
                    .description("Get historical difficulty adjustments for a time period.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*")
                    .json_response::<Vec<DifficultyAdjustmentEntry>>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/v1/mining/reward-stats/{block_count}",
        get_with(
            async |headers: HeaderMap, Path(path): Path<BlockCountParam>, _: Empty, RequestState(state): RequestState| {
                state.respond_json_content(&headers, move |q| q.reward_stats(path.block_count)).await
            },
            |op| {
                op.id("get_reward_stats")
                    .tag(tags::MINING)
                    .summary("Mining reward statistics")
                    .description("Get mining reward statistics for the last N blocks including total rewards, fees, and transaction count.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-reward-stats)*")
                    .json_response::<RewardStats>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/v1/mining/blocks/fees/{time_period}",
        get_with(
            async |headers: HeaderMap, Path(path): Path<TimePeriodParam>, _: Empty, RequestState(state): RequestState| {
                let time_period = path.time_period;
                state.respond_json_content(&headers, move |q| q.block_fees(time_period)).await
            },
            |op| {
                op.id("get_block_fees")
                    .tag(tags::MINING)
                    .summary("Block fees")
                    .description("Get average total fees per block for a time period.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-fees)*")
                    .json_response::<Vec<BlockFeesEntry>>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/v1/mining/blocks/rewards/{time_period}",
        get_with(
            async |headers: HeaderMap, Path(path): Path<TimePeriodParam>, _: Empty, RequestState(state): RequestState| {
                let time_period = path.time_period;
                state.respond_json_content(&headers, move |q| q.block_rewards(time_period)).await
            },
            |op| {
                op.id("get_block_rewards")
                    .tag(tags::MINING)
                    .summary("Block rewards")
                    .description("Get average coinbase reward (subsidy + fees) per block for a time period.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-rewards)*")
                    .json_response::<Vec<BlockRewardsEntry>>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/v1/mining/blocks/fee-rates/{time_period}",
        get_with(
            async |headers: HeaderMap, Path(path): Path<TimePeriodParam>, _: Empty, RequestState(state): RequestState| {
                let time_period = path.time_period;
                state.respond_json_content(&headers, move |q| q.block_fee_rates(time_period)).await
            },
            |op| {
                op.id("get_block_fee_rates")
                    .tag(tags::MINING)
                    .summary("Block fee rates")
                    .description("Get block fee rate percentiles (min, 10th, 25th, median, 75th, 90th, max) for a time period, each a whole sat/vB (`/precise` for 3 decimals).\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-feerates)*")
                    .json_response::<Vec<BlockFeeRatesEntry>>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/v1/mining/blocks/fee-rates/{time_period}/precise",
        get_with(
            async |headers: HeaderMap, Path(path): Path<TimePeriodParam>, _: Empty, RequestState(state): RequestState| {
                let time_period = path.time_period;
                state.respond_json_content(&headers, move |q| q.block_fee_rates_precise(time_period)).await
            },
            |op| {
                op.id("get_block_fee_rates_precise")
                    .tag(tags::MINING)
                    .summary("Block fee rates (precise)")
                    .description("Get block fee rate percentiles (min, 10th, 25th, median, 75th, 90th, max) for a time period, each in sat/vB to 3 decimals.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-feerates)*")
                    .json_response::<Vec<BlockFeeRatesPreciseEntry>>()
                    .not_modified()
            },
        ),
    )
    .api_route(
        "/api/v1/mining/blocks/sizes-weights/{time_period}",
        get_with(
            async |headers: HeaderMap, Path(path): Path<TimePeriodParam>, _: Empty, RequestState(state): RequestState| {
                let time_period = path.time_period;
                state.respond_json_content(&headers, move |q| q.block_sizes_weights(time_period)).await
            },
            |op| {
                op.id("get_block_sizes_weights")
                    .tag(tags::MINING)
                    .summary("Block sizes and weights")
                    .description("Get average block sizes and weights for a time period.\n\n*[Mempool.space docs](https://mempool.space/docs/api/rest#get-sizes-weights)*")
                    .json_response::<BlockSizesWeights>()
                    .not_modified()
            },
        ),
    )
}
