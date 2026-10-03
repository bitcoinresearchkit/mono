use bitview_primitives::TxOutIndex;
use brk_types::TxIndex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ParentRead {
    pub tx_index: TxIndex,
    pub first_txout_index: TxOutIndex,
}
