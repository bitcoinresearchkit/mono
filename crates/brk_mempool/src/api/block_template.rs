//! Core's `getblocktemplate` response, served as it arrived.

use std::{
    hash::Hasher,
    sync::{Arc, OnceLock},
};

use brk_error::{Error, Result};

use crate::{ReadOnlyState, json};

/// One poll's `getblocktemplate` response, its bytes as Core sent them, shared.
#[derive(Clone)]
pub struct BlockTemplateJson(Arc<Template>);

struct Template {
    json: Box<str>,
    /// The bytes' validator, hashed by the first request: most polls' templates are never served.
    hash: OnceLock<u64>,
}

impl BlockTemplateJson {
    pub(crate) fn new(json: Box<str>) -> Self {
        Self(Arc::new(Template { json, hash: OnceLock::new() }))
    }

    #[must_use]
    pub fn hash(&self) -> u64 {
        *self.0.hash.get_or_init(|| {
            let mut hasher = json::keyed_hasher();
            hasher.write(self.0.json.as_bytes());
            hasher.finish()
        })
    }
}

impl AsRef<[u8]> for BlockTemplateJson {
    fn as_ref(&self) -> &[u8] {
        self.0.json.as_bytes()
    }
}

impl ReadOnlyState {
    /// Core's template from the poll the published projection was built from: none before the
    /// first publication (`StateUpdating`), nor while Core gives none (`NotFound`).
    pub fn block_template(&self) -> Result<BlockTemplateJson> {
        self.snapshot().ensure_projection()?;
        self.template.clone().ok_or_else(|| {
            Error::NotFound(
                "Bitcoin Core gives no block template now (in initial sync, or with no peers)"
                    .into(),
            )
        })
    }
}
