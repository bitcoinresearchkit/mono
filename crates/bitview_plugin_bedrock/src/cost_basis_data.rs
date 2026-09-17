use brk_types::{
    Cents, CentsCompact, CostBasisByPercentile, PERCENTILES_LEN, PartsPerMillion32, Sats, UrpdRaw,
};

use crate::{DensityBands, SupplyDensity};

#[derive(Debug, PartialEq)]
pub struct CostBasisData {
    pub prices: CostBasisByPercentile,
    pub supply_density: SupplyDensity<PartsPerMillion32>,
    pub supply_density_10pct: SupplyDensity<PartsPerMillion32>,
}

impl CostBasisData {
    pub fn from_entries(
        entries: impl Iterator<Item = (CentsCompact, Sats)> + Clone,
        spot: Cents,
    ) -> Self {
        let DensityBands {
            supply_density,
            supply_density_10pct,
        } = DensityBands::from_entries(entries.clone(), spot);
        Self {
            prices: UrpdRaw::cost_basis_percentile_prices_from_entries(entries),
            supply_density,
            supply_density_10pct,
        }
    }
}

impl Default for CostBasisData {
    fn default() -> Self {
        Self {
            prices: CostBasisByPercentile {
                per_coin: [Cents::NAN; PERCENTILES_LEN],
                per_dollar: [Cents::NAN; PERCENTILES_LEN],
            },
            supply_density: SupplyDensity::NAN,
            supply_density_10pct: SupplyDensity::NAN,
        }
    }
}
