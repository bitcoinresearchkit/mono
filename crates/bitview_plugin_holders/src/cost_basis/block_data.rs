use bitview_distribution::state::cost_basis::PercentileResult;
use bitview_primitives::PartsPerMillion32;
use bitview_vecs::Density;
use brk_types::Cents;

pub struct CostBasisBlockData {
    pub percentiles: PercentileResult,
    /// The lowest and highest price any of its coins last moved at, exactly.
    pub min: Cents,
    pub max: Cents,
    pub supply_density: Density<PartsPerMillion32>,
    pub capital_density: Density<PartsPerMillion32>,
}
