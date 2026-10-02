use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use brk_exit::Exit;
use brk_types::{Height, PartsPerMillionSigned32, PartsPerMillionSigned64, StoredF64};
use vecdb::ReadableVec;

use super::super::activity;
use super::Vecs;

#[inline]
fn adjusted_inflation(
    active_to_vaulted: StoredF64,
    inflation: PartsPerMillionSigned64,
) -> PartsPerMillionSigned32 {
    PartsPerMillionSigned32::from(f64::from(active_to_vaulted) * f64::from(inflation))
}

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    inflation_rate: &impl ReadableVec<Height, PartsPerMillionSigned64>,
    velocity_native: &impl ReadableVec<Height, StoredF64>,
    velocity_fiat: &impl ReadableVec<Height, StoredF64>,
    activity: &activity::Vecs,
    exit: &Exit,
) -> Result<()> {
    let starting_height = indexer.safe_lengths().height;

    vecs.inflation_rate.ppm.height.compute_transform2(
        starting_height,
        &activity.ratio.height,
        inflation_rate,
        |(h, a2vr, inflation, ..)| (h, adjusted_inflation(a2vr, inflation)),
        exit,
    )?;

    vecs.tx_velocity_native.height.compute_multiply(
        starting_height,
        &activity.ratio.height,
        velocity_native,
        exit,
    )?;

    vecs.tx_velocity_fiat.height.compute_multiply(
        starting_height,
        &activity.ratio.height,
        velocity_fiat,
        exit,
    )?;

    Ok(())
}
