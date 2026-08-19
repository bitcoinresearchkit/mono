use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{Height, OpReturnIndex, OpReturnKind, StoredU32, TxIndex, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, Database, ImportableVec, PcoVec, Rw, Stamp, StorageMode, WritableVec};

#[derive(Traversable)]
pub struct OpReturnVecs<M: StorageMode = Rw> {
    /// Zero-based OP_RETURN-output index at which the indexed block begins,
    /// equal to the number of OP_RETURN outputs in all preceding blocks.
    pub first_index: M::Stored<PcoVec<Height, OpReturnIndex>>,
    /// Global zero-based index of a transaction in canonical blockchain order.
    /// At `tx_index`, this is the identity value; at `txin_index`, it identifies
    /// the transaction containing the input; at type-specific output indexes,
    /// it identifies the transaction containing that output.
    pub to_tx_index: M::Stored<PcoVec<OpReturnIndex, TxIndex>>,
    /// Heuristic classification of the indexed OP_RETURN output from the first
    /// non-empty pushed-data prefix, or from the full post-OP_RETURN byte count
    /// for length-based formats. `runes` is recognized from an immediate
    /// `OP_13`; unmatched payloads are `text`, `bare_hash`, or `unknown`.
    pub kind: M::Stored<PcoVec<OpReturnIndex, OpReturnKind>>,
    /// Number of serialized locking-script bytes after the initial
    /// `OP_RETURN` opcode, including push opcodes and push-length prefixes.
    pub post_op_return_bytes: M::Stored<PcoVec<OpReturnIndex, StoredU32>>,
}

impl OpReturnVecs {
    pub fn forced_import(db: &Database, version: Version) -> Result<Self> {
        let (first_index, to_tx_index, kind, post_op_return_bytes) = parallel_import! {
            first_index = PcoVec::forced_import(db, "first_op_return_index", version),
            to_tx_index = PcoVec::forced_import(db, "tx_index", version),
            kind = PcoVec::forced_import(db, "kind", version),
            post_op_return_bytes =
                PcoVec::forced_import(db, "op_return_post_op_return_bytes", version),
        };
        Ok(Self {
            first_index,
            to_tx_index,
            kind,
            post_op_return_bytes,
        })
    }

    pub fn truncate(
        &mut self,
        height: Height,
        op_return_index: OpReturnIndex,
        stamp: Stamp,
    ) -> Result<()> {
        self.first_index
            .truncate_if_needed_with_stamp(height, stamp)?;
        self.to_tx_index
            .truncate_if_needed_with_stamp(op_return_index, stamp)?;
        self.kind
            .truncate_if_needed_with_stamp(op_return_index, stamp)?;
        self.post_op_return_bytes
            .truncate_if_needed_with_stamp(op_return_index, stamp)?;
        Ok(())
    }

    pub fn par_iter_mut_any(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        [
            &mut self.first_index as &mut dyn AnyStoredVec,
            &mut self.to_tx_index,
            &mut self.kind,
            &mut self.post_op_return_bytes,
        ]
        .into_par_iter()
    }

    pub fn iter_any(&self) -> impl Iterator<Item = &dyn AnyStoredVec> {
        [
            &self.first_index as &dyn AnyStoredVec,
            &self.to_tx_index,
            &self.kind,
            &self.post_op_return_bytes,
        ]
        .into_iter()
    }
}
