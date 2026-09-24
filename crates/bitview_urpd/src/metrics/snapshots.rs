use std::path::PathBuf;

use bitview_cohort::UTXOAggregateId;
use brk_error::Result;
use brk_types::{Date, Day1};
use vecdb::StorageMode;

use super::Metrics;
use crate::{UrpdRaw, prune_snapshots};

impl<M: StorageMode> Metrics<M> {
    pub fn dir(&self, id: UTXOAggregateId) -> PathBuf {
        UrpdRaw::dir(&self.states_path, id.cohort_name().id)
    }

    pub fn raw_bytes(&self, id: UTXOAggregateId, date: Date) -> Result<Vec<u8>> {
        UrpdRaw::read_bytes(&self.states_path, id.cohort_name().id, date)
    }
}

impl Metrics {
    pub(super) fn write_snapshot(
        &self,
        id: UTXOAggregateId,
        date: Date,
        urpd: &UrpdRaw,
    ) -> Result<()> {
        UrpdRaw::write(
            &self.states_path,
            id.cohort_name().id,
            date,
            urpd.map.iter().map(|(&price, &sats)| (price, sats)),
        )
    }

    /// Remove snapshots being replaced before publishing any new result.
    pub(super) fn prune_snapshots(&self, start: usize, version_matches: bool) -> Result<()> {
        for &id in UTXOAggregateId::ALL {
            prune_snapshots(
                &self.dir(id),
                version_matches.then(|| Date::from(Day1::from(start))),
            )?;
        }
        Ok(())
    }
}
