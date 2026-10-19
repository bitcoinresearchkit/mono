use std::borrow::Cow;

use bitview_primitives::{Pool, PoolSlug};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PoolInfo {
    #[schemars(example = &"Foundry USA")]
    pub name: Cow<'static, str>,

    /// URL-friendly pool identifier
    pub slug: PoolSlug,

    /// Zero-based pool ID, matching mempool.space's `unique_id`
    #[schemars(example = 44)]
    pub unique_id: u8,
}

impl From<&'static Pool> for PoolInfo {
    fn from(pool: &'static Pool) -> Self {
        Self {
            name: Cow::Borrowed(pool.name),
            slug: pool.slug,
            unique_id: pool.mempool_unique_id(),
        }
    }
}
