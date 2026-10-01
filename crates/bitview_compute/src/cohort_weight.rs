use bitview_cohort::AgeRange;
use brk_types::{Height, Sats, StoredF64};
use vecdb::ReadableVec;

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

pub fn collect_cohort_weights(
    height: Height,
    weights: &AgeRange<&impl ReadableVec<Height, StoredF64>>,
    supplies: &AgeRange<Sats>,
) -> Option<AgeRange<f64>> {
    AgeRange::try_from_fn(|age| {
        let supply = *age.select(supplies);
        resolve_cohort_weight(age.select(weights).collect_one(height), supply).ok_or(())
    })
    .ok()
}

#[cfg(test)]
mod tests {
    use brk_types::{Sats, StoredF64};

    use super::resolve_cohort_weight;

    #[test]
    fn empty_cohorts_allow_missing_weights_but_populated_cohorts_do_not() {
        for value in [
            None,
            Some(StoredF64::NAN),
            Some(StoredF64::from(f64::INFINITY)),
        ] {
            assert_eq!(resolve_cohort_weight(value, Sats::ZERO), Some(0.0));
            assert_eq!(resolve_cohort_weight(value, Sats::from(1_u64)), None);
        }
        for supply in [Sats::ZERO, Sats::from(1_u64)] {
            for (value, expected) in [(-0.25, 0.0), (0.25, 0.25), (1.25, 1.0)] {
                assert_eq!(
                    resolve_cohort_weight(Some(StoredF64::from(value)), supply),
                    Some(expected)
                );
            }
        }
    }
}
