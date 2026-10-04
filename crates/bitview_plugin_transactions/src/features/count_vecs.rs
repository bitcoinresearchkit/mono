use bitview_primitives::Count;
use bitview_traversable::Traversable;
use bitview_vecs::PerBlockCumulativeRolling;
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct CountVecs<M: StorageMode = Rw> {
    /// Counts transactions containing at least one Taproot input with more
    /// than one witness element whose final element begins with annex prefix
    /// byte `0x50`.
    pub annex: PerBlockCumulativeRolling<Count, M>,
    /// Counts transactions containing at least one detected `SIGHASH_ALL`
    /// signature. This base hash type commits the signature to every output;
    /// the independently detected `SIGHASH_ANYONECANPAY` modifier can narrow its
    /// input commitment to the signing input.
    pub sighash_all: PerBlockCumulativeRolling<Count, M>,
    /// Counts transactions containing at least one detected `SIGHASH_NONE`
    /// signature. This base hash type commits to no transaction outputs, so
    /// outputs may be changed after signing.
    pub sighash_none: PerBlockCumulativeRolling<Count, M>,
    /// Counts transactions containing at least one detected `SIGHASH_SINGLE`
    /// signature. This base hash type normally commits only to the output at the
    /// same position as the signing input; legacy signatures without a
    /// corresponding output retain Bitcoin's historical `SIGHASH_SINGLE` bug.
    pub sighash_single: PerBlockCumulativeRolling<Count, M>,
    /// Counts transactions containing at least one detected Taproot
    /// `SIGHASH_DEFAULT` signature. Taproot's omitted hash-type byte has the
    /// same commitments as `SIGHASH_ALL` without `SIGHASH_ANYONECANPAY`.
    pub sighash_default: PerBlockCumulativeRolling<Count, M>,
    /// Counts transactions containing at least one detected signature with the
    /// `SIGHASH_ANYONECANPAY` modifier, which commits only to the signing input
    /// rather than every input. This is counted independently from
    /// `SIGHASH_ALL`, `SIGHASH_NONE`, and `SIGHASH_SINGLE`.
    pub sighash_anyone_can_pay: PerBlockCumulativeRolling<Count, M>,
    /// Counts non-coinbase transactions containing at least one output below
    /// BRK's type-specific minimal non-dust value; `OP_RETURN` is excluded.
    pub dust_output: PerBlockCumulativeRolling<Count, M>,
}
