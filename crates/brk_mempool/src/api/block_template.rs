//! The next block's template, served as built.

use std::{
    hash::Hasher,
    sync::{Arc, OnceLock},
};

use brk_error::{Error, Result};

use crate::{ReadOnlyState, json};

/// The next block's template as JSON, shared.
#[derive(Clone)]
pub struct BlockTemplateJson(Arc<Template>);

struct Template {
    json: Vec<u8>,
    /// The bytes' validator, hashed by the first request: most templates are never served.
    hash: OnceLock<u64>,
}

impl BlockTemplateJson {
    pub(crate) fn new(json: Vec<u8>) -> Self {
        Self(Arc::new(Template { json, hash: OnceLock::new() }))
    }

    #[must_use]
    pub fn hash(&self) -> u64 {
        *self.0.hash.get_or_init(|| {
            let mut hasher = json::keyed_hasher();
            hasher.write(&self.0.json);
            hasher.finish()
        })
    }
}

impl PartialEq for BlockTemplateJson {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl AsRef<[u8]> for BlockTemplateJson {
    fn as_ref(&self) -> &[u8] {
        &self.0.json
    }
}

impl ReadOnlyState {
    /// The template of the published projection's block 0: `StateUpdating` until Core accepts
    /// it (before the first publication, while the chain can't say what the next block's header
    /// is, after a rejection).
    pub fn block_template(&self) -> Result<BlockTemplateJson> {
        self.snapshot().ensure_projection()?;
        self.template.clone().ok_or_else(Error::updating)
    }
}
