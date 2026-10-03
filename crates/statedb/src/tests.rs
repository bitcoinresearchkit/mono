use crate::{Amount, Creations, History, Spends, View};
use std::{
    fs::{self},
    io::{Error, Result},
};
use tempfile::tempdir;

fn amount(sats: u64, count: u64) -> Amount {
    Amount { sats, count }
}
fn block(h: usize) -> [u8; 32] {
    let mut hash = [0; 32];
    hash[..8].copy_from_slice(&(h as u64).to_le_bytes());
    hash
}

#[test]
fn separate_producers_reconstruct_every_height_zero_value_outputs_and_same_block_spends() {
    let root = tempdir().unwrap();
    let inputs = root.path().join("inputs");
    let outputs = root.path().join("outputs");
    let snapshots = root.path().join("utxo_history");
    let mut spends = Spends::open(&inputs).unwrap();
    let mut created = Creations::open(&outputs).unwrap();
    let mut history = History::open(&snapshots).unwrap();
    history.set_snapshot_interval(7).unwrap();
    let mut expected = Vec::<Vec<Amount>>::new();
    let mut state = Vec::<Amount>::new();
    for h in 0..100 {
        state.push(amount(100, 2));
        let mut rows = vec![(h as u32, amount(0, 1))];
        state[h].count -= 1;
        if h > 0 {
            rows.push(((h - 1) as u32, amount(100, 1)));
            state[h - 1] = Amount::default();
        }
        spends.push(block(h), rows).unwrap();
        created.push(block(h), amount(100, 2), None).unwrap();
        expected.push(state.clone());
    }
    spends.commit().unwrap();
    created.commit().unwrap();
    history
        .advance(0, 100, &spends, &created, |_, _| Ok(()))
        .unwrap();
    let files = [
        (&inputs, "spends/data"),
        (&inputs, "spends/index"),
        (&inputs, "spends/commit"),
        (&outputs, "creations/data"),
        (&outputs, "creations/index"),
        (&outputs, "creations/commit"),
        (&snapshots, "snapshots/data"),
        (&snapshots, "snapshots/pages"),
    ]
    .map(|(owner, file)| owner.join(file));
    let original = files
        .iter()
        .map(fs::read)
        .collect::<Result<Vec<_>>>()
        .unwrap();
    let view = View::open(&snapshots, &inputs, &outputs).unwrap();
    let reader = view.reader().unwrap();
    for (h, expected) in expected.iter().enumerate() {
        assert_eq!(reader.state_at(h + 1).unwrap().amounts(), *expected);
        assert_eq!(
            history
                .reader(&spends, &created)
                .and_then(|reader| reader.state_at(h + 1))
                .unwrap()
                .amounts(),
            *expected
        );
    }
    assert_eq!(
        files
            .iter()
            .map(fs::read)
            .collect::<Result<Vec<_>>>()
            .unwrap(),
        original
    );
    drop((history, spends, created));
    let spends = Spends::open(&inputs).unwrap();
    let created = Creations::open(&outputs).unwrap();
    let history = History::open(&snapshots).unwrap();
    assert_eq!(
        history
            .reader(&spends, &created)
            .and_then(|reader| reader.state_at(100))
            .unwrap()
            .amounts(),
        expected[99]
    );
}

#[test]
fn reorg_rebuilds_from_ancestor_and_rejects_mixed_producer_chains() {
    let root = tempdir().unwrap();
    let mut spends = Spends::open(root.path()).unwrap();
    let mut created = Creations::open(root.path()).unwrap();
    let mut history = History::open(root.path()).unwrap();
    history.set_snapshot_interval(2).unwrap();
    for h in 0..8 {
        spends.push(block(h), []).unwrap();
        created.push(block(h), amount(10, 1), None).unwrap();
    }
    spends.commit().unwrap();
    created.commit().unwrap();
    history
        .advance(0, 8, &spends, &created, |_, _| Ok(()))
        .unwrap();
    spends.truncate(3).unwrap();
    created.truncate(3).unwrap();
    spends.push(block(1003), [(0, amount(10, 1))]).unwrap();
    created.push(block(1003), amount(7, 1), None).unwrap();
    spends.commit().unwrap();
    created.commit().unwrap();
    history
        .advance(3, 4, &spends, &created, |_, _| Ok(()))
        .unwrap();
    assert_eq!(
        history
            .reader(&spends, &created)
            .and_then(|reader| reader.state_at(4))
            .unwrap()
            .total(),
        amount(27, 3)
    );
    let pages = fs::read(root.path().join("snapshots/pages")).unwrap();
    assert!(
        !pages[9..]
            .as_chunks::<12>()
            .0
            .iter()
            .any(|row| u32::from_le_bytes(row[..4].try_into().unwrap()) == 8)
    );
    assert!(
        history
            .reader(&spends, &created)
            .and_then(|reader| reader.state_at(5))
            .is_err()
    );
    spends.push(block(4), []).unwrap();
    created.push(block(1004), amount(1, 1), None).unwrap();
    spends.commit().unwrap();
    created.commit().unwrap();
    assert!(
        history
            .advance(4, 5, &spends, &created, |_, _| Ok(()))
            .is_err()
    );
    assert_eq!(history.reader(&spends, &created).unwrap().len(), 4);
}

