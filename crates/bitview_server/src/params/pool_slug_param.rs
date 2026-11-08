use bitview_primitives::{PoolSlug, pools};
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
    /// A slug no pool has (reserved ones included) is unknown.
    pub fn pool(&self) -> Result<PoolSlug> {
        self.0
            .parse()
            .ok()
            .filter(|&slug| pools().find(slug).is_some())
            .ok_or(Error::UnknownPool)
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
    #[serde(deserialize_with = "bitview_types::de_plain")]
    pub height: Height,
}
