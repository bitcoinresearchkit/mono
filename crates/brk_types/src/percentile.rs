use crate::VSize;

/// Get a percentile value from a sorted (value, vsize) slice using
/// vsize-weighted interpolation — matches mempool.space's feeRange calculation.
///
/// Walks through the sorted pairs accumulating vsize. When cumulative vsize
/// crosses `total_vsize * percentile`, returns that value.
///
/// # Panics
/// Panics if the slice is empty.
fn get_weighted_percentile<T: Clone>(sorted_with_vsizes: &[(T, VSize)], percentile: f64) -> T {
    assert!(
        !sorted_with_vsizes.is_empty(),
        "Cannot get percentile from empty slice"
    );
    let total: u64 = sorted_with_vsizes.iter().map(|(_, v)| u64::from(*v)).sum();
    let target = (total as f64 * percentile).round() as u64;
    let mut cumulative = 0u64;
    for (value, vsize) in sorted_with_vsizes {
        cumulative += u64::from(*vsize);
        if cumulative >= target {
            return value.clone();
        }
    }
    sorted_with_vsizes.last().unwrap().0.clone()
}

/// Compute nondecreasing vsize-weighted percentile ranks with one total and one
/// cumulative walk. Rounding and zero-weight behavior match the single-rank API.
///
/// # Panics
/// Panics for an empty population or decreasing rounded percentile targets.
pub fn get_weighted_percentiles<T: Clone, const N: usize>(
    sorted_with_vsizes: &[(T, VSize)],
    percentiles: [f64; N],
) -> [T; N] {
    // A single rank is faster with the straight scan; this branch folds away
    // for the five- and seven-rank callers.
    if N == 1 {
        return percentiles.map(|p| get_weighted_percentile(sorted_with_vsizes, p));
    }
    assert!(
        !sorted_with_vsizes.is_empty(),
        "Cannot get percentile from empty slice"
    );
    let total: u64 = sorted_with_vsizes.iter().map(|(_, v)| u64::from(*v)).sum();
    let targets = percentiles.map(|percentile| (total as f64 * percentile).round() as u64);
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
