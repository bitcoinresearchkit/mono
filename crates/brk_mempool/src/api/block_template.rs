//! The next block's template, built from the published block 0 as it is asked.

use brk_types::Timestamp;

use crate::{ReadOnlyState, TipFacts, template};

impl ReadOnlyState {
    /// The next block's template (BIP 22) as JSON: the published block 0, dated now, at `tip`,
    /// what the chain says of the tip it was packed at ([`Self::projection_tip`]). Core never
    /// checked the block it describes (debug builds check every published block 0).
    pub fn block_template(&self, tip: &TipFacts) -> Vec<u8> {
        debug_assert_eq!(Some(tip.header.block_hash().into()), self.projection_tip());
        let template = template::build(&self.snapshot().block0, tip, Timestamp::now());
        serde_json::to_vec(&template).expect("a template serializes")
    }
}
