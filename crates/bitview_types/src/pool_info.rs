use std::borrow::Cow;

use bitview_primitives::{Pool, PoolSlug};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Basic pool information for listing all pools
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PoolInfo {
    /// Pool name
    #[schemars(example = &"Foundry USA")]
    name: Cow<'static, str>,

    /// URL-friendly pool identifier
    slug: PoolSlug,

    /// Unique numeric pool identifier
    #[schemars(example = 44)]
    unique_id: u8,
}

impl From<&'static Pool> for PoolInfo {
    fn from(pool: &'static Pool) -> Self {
        Self {
            name: Cow::Borrowed(pool.name),
            slug: pool.slug(),
            unique_id: pool.mempool_unique_id(),
        }
    }
}
