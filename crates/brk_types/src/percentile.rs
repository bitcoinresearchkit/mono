use crate::VSize;

/// Vsize-weighted percentiles of a sorted (value, vsize) slice with Bitcoin Core's
/// `getblockstats` rank rule: for each percent `p` (0 to 100, nondecreasing), the first value
/// whose cumulative vsize reaches `p`% of the total, in one walk.
///
/// # Panics
/// Panics for an empty population or decreasing percentiles.
pub fn get_weighted_percentiles<T: Clone, const N: usize>(
    sorted_with_vsizes: &[(T, VSize)],
    percents: [u64; N],
) -> [T; N] {
    assert!(
        !sorted_with_vsizes.is_empty(),
        "Cannot get percentile from empty slice"
    );
    let total: u64 = sorted_with_vsizes.iter().map(|(_, v)| u64::from(*v)).sum();
    let targets = percents.map(|percent| (total * percent).div_ceil(100));
    assert!(
        targets.windows(2).all(|pair| pair[0] <= pair[1]),
        "Percentile targets must be nondecreasing"
    );
    let mut entries = sorted_with_vsizes.iter();
    let mut current = entries.next().unwrap();
    let mut cumulative = u64::from(current.1);
    targets.map(|target| {
        while cumulative < target {
            let Some(next) = entries.next() else { break };
            current = next;
            cumulative += u64::from(current.1);
        }
        current.0.clone()
    })
}
