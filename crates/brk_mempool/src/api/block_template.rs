//! The next block's template, built once per published projection.

use crate::ReadOnlyState;
use brk_types::BlockHash;

impl ReadOnlyState {
    /// The next block's template (BIP 22) after `tip`, as JSON dated now, with its content
    /// identity: the published block 0, when it was packed at `tip`. Core never checked the
    /// block it describes (debug builds check every published one).
    pub fn block_template(&self, tip: &BlockHash) -> Option<(Vec<u8>, u64)> {
        if self.projection_tip() != Some(*tip) {
            return None;
        }
        self.template.as_ref().map(|template| template.dated_now())
    }
}
