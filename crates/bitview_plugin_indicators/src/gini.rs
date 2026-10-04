use bitview_cohort::AmountRange;
use bitview_plugin_distribution_utxos::Vecs as UtxosVecs;
use bitview_primitives::{Count, PartsPerMillion32};
use bitview_vecs::PercentPerBlock;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Sats, Version};
use vecdb::{AnyVec, ReadableVec, WritableVec};

pub fn compute(
    gini: &mut PercentPerBlock<PartsPerMillion32>,
    distribution_utxos: &UtxosVecs,
    starting_height: Height,
    exit: &Exit,
) -> Result<()> {
    let supplies = &distribution_utxos
        .cohorts
        .supply
        .total
        .stored
        .cohorts
        .utxo_amount;
    let counts = &distribution_utxos
        .cohorts
        .outputs
        .unspent_count
        .stored
        .cohorts
        .utxo_amount;
    let end = supplies
        .iter()
        .map(AnyVec::len)
        .chain(counts.iter().map(AnyVec::len))
        .min()
        .unwrap_or_default();
    let version = Version::combine_all(
        supplies
            .iter()
            .map(AnyVec::version)
            .chain(counts.iter().map(AnyVec::version)),
    );
    let batch_size = 4096;
    gini.ppm.height.compute_batched_to(
        starting_height,
        end,
        version,
        batch_size,
        |target, range| {
            let supplies = AmountRange::from_fn(|id| {
                id.select(supplies).collect_range_at(range.start, range.end)
            });
            let counts = AmountRange::from_fn(|id| {
                id.select(counts).collect_range_at(range.start, range.end)
            });
            for offset in 0..range.len() {
                let supply = AmountRange::from_fn(|id| id.select(&supplies)[offset]);
                let count = AmountRange::from_fn(|id| id.select(&counts)[offset]);
                target.push(gini_from_lorenz(&count, &supply));
            }
            Ok(())
        },
        exit,
    )?;
    Ok(())
}

fn gini_from_lorenz(
    counts: &AmountRange<Count>,
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
