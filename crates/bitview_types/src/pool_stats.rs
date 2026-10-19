use std::borrow::Cow;

use bitview_primitives::{Pool, PoolSlug};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Mining pool with block statistics for a time period
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PoolStats {
    /// One-based pool ID, matching mempool.space's `id`
    #[serde(rename = "poolId")]
    #[schemars(example = 111)]
    pub pool_id: u8,

    #[schemars(example = &"Foundry USA")]
    pub name: Cow<'static, str>,

    /// Pool website URL
    #[schemars(example = &"https://foundrydigital.com/")]
    pub link: Cow<'static, str>,

    #[serde(rename = "blockCount")]
    #[schemars(example = 42)]
    pub block_count: u64,

    /// Pool ranking by block count (1 = most blocks)
    #[schemars(example = 1)]
    pub rank: u32,

    /// Number of empty (coinbase-only) blocks mined in the time period
    #[serde(rename = "emptyBlocks")]
    #[schemars(example = 0)]
    pub empty_blocks: u64,

    /// URL-friendly pool identifier
    pub slug: PoolSlug,

    /// Pool's share of total blocks (0.0 - 1.0)
    #[schemars(example = 0.30)]
    pub share: f64,

    /// Zero-based pool ID, matching mempool.space's `unique_id`
    #[serde(rename = "poolUniqueId")]
    #[schemars(example = 44)]
    pub pool_unique_id: u8,
}

impl PoolStats {
    pub fn new(
        pool: &'static Pool,
        block_count: u64,
        empty_blocks: u64,
        rank: u32,
        share: f64,
    ) -> Self {
        Self {
            pool_id: pool.mempool_id(),
            name: Cow::Borrowed(pool.name),
            link: Cow::Borrowed(pool.link()),
            block_count,
            rank,
            empty_blocks,
            slug: pool.slug,
            share,
            pool_unique_id: pool.mempool_unique_id(),
        }
    }
}
