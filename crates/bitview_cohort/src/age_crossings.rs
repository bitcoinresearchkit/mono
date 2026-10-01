use crate::{AGE_BOUNDARIES, AGE_RANGE_COUNT, AGE_RANGE_IDS, AgeRangeId};
use brk_types::{ONE_HOUR_IN_SEC, Timestamp};

/// Visit origins crossing each age boundary, youngest first. The caller supplies
/// the previous state's monotonic timestamps and resets positions after a rewind.
pub fn for_each_age_crossing(
    timestamps: &[Timestamp],
    current: Timestamp,
    positions: &mut [usize; AGE_RANGE_COUNT - 1],
    mut visit: impl FnMut(usize, AgeRangeId, AgeRangeId, u32),
) {
    for_each_age_cutoff(
        timestamps,
        current,
        &AGE_BOUNDARIES,
        positions,
        |i, origin, timestamp| {
            visit(origin, AGE_RANGE_IDS[i], AGE_RANGE_IDS[i + 1], timestamp);
        },
    );
}

/// Visit only the requested cutoffs, retaining boundary search positions between blocks.
pub fn for_each_age_cutoff<const N: usize>(
    timestamps: &[Timestamp],
    current: Timestamp,
    hours: &[usize; N],
    positions: &mut [usize; N],
    mut visit: impl FnMut(usize, usize, u32),
) {
    let Some(previous) = timestamps.last() else {
        return;
    };
    if current <= *previous {
        return;
    }
    for (i, &hours) in hours.iter().enumerate() {
        let duration = hours as u32 * ONE_HOUR_IN_SEC;
        let lower = i64::from(**previous) - i64::from(duration);
        let upper = i64::from(*current) - i64::from(duration);
        if upper <= lower {
            continue;
        }
        let mut start = positions[i];
        if start == 0 {
            start = timestamps.partition_point(|t| i64::from(**t) <= lower);
        } else {
            while start < timestamps.len() && i64::from(*timestamps[start]) <= lower {
                start += 1;
            }
        }
        let mut end = start;
        while end < timestamps.len() && i64::from(*timestamps[end]) <= upper {
            visit(i, end, (*timestamps[end]).saturating_add(duration));
            end += 1;
        }
        positions[i] = end;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HOURS_4M, HOURS_5M, HOURS_6M};

    #[test]
    fn selected_cutoffs_match_full_age_crossings_with_gaps_and_flat_timestamps() {
        let timestamps = [0, 0, 1, 119, 120, 120, 149, 150, 180, 400]
            .map(|days| Timestamp::new(1_231_006_505 + days * 86400));
        let hours = [HOURS_4M, HOURS_5M, HOURS_6M];
        let mut all_positions = [0; AGE_RANGE_COUNT - 1];
        let mut selected_positions = [0; 3];
        for h in 0..timestamps.len() {
            let mut expected = Vec::new();
            for_each_age_crossing(
                &timestamps[..h],
                timestamps[h],
                &mut all_positions,
                |origin, before, _, time| {
                    for (i, &hours) in hours.iter().enumerate() {
                        if AGE_BOUNDARIES[before.index()] == hours {
                            expected.push((i, origin, time));
                        }
                    }
                },
            );
            let mut actual = Vec::new();
            for_each_age_cutoff(
                &timestamps[..h],
                timestamps[h],
                &hours,
                &mut selected_positions,
                |i, origin, time| actual.push((i, origin, time)),
            );
            assert_eq!(actual, expected);
        }
    }
}
