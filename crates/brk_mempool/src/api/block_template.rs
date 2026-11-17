//! The next block's template, served as built.

use std::sync::{Arc, OnceLock};

use brk_error::{Error, Result};
use brk_types::{BlockHash, tag_hash};

use crate::ReadOnlyMempool;

/// The next block's template as JSON, shared.
#[derive(Clone)]
pub struct BlockTemplateJson(Arc<Template>);

pub(crate) struct Template {
    json: Vec<u8>,
    /// The tip its block extends.
    tip: BlockHash,
    /// The bytes' validator, hashed by the first request: most templates are never served.
    hash: OnceLock<u64>,
}

impl Template {
    pub(crate) fn new(json: Vec<u8>, tip: BlockHash) -> Self {
        Self { json, tip, hash: OnceLock::new() }
    }
}

impl BlockTemplateJson {
    #[must_use]
    pub fn hash(&self) -> u64 {
        *self.0.hash.get_or_init(|| tag_hash(&self.0.json))
    }
}

impl AsRef<[u8]> for BlockTemplateJson {
    fn as_ref(&self) -> &[u8] {
        &self.0.json
    }
}

impl ReadOnlyMempool {
    /// The newest template Core accepted: block 0's of the newest snapshot checked, which trails
    /// the published block 0 by one check. `StateUpdating` until one is accepted at the published
    /// block 0's tip: one at an older tip describes a block no longer minable.
    pub fn block_template(&self) -> Result<BlockTemplateJson> {
        let tip = self.load().projection_tip();
        self.template
            .load_full()
            .filter(|template| Some(template.tip) == tip)
            .map(BlockTemplateJson)
            .ok_or_else(|| Error::updating())
    }
}
