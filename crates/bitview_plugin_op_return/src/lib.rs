mod batch;
mod breakdown;
mod by_kind;
mod compute;
mod dependencies;
mod import;
mod policy;

pub use dependencies::Dependencies;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_plugin_indexer::FlagView;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::Version;
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use bitview_primitives::Count;
use bitview_vecs::{IndexSources, LazyWindowStartVec, PerBlockCumulativeRolling, shapes::Windows};
use breakdown::{BucketVecs, KindBreakdownVecs, PolicyBreakdownVecs};

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("op_return"), Version::new(11));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    /// Transactions that create at least one output whose locking script begins with `OP_RETURN`.
    /// Whether the transaction is one of them.
    flag: FlagView,
    /// Metrics across every `OP_RETURN` output and every transaction creating at least one.
    #[traversable(flatten)]
    #[traversable(lead = "OP_RETURN outputs: data written on the chain, never spendable.")]
    total: BucketVecs<M>,
    /// Metrics by detected `OP_RETURN` payload protocol. Output bytes belong
    /// to one protocol, while transaction counts, full virtual sizes, and full
    /// fees are counted once for every protocol present in a transaction, so
    /// those metrics can overlap across protocols.
    #[traversable(lead = "OP_RETURN outputs, by the protocol that wrote them.")]
    protocols: KindBreakdownVecs<M>,
    /// Metrics by the size and count limits of the pre-v30 `OP_RETURN` relay
    /// policy, over non-coinbase transactions. These categories check output
    /// count and data size only, not full relay standardness (a non-push-only
    /// `OP_RETURN` script still lands in `pre_v30_standard`). `oversized` and
    /// `multiple` can overlap, and both are subsets of `pre_v30_nonstandard`;
    /// `pre_v30_standard` is the complementary category.
    policies: PolicyBreakdownVecs<M>,
    /// `OP_RETURN` outputs by how many bytes follow the opcode.
    sizes: Sizes<M>,
}

/// Outputs over each limit: the difference of two is the outputs between them.
#[derive(Traversable)]
pub(crate) struct Sizes<M: StorageMode = Rw> {
    /// More than 42 bytes after the opcode.
    #[traversable(
        wrap = "over_42_bytes",
        rename = "output_count",
        client = "OutputCount",
        lead = "OP_RETURN outputs over 42 bytes."
    )]
    over_42_bytes: PerBlockCumulativeRolling<Count, M>,
    /// More than 520 bytes after the opcode: more than one data push can hold.
    #[traversable(
        wrap = "over_520_bytes",
        rename = "output_count",
        client = "OutputCount",
        lead = "OP_RETURN outputs over 520 bytes."
    )]
    over_520_bytes: PerBlockCumulativeRolling<Count, M>,
    /// More than 10,000 bytes after the opcode.
    #[traversable(
        wrap = "over_10k_bytes",
        rename = "output_count",
        client = "OutputCount",
        lead = "OP_RETURN outputs over 10,000 bytes."
    )]
    over_10k_bytes: PerBlockCumulativeRolling<Count, M>,
}

impl Sizes {
    const LIMITS: [u32; 3] = [42, 520, 10_000];

    fn import(
        db: &Database,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let over = |name: &str| {
            PerBlockCumulativeRolling::import(
                db,
                &format!("op_return_data_over_{name}_bytes_output_count"),
                version,
                indexes,
                window_starts,
            )
        };
        Ok(Self {
            over_42_bytes: over("42")?,
            over_520_bytes: over("520")?,
            over_10k_bytes: over("10k")?,
        })
    }

    fn all_mut(&mut self) -> [&mut PerBlockCumulativeRolling<Count>; 3] {
        [
            &mut self.over_42_bytes,
            &mut self.over_520_bytes,
            &mut self.over_10k_bytes,
        ]
    }

    /// Adds an output of `bytes` to a block's count over each limit.
    pub(crate) fn add_output(counts: &mut [u64; 3], bytes: u32) {
        for (count, limit) in counts.iter_mut().zip(Self::LIMITS) {
            *count += u64::from(bytes > limit);
        }
    }

    pub(crate) fn push_block(&mut self, counts: [u64; 3]) {
        for (over, count) in self.all_mut().into_iter().zip(counts) {
            over.push_block(Count::from(count));
        }
    }
}

impl Vecs {
    /// Every bucket: the total, then the protocols and the policies.
    fn buckets_mut(&mut self) -> impl Iterator<Item = &mut BucketVecs> {
        [&mut self.total]
            .into_iter()
            .chain(self.protocols.iter_mut())
            .chain(self.policies.iter_mut())
    }

    /// Every column a block pushes to, in all buckets.
    fn stored_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let Self {
            total,
            protocols,
            policies,
            sizes,
            ..
        } = self;
        [total]
            .into_iter()
            .chain(protocols.iter_mut())
            .chain(policies.iter_mut())
            .flat_map(BucketVecs::stored_mut)
            .chain(sizes.iter_any_stored_mut())
            .collect()
    }
}
