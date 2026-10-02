use tempfile::tempdir;
use vecdb::{Database, PcoVec};

use super::*;

fn spending_positions(
    input_txs: &impl ReadableVec<TxInIndex, TxIndex>,
    first_inputs: &impl ReadableVec<TxIndex, TxInIndex>,
    requested: &mut [(TxInIndex, usize)],
    tx_bound: TxIndex,
) -> Result<Vec<(usize, TxIndex, Vin)>> {
    let ordered = requested.is_sorted_by_key(|&(input, _)| input);
    let mut positions = Vec::with_capacity(requested.len());
    visit_spending_positions(
        input_txs,
        first_inputs,
        requested,
        tx_bound,
        ordered,
        |output, tx, vin| {
            positions.push((output, tx, vin));
            Ok(())
        },
    )?;
    Ok(positions)
}

fn position_fixture(
    db: &Database,
    len: usize,
) -> (PcoVec<TxInIndex, TxIndex>, PcoVec<TxIndex, TxInIndex>) {
    use vecdb::{AnyStoredVec, ImportableVec, Version, WritableVec};
    let mut inputs = PcoVec::import(db, "inputs", Version::ONE).unwrap();
    let mut firsts = PcoVec::import(db, "firsts", Version::ONE).unwrap();
    for i in 0..len {
        inputs.push(TxIndex::from(i / 3));
    }
    for i in 0..len.div_ceil(3) {
        firsts.push(TxInIndex::from(i * 3));
    }
    inputs.write().unwrap();
    firsts.write().unwrap();
    (inputs, firsts)
}

#[test]
fn batched_spending_positions_keep_output_slots_and_safe_bounds() {
    let dir = tempdir().unwrap();
    let db = Database::open(dir.path()).unwrap();
    let (inputs, mut firsts) = position_fixture(&db, 32768);
    for requests in [
        vec![],
        vec![12usize],
        vec![32767],
        vec![32766, 0, 21, 22, 21, 5, 17000],
    ] {
        let mut requested: Vec<_> = requests
            .iter()
            .enumerate()
            .map(|(slot, &i)| (TxInIndex::from(i), slot))
            .collect();
        let mut actual =
            spending_positions(&inputs, &firsts, &mut requested, TxIndex::from(10000usize))
                .unwrap();
        actual.sort_unstable_by_key(|&(slot, _, _)| slot);
        let expected: Vec<_> = requests
            .iter()
            .enumerate()
            .filter(|&(_, &i)| i / 3 < 10000)
            .map(|(slot, &i)| (slot, TxIndex::from(i / 3), Vin::from(i % 3)))
            .collect();
        assert_eq!(actual, expected);
    }
    assert!(
        spending_positions(
            &inputs,
            &firsts,
            &mut [
                (TxInIndex::from(0usize), 0),
                (TxInIndex::from(99999usize), 1)
            ],
            TxIndex::from(10000usize)
        )
        .is_err()
    );
    use vecdb::WritableVec;
    firsts.truncate_if_needed_at(1).unwrap();
    assert!(
        spending_positions(
            &inputs,
            &firsts,
            &mut [(TxInIndex::from(6usize), 0), (TxInIndex::from(9usize), 1)],
            TxIndex::from(10000usize)
        )
        .is_err()
    );
}
