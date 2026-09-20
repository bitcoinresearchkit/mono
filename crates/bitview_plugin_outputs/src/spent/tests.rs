use brk_types::{Height, TxInIndex, Version};
use tempfile::tempdir;
use vecdb::{AnyStoredVec, Database, ReadableVec, Stamp, WritableVec};

use super::{compute::checkpoint, forced_import};

#[test]
fn update_checkpoints_preserve_spends_across_reopen_and_rollback() {
    let directory = tempdir().unwrap();
    {
        let db = Database::open(directory.path()).unwrap();
        let mut vecs = forced_import(&db, Version::ONE).unwrap();
        assert_eq!(vecs.txin_index.saved_stamped_changes(), 10);
        vecs.txin_index.push(TxInIndex::UNSPENT);
        vecs.txin_index.push(TxInIndex::UNSPENT);
        checkpoint(&mut vecs, Height::ZERO).unwrap();
        assert_eq!(vecs.txin_index.stamp(), Stamp::new(1));
        vecs.txin_index
            .update_at(0, TxInIndex::from(42usize))
            .unwrap();
        vecs.txin_index.push(TxInIndex::UNSPENT);
        checkpoint(&mut vecs, Height::new(10_000)).unwrap();
        vecs.txin_index
            .update_at(1, TxInIndex::from(43usize))
            .unwrap();
        checkpoint(&mut vecs, Height::new(20_000)).unwrap();
    }
    let db = Database::open(directory.path()).unwrap();
    let mut vecs = forced_import(&db, Version::ONE).unwrap();
    assert_eq!(
        vecs.txin_index.collect(),
        [
            TxInIndex::from(42usize),
            TxInIndex::from(43usize),
            TxInIndex::UNSPENT
        ]
    );
    let stamp = vecs.txin_index.rollback_before(Stamp::new(10_001)).unwrap();
    assert_eq!(stamp, Stamp::new(1));
    assert_eq!(vecs.txin_index.collect(), [TxInIndex::UNSPENT; 2]);
    // Recompute a shorter fork, with a different spent output.
    vecs.txin_index
        .update_at(1, TxInIndex::from(44usize))
        .unwrap();
    checkpoint(&mut vecs, Height::new(2)).unwrap();
    drop((vecs, db));
    let db = Database::open(directory.path()).unwrap();
    let vecs = forced_import(&db, Version::ONE).unwrap();
    assert_eq!(
        vecs.txin_index.collect(),
        [TxInIndex::UNSPENT, TxInIndex::from(44usize)]
    );
    assert_eq!(vecs.txin_index.stamp(), Stamp::new(3));
}

#[test]
fn catch_up_batches_skip_undo_history_and_final_update_saves_it() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut vecs = forced_import(&db, Version::ONE).unwrap();
    for height in [10_000, 20_000] {
        vecs.txin_index.push(TxInIndex::UNSPENT);
        vecs.txin_index
            .stamped_write_maybe_with_changes(Stamp::new(height + 1), false)
            .unwrap();
        vecs.txin_index.flush().unwrap();
    }
    assert!(!directory.path().join("changes").exists());
    vecs.txin_index
        .update_at(0, TxInIndex::from(42usize))
        .unwrap();
    checkpoint(&mut vecs, Height::new(20_001)).unwrap();
    assert_eq!(vecs.txin_index.find_rollback_files().unwrap().len(), 1);
    drop((vecs, db));
    let db = Database::open(directory.path()).unwrap();
    let mut vecs = forced_import(&db, Version::ONE).unwrap();
    assert_eq!(
        vecs.txin_index.rollback_before(Stamp::new(20_002)).unwrap(),
        Stamp::new(20_001)
    );
    assert_eq!(vecs.txin_index.collect(), [TxInIndex::UNSPENT; 2]);
}
