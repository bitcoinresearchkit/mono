use brk_rpc::Auth;
use brk_types::{BlockHash, FeeRate, Txid};

use super::*;

impl Mempool {
    pub(crate) fn for_test() -> Self {
        let client = Client::new("http://127.0.0.1:1", Auth::None).unwrap();
        Self::new(&client)
    }

    pub(crate) fn test_state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    pub(crate) fn published(&self) -> Arc<ReadOnlyState> {
        self.read_only.load()
    }

    pub(crate) fn test_tick(&mut self, gbt_txids: &[Txid], min_fee: FeeRate) {
        self.rebuilder.tick(&self.state, gbt_txids, min_fee);
        self.publish_observation(BlockHash::default(), true);
    }

    pub(crate) fn test_publish(&mut self, tip: BlockHash) {
        let ids: Vec<_> = self.state.txs.txids().copied().collect();
        self.rebuilder.tick(&self.state, &ids, FeeRate::new(1.0));
        self.publish_observation(tip, true);
    }
}
