//! The next block's template, served as built.

use std::{
    hash::Hasher,
    sync::{Arc, OnceLock},
};

use brk_error::{Error, Result};

use crate::{ReadOnlyMempool, json};

/// The next block's template as JSON, shared.
#[derive(Clone)]
pub struct BlockTemplateJson(Arc<Template>);

pub(crate) struct Template {
    json: Vec<u8>,
    /// The bytes' validator, hashed by the first request: most templates are never served.
    hash: OnceLock<u64>,
}

impl Template {
    pub(crate) fn new(json: Vec<u8>) -> Self {
        Self { json, hash: OnceLock::new() }
    }
}

impl BlockTemplateJson {
    #[must_use]
    pub fn hash(&self) -> u64 {
        *self.0.hash.get_or_init(|| {
            let mut hasher = json::keyed_hasher();
            hasher.write(&self.0.json);
            hasher.finish()
        })
    }
}

impl AsRef<[u8]> for BlockTemplateJson {
    fn as_ref(&self) -> &[u8] {
        &self.0.json
    }
}

impl ReadOnlyMempool {
    /// The newest template Core accepted: block 0's of the newest snapshot checked, which trails
    /// the published block 0 by one check. `StateUpdating` until the first is accepted.
    pub fn block_template(&self) -> Result<BlockTemplateJson> {
        self.template.load_full().map(BlockTemplateJson).ok_or_else(Error::updating)
    }
}
