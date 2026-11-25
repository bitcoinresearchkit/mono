use brk_types::BlockHash;

use super::{
    transaction::analysis::TransactionAnalysis,
    txin::InputResolver,
    txout::{BlockAddresses, ProcessedOutput},
};

/// Reusable buffers cleared and refilled each block to avoid allocation churn.
#[derive(Default)]
pub struct BlockBuffers {
    pub inputs: InputResolver,
    pub addresses: BlockAddresses,
    pub outputs: Vec<ProcessedOutput>,
    /// The outputs as resolved, which the analysis reads while finalization rewrites `outputs`.
    pub analyzed_outputs: Vec<ProcessedOutput>,
    pub analyses: Vec<TransactionAnalysis>,
    tip: Option<BlockHash>,
}

impl BlockBuffers {
    /// Starts a pass after `parent` that has `backlog` blocks to index: the address cache is emptied when the chain
    /// does not continue from the last block indexed, and sized for the backlog.
    pub fn continue_from(&mut self, parent: Option<BlockHash>, backlog: u32) {
        if self.tip != parent {
            self.addresses.clear_cache();
        }
        self.addresses.size_cache_for(backlog);
        self.tip = parent;
    }

    pub fn finish_block(&mut self, blockhash: BlockHash) {
        self.tip = Some(blockhash);
    }
}
