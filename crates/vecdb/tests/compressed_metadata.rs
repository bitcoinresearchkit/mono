#![cfg(feature = "pco")]

use tempfile::tempdir;
use vecdb::{AnyStoredVec, AnyVec, Database, Error, ImportableVec, PcoVec, Version, WritableVec};

#[test]
fn import_rejects_invalid_page_layouts() -> vecdb::Result<()> {
    const RAW_FLAG: u32 = 1 << 31;
    const BODY_MASK: u32 = (1 << 30) - 1;
    const VALUES_PER_PAGE: usize = 8 * 1024 / size_of::<u64>();

    {
        let temp = tempdir()?;
        let db = Database::open(temp.path())?;
        let mut vec = PcoVec::<usize, u64>::forced_import(&db, "raw_header", Version::ONE)?;
        vec.push(1);
        vec.write()?;
        let names = vec.region_names();
        drop(vec);

        let pages = db.get_region(&names[1]).expect("pages region");
        let mut bytes = pages.create_reader().read_all().to_vec();
        let body = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) & BODY_MASK;
        bytes[8..12].copy_from_slice(&(body + 1).to_le_bytes());
        pages.write_at(&bytes, 0)?;

        assert!(matches!(
            PcoVec::<usize, u64>::import(&db, "raw_header", Version::ONE),
            Err(Error::CorruptedRegion { .. })
        ));
    }

    {
        let temp = tempdir()?;
        let db = Database::open(temp.path())?;
        let mut vec = PcoVec::<usize, u64>::forced_import(&db, "two_raw", Version::ONE)?;
        for value in 0..=VALUES_PER_PAGE as u64 {
            vec.push(value);
        }
        vec.write()?;
        let names = vec.region_names();
        drop(vec);

        let pages = db.get_region(&names[1]).expect("pages region");
        let mut bytes = pages.create_reader().read_all().to_vec();
        let body_and_flags = u32::from_le_bytes(bytes[12..16].try_into().unwrap());
        let body = body_and_flags & BODY_MASK;
        bytes[8..12].copy_from_slice(&body.to_le_bytes());
        bytes[12..16].copy_from_slice(&(body_and_flags | RAW_FLAG).to_le_bytes());
        pages.write_at(&bytes, 0)?;

        assert!(matches!(
            PcoVec::<usize, u64>::import(&db, "two_raw", Version::ONE),
            Err(Error::CorruptedRegion { .. })
        ));
    }

    {
        let temp = tempdir()?;
        let db = Database::open(temp.path())?;
        let mut vec = PcoVec::<usize, u64>::forced_import(&db, "trailing", Version::ONE)?;
        vec.push(1);
        vec.write()?;
        let names = vec.region_names();
        drop(vec);

        db.get_region(&names[0]).expect("data region").write(&[0])?;

        assert!(matches!(
            PcoVec::<usize, u64>::import(&db, "trailing", Version::ONE),
            Err(Error::CorruptedRegion { .. })
        ));
    }

    Ok(())
}
