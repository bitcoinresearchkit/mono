use bitcoin::taproot::LeafVersion;

use super::revealed::Revealed;

pub struct WitnessFacts<'a> {
    pub has_annex: bool,
    pub last: Option<&'a [u8]>,
    pub leaf_version: Option<LeafVersion>,
    pub max_argument_bytes: usize,
    pub stack_items: usize,
    /// The witness script's walk, P2WSH's native or nested; none for any other witness.
    pub script: Revealed,
    pub inscription: bool,
}
