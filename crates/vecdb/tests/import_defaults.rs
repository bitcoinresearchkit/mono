use tempfile::tempdir;
use vecdb::{
    BytesVec, Database, EagerVec, Error, HEADER_OFFSET, ImportOptions, MutableVec, Result, Stamp,
    StoredVec, Version,
};

fn roundtrip<V: StoredVec<I = usize, T = u32>>(resets_corrupted: bool) -> Result<()> {
    let directory = tempdir()?;
    {
        let db = Database::open(directory.path())?;
        let mut vec = V::import(&db, "values", Version::ONE)?;
        assert_eq!(vec.name(), "values");
        assert_eq!(vec.saved_stamped_changes(), 0);
        for value in 0..4097 {
            vec.push(value);
        }
        vec.stamped_write(Stamp::new(7))?;
        vec.flush()?;
    }
    let db = Database::open(directory.path())?;
    let vec = V::import(&db, "values", Version::ONE)?;
    assert_eq!(vec.collect(), (0..4097).collect::<Vec<_>>());
    assert_eq!(vec.stamp(), Stamp::new(7));
    drop(vec);
    assert!(matches!(
        V::import(&db, "values", Version::TWO),
        Err(Error::DifferentVersion { .. })
    ));
    let vec = V::forced_import(&db, "values", Version::ONE)?;
    assert_eq!(vec.len(), 4097, "matching forced import must retain data");
    drop(vec);
    let vec = V::forced_import(&db, "values", Version::TWO)?;
    assert!(vec.is_empty());
    assert_eq!(vec.stamp(), Stamp::default());
    drop(vec);
    let options = ImportOptions::new(&db, "values", Version::TWO)
        .with_saved_stamped_changes(3)
        .with_initial_capacity(0)
        .with_max_compression_chunk_size(8192);
    let vec = V::import_with(options)?;
    assert_eq!(vec.saved_stamped_changes(), 3);
    vec.region().truncate(HEADER_OFFSET - 1)?;
    drop(vec);
    assert!(matches!(
        V::import(&db, "values", Version::TWO),
        Err(Error::CorruptedRegion { .. })
    ));
    let forced = V::forced_import(&db, "values", Version::TWO);
    if resets_corrupted {
        assert!(forced?.is_empty());
    } else {
        assert!(matches!(forced, Err(Error::CorruptedRegion { .. })));
    }
    Ok(())
}

macro_rules! roundtrips {
    ($name:ident, $vec:ty, $resets_corrupted:expr) => {
        #[test]
        fn $name() -> vecdb::Result<()> {
            roundtrip::<$vec>($resets_corrupted)?;
            roundtrip::<EagerVec<$vec>>($resets_corrupted)
        }
    };
}

roundtrips!(bytes, BytesVec<usize, u32>, false);
roundtrips!(mutable_bytes, MutableVec<BytesVec<usize, u32>>, false);
#[cfg(feature = "zerocopy")]
roundtrips!(zerocopy, vecdb::ZeroCopyVec<usize, u32>, false);
#[cfg(feature = "zerocopy")]
roundtrips!(
    mutable_zerocopy,
    MutableVec<vecdb::ZeroCopyVec<usize, u32>>,
    false
);
#[cfg(feature = "pco")]
roundtrips!(pco, vecdb::PcoVec<usize, u32>, true);
#[cfg(feature = "lz4")]
roundtrips!(lz4, vecdb::LZ4Vec<usize, u32>, true);
#[cfg(feature = "zstd")]
roundtrips!(zstd, vecdb::ZstdVec<usize, u32>, true);
