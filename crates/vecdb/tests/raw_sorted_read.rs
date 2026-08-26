use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, BytesVec, Database, EagerVec, ImportableVec, MutableVec, ReadableVec, StoredVec,
    Version, WritableVec,
};

#[test]
fn raw_sorted_reads_include_pushed_values_and_preserve_duplicates() {
    let dir = tempdir().unwrap();
    let db = Database::open(dir.path()).unwrap();
    let mut values: EagerVec<BytesVec<usize, u64>> =
        EagerVec::import(&db, "values", Version::ONE).unwrap();

    for value in 0..5 {
        values.push(value);
    }
    values.write().unwrap();
    for value in 5..8 {
        values.push(value);
    }

    let indices = [0, 2, 2, 4, 5, 7, 8];
    let mut output = vec![99];
    values.read_sorted_into_at(&indices, &mut output);
    assert_eq!(output, [99, 0, 2, 2, 4, 5, 7]);

    values.write().unwrap();
    let read_only = values.read_only_clone();
    assert_eq!(read_only.read_sorted_at(&indices), [0, 2, 2, 4, 5, 7]);
}

#[test]
fn mutable_raw_sorted_reads_preserve_indices_across_updates_and_holes() {
    let dir = tempdir().unwrap();
    let db = Database::open(dir.path()).unwrap();
    let mut values: MutableVec<BytesVec<usize, u64>> =
        MutableVec::import(&db, "values", Version::ONE).unwrap();

    for value in 0..8 {
        values.push(value);
    }
    values.write().unwrap();
    values.update_at(2, 20).unwrap();
    values.delete_at(4);

    let indices = [0, 2, 2, 4, 5, 7, 8];
    assert_eq!(values.read_sorted_at(&indices), [0, 20, 20, 5, 7]);

    values.write().unwrap();
    let read_only = values.read_only_clone();
    assert_eq!(read_only.read_sorted_at(&indices), [0, 20, 20, 5, 7]);
}
