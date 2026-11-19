use brk_types::{ONE_HOUR_IN_SEC, Timestamp};

use crate::{AGE_BOUNDARIES, AGE_RANGE_COUNT, AgeRangeId};

/// For each age boundary, the first origin younger than it at the last timestamp.
pub fn age_boundary_positions(timestamps: &[Timestamp]) -> [usize; AGE_RANGE_COUNT - 1] {
    let Some(&current) = timestamps.last() else {
        return [0; AGE_RANGE_COUNT - 1];
    };
    AGE_BOUNDARIES.map(|hours| {
        let oldest = i64::from(*current) - i64::from(hours as u32 * ONE_HOUR_IN_SEC);
        timestamps.partition_point(|t| i64::from(**t) <= oldest)
    })
}

/// Visits the origins crossing each age boundary, youngest boundary first, as (origin, younger
/// range, older range, crossing time). The caller supplies the previous state's monotonic
/// timestamps and the boundaries' positions from that state. Positions advance to `current`.
pub fn for_each_age_crossing(
    timestamps: &[Timestamp],
    current: Timestamp,
    positions: &mut [usize; AGE_RANGE_COUNT - 1],
    mut visit: impl FnMut(usize, AgeRangeId, AgeRangeId, u32),
) {
    let Some(previous) = timestamps.last() else {
        return;
    };
    if current <= *previous {
        return;
    }
    for (i, &hours) in AGE_BOUNDARIES.iter().enumerate() {
        let duration = hours as u32 * ONE_HOUR_IN_SEC;
        let upper = i64::from(*current) - i64::from(duration);
        let mut end = positions[i];
        while end < timestamps.len() && i64::from(*timestamps[end]) <= upper {
            visit(
                end,
                AgeRangeId::ALL[i],
                AgeRangeId::ALL[i + 1],
                *timestamps[end] + duration,
            );
            end += 1;
        }
        positions[i] = end;
    }
}
