use schemars::JsonSchema;
use serde::Serialize;

use super::PoolSlug;

/// Mining pool information
#[derive(Debug, Serialize, JsonSchema)]
pub struct Pool {
    /// Unique pool identifier
    pub slug: PoolSlug,

    /// Pool name
    pub name: &'static str,

    /// Known payout addresses for pool identification
    #[serde(skip)]
    pub(crate) addrs: Box<[&'static str]>,

    /// Coinbase tags used to identify blocks mined by this pool
    #[serde(skip)]
    pub(crate) tags: Box<[&'static str]>,

    /// Lowercase coinbase tags for case-insensitive matching
    #[serde(skip)]
    #[schemars(skip)]
    pub(crate) tags_lowercase: Box<[String]>,

    /// Pool website URL
    pub(crate) link: &'static str,
}

impl Pool {
    /// Get slug of pool
    pub fn slug(&self) -> PoolSlug {
        self.slug
    }

    /// Known payout addresses for pool identification
    pub fn addrs(&self) -> &[&'static str] {
        &self.addrs
    }

    /// Coinbase tags used to identify blocks mined by this pool
    pub fn tags(&self) -> &[&'static str] {
        &self.tags
    }

    /// Pool website URL
    pub fn link(&self) -> &'static str {
        self.link
    }

    /// Pool ID matching mempool.space's `unique_id` field (0-indexed, raw pools-v2.json value)
    pub fn mempool_unique_id(&self) -> u8 {
        self.slug.into()
    }

    /// Pool ID matching mempool.space's `id` field (1-indexed)
    pub fn mempool_id(&self) -> u8 {
        self.mempool_unique_id() + 1
    }
}
