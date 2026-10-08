use std::collections::{BTreeMap, BTreeSet};

use crate::{InternalValue, ValueType, compaction::stream::CompactionStream};

fn merge(items: Vec<InternalValue>, last_level: bool) -> Vec<InternalValue> {
    CompactionStream::new(items.into_iter().map(Ok))
        .evict_tombstones(last_level)
        .map(Result::unwrap)
        .collect()
}

fn version(seqno: u64, value_type: ValueType) -> InternalValue {
    let value: &[u8] = if value_type.is_tombstone() { b"" } else { b"v" };
    InternalValue::from_components("k", value, seqno, value_type)
}

fn types(items: &[InternalValue]) -> Vec<ValueType> {
    items.iter().map(|item| item.key.value_type).collect()
}

/// A reorg: an old value (V1, below the merge), its spend (W1), the rollback's restore (V2), the new spend
/// (W2). Whatever the merge grouping, the key must end deleted and V1 must not come back.
#[test]
fn restored_then_deleted_key_keeps_its_older_delete() {
    use ValueType::{Value as V, ValueOverWeakTombstone as P, WeakTombstone as W};
    // All three above V1 in one merge: the older weak tombstone carries on.
    assert_eq!(
        types(&merge(
            vec![version(4, W), version(3, V), version(2, W)],
            false
        )),
        [W]
    );
    // The restore meets the spend first: it keeps the pending weak tombstone...
    let merged = merge(vec![version(3, V), version(2, W)], false);
    assert_eq!(types(&merged), [P]);
    // ...which the new spend turns back into a weak tombstone, or resolves against V1.
    assert_eq!(
        types(&merge(vec![version(4, W), merged[0].clone()], false)),
        [W]
    );
    assert_eq!(
        types(&merge(vec![merged[0].clone(), version(1, V)], false)),
        [V]
    );
    // A key put twice, then deleted once, stays deleted; nothing pends at the last level.
    assert_eq!(
        types(&merge(
            vec![version(3, W), version(2, V), version(1, V)],
            false
        )),
        []
    );
    assert_eq!(types(&merge(vec![version(3, V), version(2, W)], true)), [V]);
}

/// Store histories as the indexer writes them (a key is put while absent, deleted while present, never deleted
/// and put again within one batch; e.g. outputs created, spent, restored by a rollback and spent again),
/// batched like `brk_store`'s pending map, with runs merged in arbitrary adjacent groups: every read must match a
/// plain map after every step.
#[test]
fn compaction_matches_a_map_for_any_merge_grouping() {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    let mut random = |bound: u64| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state % bound
    };

    for _ in 0..300 {
        let mut map = BTreeMap::<u8, u64>::new();
        // Runs, newest first; each holds at most one version per key, sorted by key.
        let mut runs: Vec<Vec<InternalValue>> = Vec::new();
        let mut seqno = 0;

        for _ in 0..40 {
            if !runs.is_empty() && random(3) == 0 {
                let len = runs.len() as u64;
                let from = random(len) as usize;
                let to = from + random(len - from as u64) as usize;
                let last_level = to == runs.len() - 1;
                let mut items: Vec<InternalValue> = runs.drain(from..=to).flatten().collect();
                items.sort_by(|a, b| a.key.cmp(&b.key));
                let merged = merge(items, last_level);
                if !merged.is_empty() {
                    runs.insert(from, merged);
                }
            } else {
                let (mut puts, mut dels) = (BTreeMap::<u8, u64>::new(), BTreeSet::<u8>::new());
                for _ in 0..=random(4) {
                    let key = random(4) as u8;
                    if map.remove(&key).is_some() {
                        if puts.remove(&key).is_none() {
                            dels.insert(key);
                        }
                    } else if !dels.contains(&key) {
                        seqno += 1;
                        map.insert(key, seqno);
                        puts.insert(key, seqno);
                    }
                }
                seqno += 1;
                let mut run: Vec<InternalValue> = puts
                    .iter()
                    .map(|(key, value)| {
                        InternalValue::from_components(
                            [*key],
                            value.to_be_bytes(),
                            seqno,
                            ValueType::Value,
                        )
                    })
                    .chain(
                        dels.iter()
                            .map(|key| InternalValue::new_weak_tombstone([*key], seqno)),
                    )
                    .collect();
                run.sort_by(|a, b| a.key.cmp(&b.key));
                if !run.is_empty() {
                    runs.insert(0, run);
                }
            }

            for key in 0..4u8 {
                let read = runs
                    .iter()
                    .find_map(|run| {
                        run.iter()
                            .find(|item| item.key.user_key == [key].as_slice())
                    })
                    .filter(|item| !item.key.value_type.is_tombstone())
                    .map(|item| u64::from_be_bytes(item.value.as_ref().try_into().unwrap()));
                assert_eq!(read, map.get(&key).copied(), "key {key}, runs {runs:?}");
            }
        }
    }
}
