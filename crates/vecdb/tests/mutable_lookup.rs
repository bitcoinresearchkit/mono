use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, BytesVec, Database, ImportableVec, MutableVec, ReadableVec, StoredVec, Version,
    WritableVec,
};

fn check(source: &impl ReadableVec<usize, u64>, expected: &[Option<u64>]) {
    assert_eq!(
        source.collect(),
        expected.iter().copied().flatten().collect::<Vec<_>>()
    );
    for indices in [
        vec![],
        vec![0],
        vec![expected.len() + 1],
        (0..expected.len() + 3).collect(),
        (0..expected.len() + 3).flat_map(|i| [i, i]).collect(),
        (0..expected.len() + 3).step_by(173).collect(),
    ] {
        let mut actual = vec![123];
        source.read_sorted_into_at(&indices, &mut actual);
        let mut wanted = vec![123];
        wanted.extend(
            indices
                .iter()
                .filter_map(|&i| expected.get(i).copied().flatten()),
        );
        assert_eq!(actual, wanted);
    }
}

macro_rules! mutation_roundtrip {
    ($raw:ty) => {{
        type V = $raw;
        let dir = tempdir().unwrap();
        let db = Database::open(dir.path()).unwrap();
        let mut source = MutableVec::<V>::import(&db, "values", Version::ONE).unwrap();
        let mut expected: Vec<_> = (0..8192).map(|i| Some(i * 7)).collect();
        for value in &expected {
            source.push(value.unwrap());
        }
        source.write().unwrap();
        let reader = source.read_only_clone();
        for i in (0..expected.len()).step_by(17).chain([5, 6, 7]) {
            source.delete_at(i);
            expected[i] = None;
        }
        for i in (0..expected.len()).step_by(23) {
            source.update_at(i, 999).unwrap();
            expected[i] = Some(999);
        }
        for i in 0..99 {
            source.push(i);
            expected.push(Some(i));
        }
        source.delete_at(8200);
        expected[8200] = None;
        check(&source, &expected);
        check(&reader, &(0..8192).map(|i| Some(i * 7)).collect::<Vec<_>>());
        source.write().unwrap();
        check(&source, &expected);
        check(&reader, &expected);
        source.truncate_if_needed_at(4100).unwrap();
        expected.truncate(4100);
        source.update_at(4099, 456).unwrap();
        expected[4099] = Some(456);
        check(&source, &expected);
        source.write().unwrap();
        check(&reader, &expected);
        drop(reader);
        drop(source);
        drop(db);
        let db = Database::open(dir.path()).unwrap();
        let source = MutableVec::<V>::import(&db, "values", Version::ONE).unwrap();
        check(&source, &expected);
    }};
}

#[test]
fn bytes_sorted_mutations_keep_physical_indices_duplicates_and_publication() {
    mutation_roundtrip!(BytesVec<usize, u64>);
}
