use super::RealizedCaps;
use brk_error::Result;
use brk_types::{CentsSats, Version};
use tempfile::tempdir;
use vecdb::{Database, Stamp};

#[test]
fn exact_caps_reopen_rollback_and_source_version_reset() -> Result<()> {
    let dir = tempdir()?;
    let original = [
        CentsSats::new(u128::from(u64::MAX) + 123),
        CentsSats::new(789),
    ];
    {
        let db = Database::open(dir.path())?;
        let mut caps = RealizedCaps::<2>::import(&db, 10)?;
        assert!(caps.validate(Version::ONE)?);
        assert_eq!(caps.end(), 0);
        caps.save(original.into_iter(), Stamp::new(10), true)?;
        caps.save(
            [CentsSats::new(1), CentsSats::new(2)].into_iter(),
            Stamp::new(11),
            true,
        )?;
        db.flush()?;
    }
    let db = Database::open(dir.path())?;
    let mut caps = RealizedCaps::<2>::import(&db, 10)?;
    assert!(!caps.validate(Version::ONE)?);
    assert_eq!(caps.end(), 12);
    assert_eq!(caps.rollback_before(Stamp::new(11))?, Stamp::new(10));
    assert_eq!(caps.load(), Some(original));
    assert_eq!(caps.end(), 11);
    assert!(caps.validate(Version::TWO)?);
    assert_eq!(caps.end(), 0);
    assert!(caps.load().is_none());
    Ok(())
}
