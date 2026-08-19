use bitview_traversable::Traversable;
use brk_types::{Height, TxIndex};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{Formattable, PcoVec, PcoVecValue, Rw, StorageMode, VecIndex};

#[derive(Traversable)]
pub struct ScriptTypeVecs<
    I: VecIndex + PcoVecValue + Formattable + Serialize + JsonSchema,
    M: StorageMode = Rw,
> {
    /// Zero-based type-specific output index at which the indexed block begins,
    /// equal to the number of outputs of this script type in preceding blocks.
    pub first_index: M::Stored<PcoVec<Height, I>>,
    /// Global zero-based index of a transaction in canonical blockchain order.
    /// At `tx_index`, this is the identity value; at `txin_index`, it identifies
    /// the transaction containing the input; at type-specific output indexes,
    /// it identifies the transaction containing that output.
    pub to_tx_index: M::Stored<PcoVec<I, TxIndex>>,
}
