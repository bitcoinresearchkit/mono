use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::{Height, PoolSlug};

/// Mining pool slug path parameter
#[derive(Deserialize, JsonSchema)]
pub struct PoolSlugParam {
    pub slug: PoolSlug,
}

/// Mining pool slug + block height path parameters
#[derive(Deserialize, JsonSchema)]
pub struct PoolSlugAndHeightParam {
    pub slug: PoolSlug,
    pub height: Height,
}
