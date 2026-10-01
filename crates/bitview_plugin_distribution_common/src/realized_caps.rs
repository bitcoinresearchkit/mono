use brk_error::Result;
use brk_types::{CentsSats, Height, Version};
use vecdb::{
    AnyStoredVec, AnyVec, BytesVec, Database, ImportOptions, ImportableVec, MutableVec,
    ReadableVec, Stamp, WritableVec,
};

/// Exact cost basis for one plugin's cohorts, with the same rollback stamps as its state.
pub struct RealizedCaps<const N: usize> {
    values: MutableVec<BytesVec<usize, CentsSats>>,
}

impl<const N: usize> RealizedCaps<N> {
    pub fn import(db: &Database, saved_checkpoints: u16) -> Result<Self> {
        Ok(Self {
            values: MutableVec::forced_import_with(
                ImportOptions::new(db, "cohort_caps", Version::TWO)
                    .with_saved_stamped_changes(saved_checkpoints),
            )?,
        })
    }

    pub fn end(&self) -> usize {
        if self.values.len() == N {
            usize::from(Height::from(self.values.stamp()).incremented())
        } else {
            0
        }
    }

    pub fn validate(&mut self, version: Version) -> Result<bool> {
        let changed =
            self.values.header().computed_version() != self.values.header().vec_version() + version;
        self.values.validate_computed_version_or_reset(version)?;
        Ok(changed)
    }

    pub fn rollback_before(&mut self, stamp: Stamp) -> Result<Stamp> {
        Ok(self.values.rollback_before(stamp)?)
    }

    pub fn reset(&mut self) -> Result<()> {
        self.values.reset()?;
        Ok(())
    }

    pub fn load(&self) -> Option<[CentsSats; N]> {
        self.values.collect().try_into().ok()
    }

    pub fn save(
        &mut self,
        caps: impl Iterator<Item = CentsSats>,
        stamp: Stamp,
        with_changes: bool,
    ) -> Result<()> {
        if self.values.is_empty() {
            self.values.extend(caps);
        } else {
            self.values.update_many(caps.enumerate())?;
        }
        assert_eq!(self.values.len(), N, "incomplete realized-cap checkpoint");
        self.values
            .stamped_write_maybe_with_changes(stamp, with_changes)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "realized_caps_tests.rs"]
mod tests;
