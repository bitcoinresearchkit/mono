use brk_types::{Cents, CostBasisByPercentile};

/// Coin- and capital-weighted prices from sorted, rounded URPD buckets.
pub(super) struct PriceStats {
    pub cost_basis: CostBasisByPercentile,
    pub capitalized_price: Cents,
}

impl Default for PriceStats {
    fn default() -> Self {
        Self {
            cost_basis: CostBasisByPercentile::default(),
            capitalized_price: Cents::NAN,
        }
    }
}
