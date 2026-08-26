use vecdb::{AnyStoredVec, Database, ImportableVec, Version, WritableVec};

macro_rules! range_cursor_test {
    ($module:ident, $vec:ty) => {
        mod $module {
            use super::*;

            #[test]
            fn reads_and_skips_with_rw_and_ro_views() {
                let dir = tempfile::tempdir().unwrap();
                let db = Database::open(dir.path()).unwrap();
                let mut vec: $vec = <$vec>::forced_import(&db, "values", Version::ONE).unwrap();
                let values: Vec<_> = (0..20_000)
                    .map(|index| (index as u64).wrapping_mul(37))
                    .collect();
                for &value in &values {
                    vec.push(value);
                }
                vec.write().unwrap();

                let mut cursor = vec.range_cursor_at(2_000, 18_000);
                assert_eq!(cursor.position(), 2_000);
                assert_eq!(cursor.remaining(), 16_000);
                cursor.advance(31);
                assert_eq!(cursor.next(), Some(values[2_031]));
                let sum = cursor.fold(4_000, 0_u64, u64::wrapping_add);
                assert_eq!(sum, values[2_032..6_032].iter().copied().sum::<u64>());

                let read_only = vec.read_only_clone();
                let mut cursor = read_only.range_cursor_at(17_990, usize::MAX);
                let mut tail = Vec::new();
                cursor.for_each(usize::MAX, |value| tail.push(value));
                assert_eq!(tail, values[17_990..]);
                assert_eq!(cursor.remaining(), 0);
                assert_eq!(cursor.next(), None);
            }
        }
    };
}

#[cfg(feature = "pco")]
range_cursor_test!(pco, vecdb::PcoVec<usize, u64>);

#[cfg(feature = "lz4")]
range_cursor_test!(lz4, vecdb::LZ4Vec<usize, u64>);

#[cfg(feature = "zstd")]
range_cursor_test!(zstd, vecdb::ZstdVec<usize, u64>);
