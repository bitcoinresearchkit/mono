use bitview_cohort::{ByType, OutputTypeId};
use bitview_plugin_indexer::BlockCounterVecs;
use bitview_primitives::Count;
use bitview_traversable::Traversable;
use bitview_vecs::{
    IndexSources, LazyWindowStartVec, PerBlockCumulativeRolling, counter_group, shapes::Windows,
};
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

counter_group! {
    BlockCounterVecs;

    pub(crate) struct Pubkeys {
        /// 33-byte public keys in outputs: in P2PK and P2MS scripts, curve-checked. P2TR's x-only
        /// keys aren't ECDSA keys and aren't counted.
        #[traversable(lead = "Compressed public keys in outputs.")]
        compressed: Count => compressed_pubkey_in_output_count,
        /// 65-byte public keys in outputs, by the same rule.
        #[traversable(lead = "Uncompressed public keys in outputs.")]
        uncompressed: Count => uncompressed_pubkey_in_output_count,
    }
}

/// The outputs of each block's coinbase transaction.
#[derive(Traversable)]
pub(crate) struct Coinbase<M: StorageMode = Rw> {
    /// All outputs of coinbase transactions.
    #[traversable(lead = "Coinbase outputs.")]
    count: PerBlockCumulativeRolling<Count, M>,
    /// Coinbase outputs by type.
    types: ByType<PerBlockCumulativeRolling<Count, M>>,
}

impl Coinbase {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            count: PerBlockCumulativeRolling::import(
                db,
                "coinbase_output_count",
                version,
                indexes,
                window_starts,
            )?,
            types: ByType::try_from_fn(|kind| {
                let name = format!("coinbase_{}_output_count", kind.key());
                PerBlockCumulativeRolling::import(db, &name, version, indexes, window_starts)
            })?,
        })
    }

    /// A block's: its coinbase's outputs, in all and by [`OutputTypeId::index`].
    pub(crate) fn push_block(&mut self, total: u64, types: &[u64; OutputTypeId::COUNT]) {
        self.count.push_block(Count::from(total));
        for (column, count) in self.types.iter_mut().zip(types) {
            column.push_block(Count::from(*count));
        }
    }
}
