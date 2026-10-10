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
    /// The transaction containing the output.
    #[traversable(rename = "tx_index")]
    pub to_tx_index: M::Stored<PcoVec<I, TxIndex>>,
}
