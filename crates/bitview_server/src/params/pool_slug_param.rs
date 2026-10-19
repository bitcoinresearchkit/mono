use bitview_primitives::PoolSlug;
use brk_types::Height;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct PoolSlugParam {
    pub slug: PoolSlug,
}

#[derive(Deserialize, JsonSchema)]
pub struct PoolSlugAndHeightParam {
    pub slug: PoolSlug,
    pub height: Height,
}
