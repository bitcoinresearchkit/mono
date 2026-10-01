use statedb::{Amount, Creations, History, Spends, State, View};
use std::{
    fs::{self, OpenOptions},
    io::{Error, Result, Write},
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
fn reconstructs_every_height_zero_value_outputs_and_same_block_spends() {
    let root = tempdir().unwrap();
    let mut spends = Spends::open(root.path()).unwrap();
    let mut created = Creations::open(root.path()).unwrap();
    let mut history = History::open(root.path()).unwrap();
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
    let view = View::open(root.path(), root.path(), root.path()).unwrap();
    let reader = view.reader().unwrap();
    for h in 0..100 {
        assert_eq!(reader.state_at(h + 1).unwrap().amounts(), expected[h]);
        assert_eq!(
            history
                .reader(&spends, &created)
                .and_then(|reader| reader.state_at(h + 1))
                .unwrap()
                .amounts(),
            expected[h]
        );
    }
    drop((history, spends, created));
    let spends = Spends::open(root.path()).unwrap();
    let created = Creations::open(root.path()).unwrap();
    let history = History::open(root.path()).unwrap();
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
fn view_reopens_published_state_after_legacy_origins_split() -> Result<()> {
    let source = tempdir()?;
    let origins = source.path().join("origins");

    let mut spends = Spends::open(&origins)?;
    let mut creations = Creations::open(&origins)?;
    let mut history = History::open(&origins)?;
    spends.push(block(0), [])?;
    creations.push(block(0), amount(10, 1), None)?;
    spends.push(block(1), [(0, amount(10, 1))])?;
    creations.push(block(1), amount(5, 1), None)?;
    spends.commit()?;
    creations.commit()?;
    history.advance(0, 2, &spends, &creations, |_, _| Ok(()))?;
    drop((history, spends, creations));

    let files = [
        ("spends/data", "inputs"),
        ("spends/index", "inputs"),
        ("spends/commit", "inputs"),
        ("creations/data", "outputs"),
        ("creations/index", "outputs"),
        ("creations/commit", "outputs"),
        ("snapshots/data", "utxo_history"),
        ("snapshots/pages", "utxo_history"),
    ];
    let original = files
        .iter()
        .map(|(path, _)| fs::read(origins.join(path)))
        .collect::<Result<Vec<_>>>()?;

    let destination = tempdir()?;
    let plugins = destination.path().join("plugins");
    for (subdir, owner) in [
        ("spends", "inputs"),
        ("creations", "outputs"),
        ("snapshots", "utxo_history"),
    ] {
        let owner_path = plugins.join(owner);
        fs::create_dir_all(&owner_path)?;
        fs::rename(origins.join(subdir), owner_path.join(subdir))?;
    }

    let inputs = plugins.join("inputs");
    let outputs = plugins.join("outputs");
    let utxo_history = plugins.join("utxo_history");
    let spends = Spends::open(&inputs)?;
    let creations = Creations::open(&outputs)?;
    let history = History::open(&utxo_history)?;
    assert_eq!(
        history.reader(&spends, &creations)?.state_at(2)?.total(),
        amount(5, 1)
    );
    drop((history, spends, creations));

    let view = View::open(&utxo_history, &inputs, &outputs)?;
    assert_eq!(view.reader()?.state_at(2)?.total(), amount(5, 1));
    assert_eq!(
        files
            .iter()
            .map(|(path, owner)| fs::read(plugins.join(owner).join(path)))
            .collect::<Result<Vec<_>>>()?,
        original
    );
    Ok(())
}

#[test]
fn unpublished_tails_are_discarded_but_committed_corruption_is_an_error() {
    let root = tempdir().unwrap();
    let mut spends = Spends::open(root.path()).unwrap();
    spends.push(block(0), []).unwrap();
    spends.commit().unwrap();
    spends.push(block(1), []).unwrap();
    drop(spends);
    let spends = Spends::open(root.path()).unwrap();
    assert_eq!(spends.len(), 1);
    drop(spends);
    OpenOptions::new()
        .write(true)
        .open(root.path().join("spends/data"))
        .unwrap()
        .set_len(1)
        .unwrap();
    assert!(Spends::open(root.path()).is_err());
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
            .chunks_exact(12)
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
fn corrections_are_not_actual_spends_and_counts_survive_zero_sats() {
    let root = tempdir().unwrap();
    let mut spends = Spends::open(root.path()).unwrap();
    let mut created = Creations::open(root.path()).unwrap();
    let mut history = History::open(root.path()).unwrap();
    spends.push(block(0), []).unwrap();
    created.push(block(0), amount(50, 1), None).unwrap();
    spends.push(block(1), []).unwrap();
    created
        .push(block(1), amount(0, 3), Some((0, amount(50, 1))))
        .unwrap();
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
        amount(0, 3)
    );
    let reader = history.reader(&spends, &created).unwrap();
    let mut state = reader.state_at(0).unwrap();
    let mut cursor = reader.cursor(&mut state).unwrap();
    for h in 0..2 {
        let diff = cursor.advance().unwrap().unwrap();
        assert_eq!(diff.spent().len(), 0);
        let expected: &[(u32, Amount)] = if h == 1 { &[(0, amount(50, 1))] } else { &[] };
        assert_eq!(diff.removed().collect::<Vec<_>>(), expected);
    }
}

#[test]
fn invalid_spends_do_not_publish_partial_state() {
    let root = tempdir().unwrap();
    let mut spends = Spends::open(root.path()).unwrap();
    let mut created = Creations::open(root.path()).unwrap();
    let mut history = History::open(root.path()).unwrap();
    spends.push(block(0), []).unwrap();
    created.push(block(0), amount(1, 1), None).unwrap();
    spends.commit().unwrap();
    created.commit().unwrap();
    history
        .advance(0, 1, &spends, &created, |_, _| Ok(()))
        .unwrap();
    spends
        .push(block(1), [(0, amount(1, 1)), (0, amount(1, 1))])
        .unwrap();
    created.push(block(1), amount(5, 1), None).unwrap();
    spends.commit().unwrap();
    created.commit().unwrap();
    let mut state = history
        .reader(&spends, &created)
        .and_then(|reader| reader.state_at(1))
        .unwrap();
    assert!(
        history
            .reader(&spends, &created)
            .unwrap()
            .replay(&mut state, 2, |_, _| Ok(()))
            .is_err()
    );
    assert_eq!(state.amounts(), [amount(1, 1)]);
    assert!(
        history
            .advance(1, 2, &spends, &created, |_, _| Ok(()))
            .is_err()
    );
    assert_eq!(history.reader(&spends, &created).unwrap().len(), 1);
}

#[test]
fn seeded_history_has_no_invented_prefix_and_source_versions_invalidate_snapshots() {
    let root = tempdir().unwrap();
    let mut spends = Spends::open(root.path()).unwrap();
    let mut created = Creations::open(root.path()).unwrap();
    let mut history = History::open(root.path()).unwrap();
    spends.seed(3).unwrap();
    created.seed(3).unwrap();
    spends.validate_version(7).unwrap();
    created.validate_version(8).unwrap();
    history
        .seed(
            &State::new(vec![amount(10, 1); 3], block(2)).unwrap(),
            &spends,
            &created,
        )
        .unwrap();
    assert!(
        history
            .reader(&spends, &created)
            .and_then(|reader| reader.state_at(2))
            .is_err()
    );
    spends.push(block(3), []).unwrap();
    created.push(block(3), amount(2, 1), None).unwrap();
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
        amount(32, 4)
    );
    spends.validate_version(9).unwrap();
    assert_eq!(spends.start(), 0);
    assert_eq!(history.reader(&spends, &created).unwrap().len(), 0);
    assert!(
        history
            .reader(&spends, &created)
            .and_then(|reader| reader.state_at(4))
            .is_err()
    );
}

#[test]
fn one_writer_per_column_and_snapshot_truncation_rejected() {
    let root = tempdir().unwrap();
    let mut spends = Spends::open(root.path()).unwrap();
    assert!(Spends::open(root.path()).is_err());
    let mut created = Creations::open(root.path()).unwrap();
    let mut history = History::open(root.path()).unwrap();
    assert!(History::open(root.path()).is_err());
    spends.push(block(0), []).unwrap();
    created.push(block(0), amount(1, 1), None).unwrap();
    spends.commit().unwrap();
    created.commit().unwrap();
    history
        .advance(0, 1, &spends, &created, |_, _| Ok(()))
        .unwrap();
    let path = root.path().join("snapshots/data");
    let mut bytes = fs::read(&path).unwrap();
    bytes.pop();
    fs::write(&path, bytes).unwrap();
    assert!(
        history
            .reader(&spends, &created)
            .and_then(|reader| reader.state_at(1))
            .is_err()
    );
}

#[test]
fn pending_truncate_and_extra_index_bytes_recover_to_a_valid_commit() {
    let root = tempdir().unwrap();
    let mut spends = Spends::open(root.path()).unwrap();
    for h in 0..4 {
        spends.push(block(h), []).unwrap();
    }
    spends.truncate(2).unwrap();
    drop(spends);
    OpenOptions::new()
        .append(true)
        .open(root.path().join("spends/index"))
        .unwrap()
        .write_all(&[123; 5])
        .unwrap();
    let spends = Spends::open(root.path()).unwrap();
    assert_eq!(spends.len(), 2);
    assert_eq!(spends.hash(1).unwrap(), block(1));
    assert_eq!(
        fs::metadata(root.path().join("spends/index"))
            .unwrap()
            .len(),
        16
    );
}

#[test]
fn missing_commit_does_not_erase_existing_records() {
    let root = tempdir().unwrap();
    let mut s = Spends::open(root.path()).unwrap();
    s.push(block(0), []).unwrap();
    s.commit().unwrap();
    drop(s);
    let path = root.path().join("spends/data");
    let expected = fs::read(&path).unwrap();
    fs::remove_file(root.path().join("spends/commit")).unwrap();
    assert!(Spends::open(root.path()).is_err());
    assert_eq!(fs::read(path).unwrap(), expected);
}

#[test]
fn failed_commit_requires_reopening_instead_of_appending_to_a_partial_transaction() {
    let root = tempdir().unwrap();
    let mut s = Spends::open(root.path()).unwrap();
    s.push(block(0), []).unwrap();
    // Make the atomic manifest's temporary path unwritable as a file.
    fs::create_dir(root.path().join("spends/commit.next")).unwrap();
    assert!(s.commit().is_err());
    assert!(s.push(block(1), []).is_err());
    assert!(s.commit().is_err());
    drop(s);
    fs::remove_dir(root.path().join("spends/commit.next")).unwrap();
    let s = Spends::open(root.path()).unwrap();
    assert_eq!(s.len(), 0);
}

#[test]
fn rejected_rows_do_not_append_a_partial_record() {
    let root = tempdir().unwrap();
    let mut spends = Spends::open(root.path()).unwrap();
    assert!(
        spends
            .push(block(0), [(0, amount(1, 1)), (1, amount(2, 1))])
            .is_err()
    );
    assert_eq!(spends.len(), 0);
    assert!(
        spends
            .push(
                block(0),
                [(0, amount(1, 1)), (0, amount(0, u64::from(u32::MAX) + 1))],
            )
            .is_err()
    );
    assert_eq!(spends.len(), 0);
    let removed = amount(3, u64::from(u32::MAX));
    spends.push(block(0), [(0, removed)]).unwrap();
    spends.commit().unwrap();
    let mut created = Creations::open(root.path()).unwrap();
    created.push(block(0), removed, None).unwrap();
    created.commit().unwrap();
    let mut history = History::open(root.path()).unwrap();
    history
        .advance(0, 1, &spends, &created, |_, total| {
            assert_eq!(total, Amount::default());
            Ok(())
        })
        .unwrap();
}

#[test]
fn reader_hides_unpublished_blocks_and_replays_the_same_diffs_as_snapshots() -> Result<()> {
    let root = tempdir()?;
    let mut spends = Spends::open(root.path())?;
    let mut creations = Creations::open(root.path())?;
    let mut history = History::open(root.path())?;
    for h in 0..3 {
        spends.push(block(h), [])?;
        creations.push(block(h), amount(10, 2), None)?;
    }
    spends.commit()?;
    creations.commit()?;
    history.advance(0, 2, &spends, &creations, |_, _| Ok(()))?;
    let reader = history.reader(&spends, &creations)?;
    assert_eq!(reader.len(), 2);
    assert!(reader.state_at(3).is_err());
    let mut state = reader.state_at(0)?;
    let mut cursor = reader.cursor(&mut state)?;
    for h in 0..2 {
        let diff = cursor.advance()?.unwrap();
        assert_eq!(diff.hash, block(h));
        assert_eq!(diff.created, amount(10, 2));
        assert_eq!(diff.spent().len(), 0);
    }
    assert!(cursor.advance()?.is_none());
    let mut state = reader.state_at(0)?;
    reader.replay(&mut state, 2, |_, _| Ok(()))?;
    assert_eq!(state.amounts(), reader.state_at(2)?.amounts());
    Ok(())
}

#[test]
fn reader_rejects_a_stale_publication_after_producer_rewind_or_replacement() -> Result<()> {
    let root = tempdir()?;
    let mut spends = Spends::open(root.path())?;
    let mut creations = Creations::open(root.path())?;
    let mut history = History::open(root.path())?;
    spends.push(block(0), [])?;
    creations.push(block(0), amount(10, 1), None)?;
    spends.commit()?;
    creations.commit()?;
    history.advance(0, 1, &spends, &creations, |_, _| Ok(()))?;
    spends.truncate(0)?;
    assert!(history.reader(&spends, &creations).is_err());
    spends.push(block(100), [])?;
    spends.commit()?;
    assert!(history.reader(&spends, &creations).is_err());
    creations.truncate(0)?;
    creations.push(block(100), amount(7, 1), None)?;
    creations.commit()?;
    // Even matching replacement producers are not the previously published chain.
    assert!(history.reader(&spends, &creations).is_err());
    history.advance(0, 1, &spends, &creations, |_, _| Ok(()))?;
    assert_eq!(
        history.reader(&spends, &creations)?.state_at(1)?.total(),
        amount(7, 1)
    );
    Ok(())
}

#[test]
fn cursor_reuses_canonical_state_and_rejects_another_chain() -> Result<()> {
    let root = tempdir()?;
    let mut spends = Spends::open(root.path())?;
    let mut creations = Creations::open(root.path())?;
    let mut history = History::open(root.path())?;
    spends.push(block(0), [])?;
    creations.push(block(0), amount(10, 3), None)?;
    spends.push(block(1), [(0, amount(10, 1))])?;
    creations.push(block(1), amount(0, 1), None)?;
    spends.commit()?;
    creations.commit()?;
    history.advance(0, 2, &spends, &creations, |_, _| Ok(()))?;
    let reader = history.reader(&spends, &creations)?;
    let mut stale = State::new(vec![amount(10, 3)], block(99))?;
    assert!(reader.cursor(&mut stale).is_err());
    assert!(reader.replay(&mut stale, 2, |_, _| Ok(())).is_err());
    let mut state = reader.state_at(0)?;
    let mut cursor = reader.cursor(&mut state)?;
    assert_eq!(cursor.advance()?.unwrap().created, amount(10, 3));
    assert_eq!(cursor.state().total(), amount(10, 3));
    assert_eq!(
        cursor.advance()?.unwrap().spent().collect::<Vec<_>>(),
        [(0, amount(10, 1))]
    );
    assert_eq!(cursor.state().amounts(), [amount(0, 2), amount(0, 1)]);
    assert!(cursor.advance()?.is_none());
    assert_eq!(state.amounts(), reader.state_at(2)?.amounts());
    Ok(())
}

#[test]
fn read_only_view_does_not_lock_or_truncate_unpublished_files() -> Result<()> {
    let root = tempdir()?;
    let mut spends = Spends::open(root.path())?;
    let mut creations = Creations::open(root.path())?;
    let mut history = History::open(root.path())?;
    spends.push(block(0), [])?;
    creations.push(block(0), amount(10, 2), None)?;
    spends.commit()?;
    creations.commit()?;
    history.advance(0, 1, &spends, &creations, |_, _| Ok(()))?;
    let path = root.path().join("spends/data");
    OpenOptions::new()
        .append(true)
        .open(&path)?
        .write_all(&[1, 2, 3])?;
    let len = fs::metadata(&path)?.len();
    let view = View::open(root.path(), root.path(), root.path())?;
    let reader = view.reader()?;
    assert_eq!(reader.len(), 1);
    assert_eq!(reader.state_at(1)?.total(), amount(10, 2));
    assert!(reader.state_at(2).is_err());
    assert_eq!(fs::metadata(&path)?.len(), len);
    assert!(Spends::open(root.path()).is_err());
    let missing = root.path().join("missing");
    assert!(View::open(&missing, root.path(), root.path()).is_err());
    assert!(!missing.exists());
    Ok(())
}

#[test]
fn warm_append_noop_and_failed_visit_recover_from_published_prefix() {
    let root = tempdir().unwrap();
    let mut spends = Spends::open(root.path()).unwrap();
    let mut created = Creations::open(root.path()).unwrap();
    let mut history = History::open(root.path()).unwrap();
    for h in 0..4 {
        spends.push(block(h), []).unwrap();
        created.push(block(h), amount(10, 1), None).unwrap();
    }
    spends.commit().unwrap();
    created.commit().unwrap();
    history
        .advance(0, 2, &spends, &created, |_, _| Ok(()))
        .unwrap();
    let snapshot = root.path().join("snapshots/data");
    let before = fs::metadata(&snapshot).unwrap().modified().unwrap();
    history
        .advance(2, 2, &spends, &created, |_, _| {
            panic!("no-op visited a block")
        })
        .unwrap();
    assert_eq!(before, fs::metadata(&snapshot).unwrap().modified().unwrap());
    assert!(
        history
            .advance(2, 4, &spends, &created, |_, _| Err(Error::other(
                "visit failed"
            )))
            .is_err()
    );
    assert_eq!(history.reader(&spends, &created).unwrap().len(), 2);
    let mut visits = Vec::new();
    history
        .advance(2, 4, &spends, &created, |h, v| {
            visits.push((h, v));
            Ok(())
        })
        .unwrap();
    assert_eq!(visits, [(2, amount(30, 3)), (3, amount(40, 4))]);
    assert_eq!(
        history
            .reader(&spends, &created)
            .unwrap()
            .state_at(4)
            .unwrap()
            .total(),
        amount(40, 4)
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
fn seed_remains_reconstructible_after_rewind_and_append() {
    let root = tempdir().unwrap();
    let mut spends = Spends::open(root.path()).unwrap();
    let mut created = Creations::open(root.path()).unwrap();
    spends.seed(3).unwrap();
    created.seed(3).unwrap();
    let mut history = History::open(root.path()).unwrap();
    history
        .seed(
            &State::new(vec![amount(10, 1); 3], block(2)).unwrap(),
            &spends,
            &created,
        )
        .unwrap();
    for h in 3..8 {
        spends.push(block(h), []).unwrap();
        created.push(block(h), amount(10, 1), None).unwrap();
    }
    spends.commit().unwrap();
    created.commit().unwrap();
    history
        .advance(3, 8, &spends, &created, |_, _| Ok(()))
        .unwrap();
    history
        .advance(3, 3, &spends, &created, |_, _| Ok(()))
        .unwrap();
    history
        .advance(3, 8, &spends, &created, |_, _| Ok(()))
        .unwrap();
    assert_eq!(
        history
            .reader(&spends, &created)
            .unwrap()
            .state_at(3)
            .unwrap()
            .total(),
        amount(30, 3)
    );
    assert_eq!(
        View::open(root.path(), root.path(), root.path())
            .unwrap()
            .reader()
            .unwrap()
            .state_at(3)
            .unwrap()
            .total(),
        amount(30, 3)
    );
}

#[test]
fn corrupt_tip_retention_flag_is_rejected() {
    let root = tempdir().unwrap();
    let mut spends = Spends::open(root.path()).unwrap();
    let mut created = Creations::open(root.path()).unwrap();
    let mut history = History::open(root.path()).unwrap();
    spends.push(block(0), []).unwrap();
    created.push(block(0), amount(10, 1), None).unwrap();
    spends.commit().unwrap();
    created.commit().unwrap();
    history
        .advance(0, 1, &spends, &created, |_, _| Ok(()))
        .unwrap();
    drop((history, spends, created));
    let path = root.path().join("snapshots/pages");
    let mut bytes = fs::read(&path).unwrap();
    bytes[8] = 2;
    fs::write(&path, bytes).unwrap();
    assert!(History::open(root.path()).is_err());
    assert!(View::open(root.path(), root.path(), root.path()).is_err());
}

#[test]
fn malformed_totals_and_rows_roll_back_a_partially_applied_block() -> Result<()> {
    let root = tempdir()?;
    let mut spends = Spends::open(root.path())?;
    let mut created = Creations::open(root.path())?;
    spends.push(block(0), [])?;
    created.push(block(0), amount(20, 3), None)?;
    spends.push(block(1), [(0, amount(5, 1)), (0, amount(5, 1))])?;
    created.push(block(1), amount(0, 1), None)?;
    spends.commit()?;
    created.commit()?;
    let mut history = History::open(root.path())?;
    history.advance(0, 2, &spends, &created, |_, _| Ok(()))?;
    let path = root.path().join("spends/data");
    let original = fs::read(&path)?;
    for count_error in [false, true] {
        let mut bytes = original.clone();
        if count_error {
            bytes[116..120].copy_from_slice(&0u32.to_le_bytes());
        } else {
            bytes[80..88].copy_from_slice(&15u64.to_le_bytes());
        }
        fs::write(&path, bytes)?;
        let reader = history.reader(&spends, &created)?;
        let mut state = reader.state_at(1)?;
        let mut cursor = reader.cursor(&mut state)?;
        assert!(cursor.advance().is_err());
        assert_eq!(state.amounts(), [amount(20, 3)]);
        assert_eq!(state.total(), amount(20, 3));
        assert_eq!(state.hash(), block(0));
    }
    Ok(())
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
#[test]
fn empty_commit_preserves_metadata_while_seed_and_version_changes_publish() -> Result<()> {
    let root = tempdir()?;
    let mut spends = Spends::open(root.path())?;
    spends.seed(700000)?;
    spends.validate_version(42)?;
    let path = root.path().join("spends/commit");
    let before = fs::metadata(&path)?.modified()?;
    spends.commit()?;
    assert_eq!(before, fs::metadata(&path)?.modified()?);
    drop(spends);
    let spends = Spends::open(root.path())?;
    assert_eq!(
        (spends.start(), spends.len(), spends.version()),
        (700000, 700000, 42)
    );
    Ok(())
}
