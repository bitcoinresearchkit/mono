use bitview_cohort::{SpendableType, SpendableTypeId};
use bitview_traversable::Traversable;
use brk_types::{PartsPerMillion32, StoredU16, StoredU64};
use vecdb::{Rw, StorageMode};

use super::WithInputTypes;
use bitview_compute::{
    ColumnarPerBlock, ColumnarPerBlockCumulativeRolling, LazyColumnCountPerBlockCumulativeRolling,
    LazyColumnPerBlockCumulativeRolling, LazyPercentCumulativeRolling,
};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Counts of transaction inputs. The `all` aggregate includes one coinbase
    /// input per block. Per-type series exclude coinbase and classify inputs by
    /// the BRK output type of the previous output they spend; `OP_RETURN` is
    /// excluded because it is unspendable.
    pub input_count: ColumnarPerBlock<
        StoredU16,
        SpendableTypeId,
        WithInputTypes<LazyColumnCountPerBlockCumulativeRolling>,
        M,
    >,
    /// Inputs spending a previous-output type divided by all inputs
    /// over the same cumulative or trailing window. The denominator includes
    /// coinbase inputs.
    pub input_share: SpendableType<LazyPercentCumulativeRolling<PartsPerMillion32>>,
    /// Number of non-coinbase transactions containing at least one input that
    /// spends a previous-output type. Each transaction is counted
    /// once per type; the `all` aggregate counts every non-coinbase transaction.
    pub tx_count: ColumnarPerBlockCumulativeRolling<
        StoredU64,
        SpendableTypeId,
        WithInputTypes<LazyColumnPerBlockCumulativeRolling<StoredU64, SpendableTypeId>>,
        M,
    >,
    /// Non-coinbase transactions containing a previous-output type
    /// divided by all non-coinbase transactions over the same cumulative or
    /// trailing window.
    pub tx_share: SpendableType<LazyPercentCumulativeRolling<PartsPerMillion32>>,
}
