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
    pub analyses: Vec<TransactionAnalysis>,
    tip: Option<BlockHash>,
}

impl BlockBuffers {
    pub fn continue_from(&mut self, parent: Option<BlockHash>) {
        if self.tip != parent {
            self.addresses.clear_cache();
        }
        self.tip = parent;
    }

    pub fn finish_block(&mut self, blockhash: BlockHash) {
        self.tip = Some(blockhash);
    }
}
