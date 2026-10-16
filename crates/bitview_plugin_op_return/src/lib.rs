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
use brk_types::Version;
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

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
    total: BucketVecs<M>,
    /// Metrics by detected `OP_RETURN` payload protocol. Output bytes belong
    /// to one protocol, while transaction counts, full virtual sizes, and full
    /// fees are counted once for every protocol present in a transaction, so
    /// those metrics can overlap across protocols.
    protocols: KindBreakdownVecs<M>,
    /// Metrics by the size and count limits of the pre-v30 `OP_RETURN` relay
    /// policy, over non-coinbase transactions. These categories check output
    /// count and data size only, not full relay standardness (a non-push-only
    /// `OP_RETURN` script still lands in `pre_v30_standard`). `oversized` and
    /// `multiple` can overlap, and both are subsets of `pre_v30_nonstandard`;
    /// `pre_v30_standard` is the complementary category.
    policies: PolicyBreakdownVecs<M>,
}

impl Vecs {
    /// Every column a block pushes to, in all buckets.
    fn stored_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.total
            .stored_mut()
            .into_iter()
            .chain(self.protocols.iter_mut().flat_map(BucketVecs::stored_mut))
            .chain(self.policies.iter_mut().flat_map(BucketVecs::stored_mut))
    }
}
