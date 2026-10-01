use bitview_plugin_indexer::Indexer;
use brk_types::{Height, OutPoint, Sats, TxInIndex, TxIndex, TxOutIndex, Weight};
use vecdb::{Budgeted, BytesVec, OverflowVec, PcoVec};

/// Only the raw columns needed by the fee pass; no indexer lifecycle state.
#[derive(Clone, Copy)]
pub(super) struct Sources<'a> {
    pub block_txs: &'a PcoVec<Height, TxIndex, Budgeted>,
    pub block_inputs: &'a PcoVec<Height, TxInIndex, Budgeted>,
    pub block_outputs: &'a PcoVec<Height, TxOutIndex, Budgeted>,
    pub tx_inputs: &'a PcoVec<TxIndex, TxInIndex>,
    pub tx_outputs: &'a BytesVec<TxIndex, TxOutIndex>,
    pub weights: &'a PcoVec<TxIndex, Weight>,
    pub outpoints: &'a PcoVec<TxInIndex, OutPoint>,
    pub values: &'a OverflowVec<TxOutIndex, Sats>,
}

impl<'a> From<&'a Indexer> for Sources<'a> {
    fn from(indexer: &'a Indexer) -> Self {
        let raw = indexer.vecs();
        Self {
            block_txs: &raw.transactions.first_tx_index,
            block_inputs: &raw.inputs.first_txin_index,
            block_outputs: &raw.outputs.first_txout_index,
            tx_inputs: &raw.transactions.first_txin_index,
            tx_outputs: &raw.transactions.first_txout_index,
            weights: &raw.transactions.weight,
            outpoints: &raw.inputs.outpoint,
            values: &raw.outputs.value,
        }
    }
}
