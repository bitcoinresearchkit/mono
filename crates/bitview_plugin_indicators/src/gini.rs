use bitview_cohort::AmountRange;
use bitview_compute::PercentPerBlock;
use bitview_plugin_distribution::Vecs as DistributionVecs;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, PartsPerMillion32, Sats, StoredU64};

pub fn compute(
    gini: &mut PercentPerBlock<PartsPerMillion32>,
    distribution: &DistributionVecs,
    starting_height: Height,
    exit: &Exit,
) -> Result<()> {
    gini.ppm.height.compute_transform2(
        starting_height,
        &distribution
            .cohorts
            .supply
            .total
            .matrices
            .amount_range_matrix,
        &distribution
            .cohorts
            .outputs
            .unspent_count
            .matrices
            .amount_range_matrix,
        |(height, supply, count, ..)| (height, gini_from_lorenz(&count, &supply)),
        exit,
    )?;
    Ok(())
}

fn gini_from_lorenz(
    counts: &AmountRange<StoredU64>,
    supplies: &AmountRange<Sats>,
) -> PartsPerMillion32 {
    let total_count: u64 = counts.iter().copied().map(u64::from).sum();
    let total_supply: u64 = supplies.iter().copied().map(u64::from).sum();

    if total_count == 0 || total_supply == 0 {
        return PartsPerMillion32::ZERO;
    }

    let mut cumulative_supply = 0u64;
    let mut numerator = 0.0f64;

    for (count, supply) in counts.iter().zip(supplies.iter()) {
        let previous_supply = cumulative_supply;
        cumulative_supply += u64::from(*supply);
        numerator += u64::from(*count) as f64 * (previous_supply + cumulative_supply) as f64;
    }

    let denominator = total_count as f64 * total_supply as f64;
    PartsPerMillion32::from(1.0 - numerator / denominator)
}
