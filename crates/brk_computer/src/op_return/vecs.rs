use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Height, OpReturnKind, OpReturnPolicyId, Version};
use vecdb::{Database, Rw, StorageMode};

use super::{breakdown::BreakdownVecs, total::Total};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    pub(crate) db: Database,
    pub total: Total<M>,
    pub by_kind: BreakdownVecs<OpReturnKind, M>,
    pub policy: BreakdownVecs<OpReturnPolicyId, M>,
}

impl Vecs {
    pub(crate) fn min_len(&self) -> usize {
        self.total
            .len()
            .min(self.by_kind.len())
            .min(self.policy.len())
    }

    pub(crate) fn validate_and_truncate(&mut self, version: Version, height: Height) -> Result<()> {
        self.total.validate_and_truncate(version, height)?;
        self.by_kind.validate_and_truncate(version, height)?;
        self.policy.validate_and_truncate(version, height)?;
        Ok(())
    }

    pub(crate) fn truncate_if_needed_at(&mut self, len: usize) -> Result<()> {
        self.total.truncate_if_needed_at(len)?;
        self.by_kind.truncate_if_needed_at(len)?;
        self.policy.truncate_if_needed_at(len)?;
        Ok(())
    }

    pub(crate) fn write(&mut self) -> Result<()> {
        self.total.write()?;
        self.by_kind.write()?;
        self.policy.write()?;
        Ok(())
    }
}
