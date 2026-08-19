mod compute;
mod import;

use brk_error::Result;

use bitview_plugin::{Plugin, PluginGate, PluginId};
use bitview_traversable::Traversable;
use brk_types::{Height, OpReturnKind, OpReturnPolicyId, Version};
use vecdb::{Database, Rw, StorageMode};

use super::{breakdown::BreakdownVecs, total::Total};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    plugin_gate: PluginGate,
    #[traversable(skip)]
    db: Database,
    /// Metrics across every `OP_RETURN` output and every transaction carrying
    /// at least one such output.
    pub total: Total<M>,
    /// Metrics split by detected `OP_RETURN` payload kind. Output bytes belong
    /// to one kind, while transaction counts, full virtual sizes, and full fees
    /// are counted once for every kind present in a transaction, so those
    /// metrics can overlap across kinds.
    pub by_kind: BreakdownVecs<OpReturnKind, M>,
    /// Metrics split by pre-v30 `OP_RETURN` relay-policy shape. `oversized` and
    /// `multiple` can overlap, and both are subsets of `pre_v30_nonstandard`;
    /// `pre_v30_standard` is the complementary category.
    pub policy: BreakdownVecs<OpReturnPolicyId, M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn id(&self) -> PluginId {
        super::ID
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}

impl Vecs {
    fn min_len(&self) -> usize {
        self.total
            .len()
            .min(self.by_kind.len())
            .min(self.policy.len())
    }

    fn validate_and_truncate(&mut self, version: Version, height: Height) -> Result<()> {
        self.total.validate_and_truncate(version, height)?;
        self.by_kind.validate_and_truncate(version, height)?;
        self.policy.validate_and_truncate(version, height)?;
        Ok(())
    }

    fn truncate_if_needed_at(&mut self, len: usize) -> Result<()> {
        self.total.truncate_if_needed_at(len)?;
        self.by_kind.truncate_if_needed_at(len)?;
        self.policy.truncate_if_needed_at(len)?;
        Ok(())
    }

    fn write(&mut self) -> Result<()> {
        self.total.write()?;
        self.by_kind.write()?;
        self.policy.write()?;
        Ok(())
    }
}
