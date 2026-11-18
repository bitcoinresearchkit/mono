//! The next block's template, built once per projection.

use crate::{DatedTemplate, ReadOnlyState};
use brk_types::BlockHash;

impl ReadOnlyState {
    /// The next block's template (BIP 22) after `tip`, dated now: the published block 0, when it
    /// was packed at `tip`. Core never checked the block it describes (debug builds check every
    /// published one).
    pub fn block_template(&self, tip: &BlockHash) -> Option<DatedTemplate> {
        if self.projection_tip() != Some(*tip) {
            return None;
        }
        self.template.as_ref().map(|template| template.dated())
    }
}
