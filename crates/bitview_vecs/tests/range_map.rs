use bitview_vecs::RangeMapVec;
use brk_types::{Height, TxIndex, Version};
use rangeindex::SharedRangeMap;
use vecdb::ReadableVec;

#[test]
fn readers_share_rewrites_appends_and_resets() {
    let mapping = SharedRangeMap::new([0, 2, 2, 5].map(TxIndex::new).to_vec());
    let index =
        RangeMapVec::<Height, TxIndex>::new("first_tx_index", Version::ONE, mapping.clone());
    let clone = index.clone();
    assert_eq!(clone.collect_one_at(4), None);
    let mut out = vec![TxIndex::new(99)];
    index.read_sorted_into_at(&[0, 1, 1, 3, 4, usize::MAX], &mut out);
    assert_eq!(out, [99, 0, 2, 2, 5].map(TxIndex::new));

    mapping.update_at(2, [3, 8].map(TxIndex::new));
    assert_eq!(clone.collect(), [0, 2, 3, 8].map(TxIndex::new));
    mapping.update_at(4, [9].map(TxIndex::new));
    assert_eq!(clone.collect_one_at(4), Some(TxIndex::new(9)));
    mapping.update_at(0, []);
    assert!(clone.collect().is_empty());
}

#[test]
fn all_range_paths_clamp_and_preserve_early_exit() {
    let index = RangeMapVec::<Height, u64>::new(
        "values",
        Version::ONE,
        SharedRangeMap::new(vec![0, 2, 4, 6, 8]),
    );
    for (from, to) in [(0, 5), (1, 4), (3, 99), (9, 10), (4, 1)] {
        let end = to.min(5);
        let expected: Vec<_> = (from.min(end)..end).map(|i| i as u64 * 2).collect();
        assert_eq!(index.collect_range_at(from, to), expected);
        let mut chunks = Vec::new();
        index.for_each_chunk_at(from, to, &mut |at, values| {
            assert_eq!(at, from + chunks.len());
            chunks.extend_from_slice(values);
        });
        assert_eq!(chunks, expected);
        assert_eq!(
            index.fold_range_at(from, to, 0, |sum, value| sum + value),
            expected.iter().sum::<u64>()
        );
    }
    let mut visited = Vec::new();
    let result = index.try_fold_range_at(0, 5, (), |(), value| {
        visited.push(value);
        if value == 4 { Err("stop") } else { Ok(()) }
    });
    assert_eq!(result, Err("stop"));
    assert_eq!(visited, [0, 2, 4]);
}