#[test]
fn source_versions_invalidate_snapshots() {
    let root = tempdir().unwrap();
    let mut spends = Spends::open(root.path()).unwrap();
    let mut created = Creations::open(root.path()).unwrap();
    let mut history = History::open(root.path()).unwrap();
    spends.validate_version(7).unwrap();
    created.validate_version(8).unwrap();
    for height in 0..2 {
        spends.push(block(height), []).unwrap();
        created.push(block(height), amount(10, 1), None).unwrap();
    }
    spends.commit().unwrap();
    created.commit().unwrap();
    history
        .advance(0, 2, &spends, &created, |_, _| Ok(()))
        .unwrap();
    assert_eq!(
        history
            .reader(&spends, &created)
            .and_then(|reader| reader.state_at(2))
            .unwrap()
            .total(),
        amount(20, 2)
    );
    spends.validate_version(9).unwrap();
    assert_eq!(history.reader(&spends, &created).unwrap().len(), 0);
    assert!(
        history
            .reader(&spends, &created)
            .and_then(|reader| reader.state_at(2))
            .is_err()
    );
}

#[test]
fn replaces_tip_without_accumulating_old_snapshots_and_can_rewind_to_periodic_tip() {
    let root = tempdir().unwrap();
    let mut spends = Spends::open(root.path()).unwrap();
    let mut created = Creations::open(root.path()).unwrap();
    let mut history = History::open(root.path()).unwrap();
    history.set_snapshot_interval(10).unwrap();
    for h in 0..29 {
        spends.push(block(h), []).unwrap();
        created.push(block(h), amount(10, 1), None).unwrap();
    }
    spends.commit().unwrap();
    created.commit().unwrap();
    for end in 1..30 {
        history
            .advance(end - 1, end, &spends, &created, |_, _| Ok(()))
            .unwrap();
        let size = fs::metadata(root.path().join("snapshots/data"))
            .unwrap()
            .len();
        assert!(size < 2000, "obsolete tips accumulated: {size}");
        let view = View::open(root.path(), root.path(), root.path()).unwrap();
        let state = view.reader().unwrap().state_at(end).unwrap();
        assert_eq!(state.total(), amount(end as u64 * 10, end as u64));
    }
    spends.truncate(10).unwrap();
    created.truncate(10).unwrap();
    history
        .advance(10, 10, &spends, &created, |_, _| Ok(()))
        .unwrap();
    assert_eq!(history.reader(&spends, &created).unwrap().len(), 10);
    assert_eq!(
        View::open(root.path(), root.path(), root.path())
            .unwrap()
            .reader()
            .unwrap()
            .state_at(10)
            .unwrap()
            .total(),
        amount(100, 10)
    );
}

#[test]
fn empty_rewind_reopens_and_accepts_a_replacement_chain() -> Result<()> {
    let root = tempdir()?;
    let mut spends = Spends::open(root.path())?;
    let mut created = Creations::open(root.path())?;
    let mut history = History::open(root.path())?;
    spends.push(block(0), [])?;
    created.push(block(0), amount(10, 1), None)?;
    spends.commit()?;
    created.commit()?;
    history.advance(0, 1, &spends, &created, |_, _| Ok(()))?;
    spends.truncate(0)?;
    created.truncate(0)?;
    history.advance(0, 0, &spends, &created, |_, _| Ok(()))?;
    let empty = View::open(root.path(), root.path(), root.path())?
        .reader()?
        .state_at(0)?;
    assert!(empty.is_empty());
    assert_eq!(empty.total(), Amount::default());
    spends.push(block(99), [])?;
    created.push(block(99), amount(12, 1), None)?;
    spends.commit()?;
    created.commit()?;
    history.advance(0, 1, &spends, &created, |_, _| Ok(()))?;
    assert_eq!(
        View::open(root.path(), root.path(), root.path())?
            .reader()?
            .state_at(1)?
            .total(),
        amount(12, 1)
    );
    Ok(())
}

#[test]
fn a_failed_visit_after_a_checkpoint_leaves_a_reopenable_published_prefix() -> Result<()> {
    let root = tempdir()?;
    let mut spends = Spends::open(root.path())?;
    let mut created = Creations::open(root.path())?;
    let mut history = History::open(root.path())?;
    history.set_snapshot_interval(3)?;
    for h in 0..8 {
        spends.push(block(h), [])?;
        created.push(block(h), amount(10, 1), None)?;
    }
    spends.commit()?;
    created.commit()?;
    assert!(
        history
            .advance(0, 8, &spends, &created, |h, _| if h == 5 {
                Err(Error::other("visit failed"))
            } else {
                Ok(())
            })
            .is_err()
    );
    assert_eq!(history.reader(&spends, &created)?.len(), 3);
    assert_eq!(
        View::open(root.path(), root.path(), root.path())?
            .reader()?
            .state_at(3)?
            .total(),
        amount(30, 3)
    );
    history.advance(3, 8, &spends, &created, |_, _| Ok(()))?;
    assert_eq!(
        View::open(root.path(), root.path(), root.path())?
            .reader()?
            .state_at(8)?
            .total(),
        amount(80, 8)
    );
    Ok(())
}
