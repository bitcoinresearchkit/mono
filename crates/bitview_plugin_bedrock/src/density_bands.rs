use bitview_traversable::Traversable;
use brk_types::{Cents, CentsCompact, PartsPerMillion32, Sats};

use crate::SupplyDensity;

#[derive(Debug, PartialEq, Traversable)]
pub struct DensityBands<T> {
    pub supply_density: T,
    pub supply_density_10pct: T,
}

impl<T> DensityBands<T> {
    pub fn try_from_fn<E>(mut create: impl FnMut(&str) -> Result<T, E>) -> Result<Self, E> {
        Ok(Self {
            supply_density: create("")?,
            supply_density_10pct: create("_10pct")?,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.supply_density, &self.supply_density_10pct].into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.supply_density, &mut self.supply_density_10pct].into_iter()
    }
}

impl DensityBands<SupplyDensity<PartsPerMillion32>> {
    pub fn from_entries(
        entries: impl Iterator<Item = (CentsCompact, Sats)> + Clone,
        spot: Cents,
    ) -> Self {
        Self {
            supply_density: SupplyDensity::from_entries::<5>(entries.clone(), spot),
            supply_density_10pct: SupplyDensity::from_entries::<10>(entries, spot),
        }
    }
}

impl Default for DensityBands<SupplyDensity<PartsPerMillion32>> {
    fn default() -> Self {
        Self {
            supply_density: SupplyDensity::NAN,
            supply_density_10pct: SupplyDensity::NAN,
        }
    }
}
