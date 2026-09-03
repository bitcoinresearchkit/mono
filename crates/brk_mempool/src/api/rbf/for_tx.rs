use brk_types::Txid;

use super::RbfNode;

#[derive(Debug, Clone, Default)]
pub struct RbfForTx {
    /// Tree rooted at the terminal replacer. `None` if `txid` is unknown.
    pub root: Option<RbfNode>,
    /// Direct predecessors of the requested tx (txids only).
    pub replaces: Vec<Txid>,
}

impl RbfForTx {
    pub fn is_empty(&self) -> bool {
        self.root.is_none() && self.replaces.is_empty()
    }
}
