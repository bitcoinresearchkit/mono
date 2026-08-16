use brk_traversable::Traversable;
use brk_types::{Height, SigOps, TxIndex};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{BytesVec, Formattable, PcoVec, PcoVecValue, Rw, StorageMode, VecIndex};

#[derive(Traversable)]
pub struct ScriptTypeWithSigOpsVecs<
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
    /// BIP-141 signature-operation cost attributable to the indexed locking
    /// script using accurate multisig counting. Each `CHECKSIG` costs four;
    /// `CHECKMULTISIG` costs four times a preceding `OP_1` through `OP_16`, or
    /// 80 when no such key-count opcode precedes it.
    pub legacy_sigops: M::Stored<BytesVec<I, SigOps>>,
}
