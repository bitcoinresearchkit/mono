use crate::PoolSlug;

#[derive(Debug)]
pub struct Pool {
    pub slug: PoolSlug,
    pub name: &'static str,
    /// What its series are of, leading their descriptions: "Uses blocks mined by Foundry USA."
    pub lead: &'static str,
    pub(crate) addrs: Box<[&'static str]>,
    pub(crate) tags: Box<[&'static str]>,
    /// Lowercase coinbase tags for case-insensitive matching
    pub(crate) tags_lowercase: Box<[String]>,
    pub(crate) link: &'static str,
}

impl Pool {
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
