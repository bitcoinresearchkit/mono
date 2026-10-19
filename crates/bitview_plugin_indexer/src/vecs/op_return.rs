use bitview_primitives::{Bytes, Bytes32, OpReturnIndex, OpReturnKind};
use bitview_traversable::{Compact, Traversable};
use brk_error::Result;
use brk_types::{Height, TxIndex, Version};
use vecdb::{AnyStoredVec, Database, ImportableVec, PcoVec, Rw, StorageMode};

#[derive(Traversable)]
pub struct OpReturnVecs<M: StorageMode = Rw> {
    /// Zero-based OP_RETURN-output index at which the indexed block begins,
    /// equal to the number of OP_RETURN outputs in all preceding blocks.
    pub first_index: M::Stored<PcoVec<Height, OpReturnIndex>>,
    /// The transaction containing the `OP_RETURN` output.
    #[traversable(rename = "tx_index")]
    pub to_tx_index: M::Stored<PcoVec<OpReturnIndex, TxIndex>>,
    /// Heuristic classification of the indexed OP_RETURN output from the first
    /// non-empty pushed-data prefix, or from the full post-OP_RETURN byte count
    /// for length-based formats. `runes` is recognized from an immediate
    /// `OP_13` and `witness_commitment` from the BIP141 header; a payload
    /// without a non-empty push is `empty`, and other unmatched payloads are
    /// `text`, `bare_hash`, or `unknown`.
    #[traversable(rename = "protocol")]
    pub kind: M::Stored<PcoVec<OpReturnIndex, OpReturnKind>>,
    /// Number of serialized locking-script bytes after the initial
    /// `OP_RETURN` opcode, including push opcodes and push-length prefixes.
    pub post_op_return_bytes: Compact<PcoVec<OpReturnIndex, Bytes32>, Bytes, M>,
}

impl OpReturnVecs {
    pub fn import(db: &Database, version: Version) -> Result<Self> {
        let (first_index, to_tx_index, kind, post_op_return_bytes) = parallel_import! {
            first_index = PcoVec::import(db, "first_op_return_index", version),
            to_tx_index = PcoVec::import(db, "op_return_tx_index", version),
            kind = PcoVec::import(db, "op_return_protocol", version),
            post_op_return_bytes =
                PcoVec::import(db, "post_op_return_bytes", version).map(Compact::new),
        };
        Ok(Self {
            first_index,
            to_tx_index,
            kind,
            post_op_return_bytes,
        })
    }

    pub fn iter_any(&self) -> impl Iterator<Item = &dyn AnyStoredVec> {
        [
            &self.first_index as &dyn AnyStoredVec,
            &self.to_tx_index,
            &self.kind,
            &*self.post_op_return_bytes,
        ]
        .into_iter()
    }
}
