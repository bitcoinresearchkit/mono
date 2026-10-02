use brk_types::{Sats, Version};
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, Database, EagerVec, ImportableVec, PcoVec, ReadOnlyClone, ReadableVec,
    WritableVec,
};

use super::*;

#[test]
fn cumulative_delta_matches_range_sum_across_read_strategies() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut cumulative: EagerVec<PcoVec<Height, Sats>> =
        EagerVec::forced_import(&db, "cumulative", Version::ONE).unwrap();
    let values = (0_usize..2_051)
        .map(|index| Sats::from(index + 1))
        .collect::<Vec<_>>();
    let mut total = Sats::ZERO;

    for value in &values {
        total += *value;
        cumulative.push(total);
    }
    cumulative.write().unwrap();

    let read_only = cumulative.read_only_clone();
    let page = read_only.cursor_chunk_size();
    for (start, end) in [
        (0, 0),
        (0, values.len() - 1),
        (values.len() - 1, values.len() - 1),
        (values.len() - page, values.len() - 1),
        (values.len() - page - 1, values.len() - 1),
        (32, 1_750),
    ] {
        let expected = values[start..=end].iter().copied().sum();
        let actual = cumulative_delta(&read_only, Height::from(start), Height::from(end)).unwrap();
        assert_eq!(actual, expected, "range {start}..={end}");
    }
}
