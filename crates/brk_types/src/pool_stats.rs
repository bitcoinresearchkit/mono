use std::borrow::Cow;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Pool, PoolSlug};

/// Mining pool with block statistics for a time period
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PoolStats {
    /// Unique pool identifier
    #[serde(rename = "poolId")]
    #[schemars(example = 111)]
    pool_id: u8,

    /// Pool name
    #[schemars(example = &"Foundry USA")]
    name: Cow<'static, str>,

    /// Pool website URL
    #[schemars(example = &"https://foundrydigital.com/")]
    link: Cow<'static, str>,

    /// Number of blocks mined in the time period
    #[serde(rename = "blockCount")]
    #[schemars(example = 42)]
    block_count: u64,

    /// Pool ranking by block count (1 = most blocks)
    #[schemars(example = 1)]
    rank: u32,

    /// Number of empty blocks mined
    #[serde(rename = "emptyBlocks")]
    #[schemars(example = 0)]
    empty_blocks: u64,

    /// URL-friendly pool identifier
    slug: PoolSlug,

    /// Pool's share of total blocks (0.0 - 1.0)
    #[schemars(example = 0.30)]
    share: f64,

    /// Unique pool identifier
    #[serde(rename = "poolUniqueId")]
    #[schemars(example = 44)]
    pool_unique_id: u8,
}

impl PoolStats {
    /// Create a new PoolStats from a Pool reference
    pub fn new(pool: &'static Pool, block_count: u64, rank: u32, share: f64) -> Self {
        Self {
            pool_id: pool.mempool_id(),
            name: Cow::Borrowed(pool.name),
            link: Cow::Borrowed(pool.link),
            block_count,
            rank,
            empty_blocks: 0,
            slug: pool.slug(),
            share,
            pool_unique_id: pool.mempool_unique_id(),
        }
    }
}
