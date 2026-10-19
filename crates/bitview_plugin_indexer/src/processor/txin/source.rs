use bitview_primitives::{TxOutIndex, TypeIndex};
use brk_types::{OutPoint, OutputType, Sats, SigOps32};

#[derive(Debug, Clone, Copy)]
pub enum InputSource {
    Coinbase,
    PreviousBlock {
        outpoint: OutPoint,
        txout_index: TxOutIndex,
        output_type: OutputType,
        legacy_sigop_cost: SigOps32,
        type_index: TypeIndex,
        value: Sats,
    },
    SameBlock {
        outpoint: OutPoint,
        txout_offset: usize,
        txout_index: TxOutIndex,
        value: Sats,
    },
}
