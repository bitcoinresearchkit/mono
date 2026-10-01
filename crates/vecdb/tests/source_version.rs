use tempfile::TempDir;
#[cfg(feature = "pco")]
use vecdb::PcoVec;
use vecdb::{AnyVec, BytesVec, Database, EagerVec, ImportableVec, Result, StoredVec, Version};

fn provenance<V: StoredVec<I = usize, T = u64>>() -> Result<()> {
    let directory = TempDir::new()?;
    let schema = Version::new(17);
    let dependencies = Version::new(101);
    {
        let db = Database::open(directory.path())?;
        let mut source: V = V::import(&db, "source", schema)?;
        assert_eq!(source.version(), source.header().vec_version());
        let reader = source.read_only_clone();
        source.validate_computed_version_or_reset(dependencies)?;
        source.push(3);
        source.write()?;
        let computed = source.header().vec_version() + dependencies;
        assert_eq!(source.version(), computed);
        assert_eq!(reader.version(), computed);
        let eager: EagerVec<V> = EagerVec::import(&db, "source", schema)?;
        assert_eq!(eager.version(), computed);
        db.flush()?;
    }
    let db = Database::open(directory.path())?;
    let source: V = V::import(&db, "source", schema)?;
    assert_eq!(
        source.version(),
        source.header().vec_version() + dependencies
    );
    assert_eq!(source.collect(), [3]);
    Ok(())
}

#[test]
fn raw_computed_sources_share_provenance_and_reopen_it() -> Result<()> {
    provenance::<BytesVec<usize, u64>>()
}

#[cfg(feature = "pco")]
#[test]
fn compressed_computed_sources_share_provenance_and_reopen_it() -> Result<()> {
    provenance::<PcoVec<usize, u64>>()
}
