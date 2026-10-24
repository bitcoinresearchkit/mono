use bitview_primitives::PoolSlug;
use bitview_query::{Error, Result};
use brk_types::Height;
use schemars::JsonSchema;
use serde::Deserialize;

/// A pool's slug as the path writes it: an unknown one is a missing resource (404), as
/// mempool.space answers, not a malformed request.
#[derive(Clone, Deserialize)]
#[serde(transparent)]
pub struct SlugText(String);

impl SlugText {
    pub fn pool(&self) -> Result<PoolSlug> {
        self.0.parse().map_err(|()| Error::UnknownPool)
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct PoolSlugParam {
    #[schemars(with = "PoolSlug")]
    pub slug: SlugText,
}

#[derive(Deserialize, JsonSchema)]
pub struct PoolSlugAndHeightParam {
    #[schemars(with = "PoolSlug")]
    pub slug: SlugText,
    pub height: Height,
}
