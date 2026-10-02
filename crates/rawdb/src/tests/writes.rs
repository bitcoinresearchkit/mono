use crate::{Database, PAGE_SIZE, Result};

use super::setup_test_db;

#[test]
fn truncating_relocation_preserves_prefix_and_neighbor_on_reopen() -> Result<()> {
    for prefix in [0, 17, PAGE_SIZE] {
        let (db, dir) = setup_test_db()?;
        let region = db.create_region_if_needed("rewrite")?;
        region.write(&vec![3; 2 * PAGE_SIZE])?;
        let old_start = region.meta().start();
        let neighbor = db.create_region_if_needed("neighbor")?;
        neighbor.write(b"kept")?;
        db.flush()?;

        region.truncate_write(prefix, &vec![7; 3 * PAGE_SIZE - prefix])?;
        assert_ne!(region.meta().start(), old_start);
        assert_eq!(db.flush()?, 1);
        drop(region);
        drop(neighbor);
        drop(db);

        let db = Database::open(dir.path())?;
        let region = db.get_region("rewrite").unwrap();
        let reader = region.create_reader();
        assert_eq!(reader.len(), 3 * PAGE_SIZE);
        assert!(reader.read_all()[..prefix].iter().all(|&byte| byte == 3));
        assert!(reader.read_all()[prefix..].iter().all(|&byte| byte == 7));
        assert_eq!(
            db.get_region("neighbor")
                .unwrap()
                .create_reader()
                .read_all(),
            b"kept"
        );
    }
    Ok(())
}

#[test]
fn batch_dirty_span_covers_every_middle_write() -> Result<()> {
    let (db, temp) = setup_test_db()?;
    let region = db.create_region_if_needed("batch")?;
    region.write(&[0; 16])?;
    db.flush()?;
    // Bounds safety and dirty tracking do not depend on interior ordering.
    region.batch_write_ordered(
        [(0, 1u8), (8, 2), (4, 3), (12, 4)].into_iter(),
        1,
        |value, bytes| bytes[0] = *value,
    );
    assert_eq!(db.flush()?, 1);
    drop(region);
    drop(db);
    let db = Database::open(temp.path())?;
    let mut expected = [0; 16];
    for (offset, value) in [(0, 1), (8, 2), (4, 3), (12, 4)] {
        expected[offset] = value;
    }
    assert_eq!(
        db.get_region("batch").unwrap().create_reader().read_all(),
        expected
    );
    Ok(())
}
