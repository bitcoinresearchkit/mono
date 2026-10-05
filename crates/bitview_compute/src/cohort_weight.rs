use bitview_cohort::AgeRange;
use bitview_primitives::Ratio64;
use brk_types::{Height, Sats};
use vecdb::{Cursor, ReadableVec};

/// Empty cohorts may have no observation; populated cohorts require a finite one.
fn resolve_cohort_value<T>(value: Option<T>, supply: Sats) -> Option<f64>
where
    f64: From<T>,
{
    match value.map(f64::from) {
        Some(value) if value.is_finite() => Some(value),
        _ if supply == Sats::ZERO => Some(0.0),
        _ => None,
    }
}

pub fn resolve_cohort_weight<T>(value: Option<T>, supply: Sats) -> Option<f64>
where
    f64: From<T>,
{
    resolve_cohort_value(value, supply).map(|value| value.clamp(0.0, 1.0))
}

/// One block's cohort weights, read through cursors: a per-block loop decodes each page once
/// instead of once per block.
pub fn collect_cohort_weights<V: ReadableVec<Height, Ratio64>>(
    height: Height,
    weights: &mut AgeRange<Cursor<'_, Height, Ratio64, V>>,
    supplies: &AgeRange<Sats>,
) -> Option<AgeRange<f64>> {
    AgeRange::try_from_fn(|age| {
        let supply = *age.select(supplies);
        resolve_cohort_weight(age.select_mut(weights).get(usize::from(height)), supply).ok_or(())
    })
    .ok()
}
