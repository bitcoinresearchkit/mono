mod batch;
mod breakdown;
mod by_kind;
mod compute;
mod dependencies;
mod import;
mod policy;
mod total;

pub use dependencies::Dependencies;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{Database, Rw, StorageMode};

use breakdown::{KindBreakdownVecs, PolicyBreakdownVecs};
use total::Total;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("op_return"), Version::new(9));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    /// Metrics across every `OP_RETURN` output and every transaction carrying
    /// at least one such output.
    #[traversable(flatten)]
    total: Total<M>,
    /// Metrics by detected `OP_RETURN` payload protocol. Output bytes belong
    /// to one protocol, while transaction counts, full virtual sizes, and full
    /// fees are counted once for every protocol present in a transaction, so
    /// those metrics can overlap across protocols.
    protocols: KindBreakdownVecs<M>,
    /// Metrics by pre-v30 `OP_RETURN` relay-policy shape. `oversized` and
    /// `multiple` can overlap, and both are subsets of `pre_v30_nonstandard`;
    /// `pre_v30_standard` is the complementary category.
    policies: PolicyBreakdownVecs<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}

impl Vecs {
    fn min_len(&self) -> usize {
        self.total
            .len()
            .min(self.protocols.len())
            .min(self.policies.len())
    }

    fn validate_and_truncate(&mut self, version: Version, height: Height) -> Result<()> {
        self.total.validate_and_truncate(version, height)?;
        self.protocols.validate_and_truncate(version, height)?;
        self.policies.validate_and_truncate(version, height)?;
        Ok(())
    }

    fn truncate_if_needed_at(&mut self, len: usize) -> Result<()> {
        self.total.truncate_if_needed_at(len)?;
        self.protocols.truncate_if_needed_at(len)?;
        self.policies.truncate_if_needed_at(len)?;
        Ok(())
    }

    fn write(&mut self) -> Result<()> {
        self.total.write()?;
        self.protocols.write()?;
        self.policies.write()?;
        Ok(())
    }
}
