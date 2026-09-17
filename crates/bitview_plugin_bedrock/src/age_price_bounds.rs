use std::path::Path;

use bitview_cohort::AgeRangeId;
use bitview_plugin_distribution::AgeRangeUrpds;
use brk_error::Result;
use brk_types::{Cents, CentsCompact, Date, Sats};

use crate::{AgeCutoffs, PriceBounds};

impl AgeCutoffs<PriceBounds<Cents>> {
    pub fn include(&mut self, age: AgeRangeId, price: CentsCompact, sats: Sats) {
        if sats == Sats::ZERO {
            return;
        }
        for bounds in self.containing_mut(age) {
            bounds.include(Cents::from(price));
        }
    }

    pub fn read_if_exists(states_path: &Path, date: Date) -> Result<Self> {
        let mut bounds = Self::default();
        if !AgeRangeUrpds::path(states_path, date).try_exists()? {
            return Ok(bounds);
        }
        let sources = AgeRangeUrpds::read(states_path, date)?;
        for &age in &AgeRangeId::ALL[..AgeRangeId::From6MTo9M.index()] {
            // Snapshot entries are sorted. Only their occupied endpoints can
            // change an age cutoff's bounds; no aggregate URPD is needed.
            let entries = sources.get(age);
            for &(price, sats) in entries
                .iter()
                .find(|(_, sats)| *sats != Sats::ZERO)
                .into_iter()
                .chain(entries.iter().rfind(|(_, sats)| *sats != Sats::ZERO))
            {
                bounds.include(age, price, sats);
            }
        }
        Ok(bounds)
    }
}

#[cfg(test)]
#[path = "../tests/unit/age_price_bounds.rs"]
mod tests;
