use bitview_distribution::state::cost_basis::PercentileResult;
use bitview_primitives::PartsPerMillion32;
use bitview_vecs::Density;

pub struct CostBasisBlockData {
    pub percentiles: PercentileResult,
    pub supply_density: Density<PartsPerMillion32>,
    pub capital_density: Density<PartsPerMillion32>,
}
