use rangeindex::SharedRangeMap;

#[test]
fn clones_share_forward_and_reverse_mappings_without_storage_traits() {
    let map = SharedRangeMap::<usize, usize>::new(vec![0, 2, 2, 5]);
    let reader = map.clone();
    assert_eq!(reader.len(), 4);
    assert_eq!(reader.read().as_slice(), [0, 2, 2, 5]);
    assert_eq!(reader.read().get_shared(2), Some(2));

    map.update_at(2, [3, 8]);
    assert_eq!(reader.read().as_slice(), [0, 2, 3, 8]);
    assert_eq!(reader.read().get_shared(2), Some(1));
    map.update_at(4, [8, 9]);
    assert_eq!(reader.read().as_slice(), [0, 2, 3, 8, 8, 9]);
    assert_eq!(reader.read().get_shared(8), Some(4));
    map.update_at(3, []);
    assert_eq!(reader.read().as_slice(), [0, 2, 3]);
    map.update_at(0, []);
    assert!(reader.is_empty());
}

#[test]
fn cached_cursors_use_shared_boundaries_and_refresh_after_updates() {
    let map = SharedRangeMap::<usize, usize>::new(vec![3, 3, 5, 10, 1029, 2053]);
    let reader = map.clone();
    for (from, suffix) in [(6, vec![3077]), (2, vec![8, 8, 9]), (0, vec![])] {
        map.update_at(from, suffix);
        let guard = reader.read();
        let mut cursor = guard.cached_cursor();
        for _ in 0..3 {
            for index in [0, 3, 5, 5, 8, 10, 1029, 2053, 3077, 5, usize::MAX] {
                assert_eq!(
                    cursor.get(index),
                    guard.as_slice().iter().rposition(|&start| start <= index)
                );
            }
        }
    }
}
