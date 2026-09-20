use std::fs;

use crate::{PAGE_SIZE, Result};

use super::setup_test_db;

#[test]
fn test_complex_region_lifecycle() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Create multiple regions
    let r1 = db.create_region_if_needed("region1")?;
    let r2 = db.create_region_if_needed("region2")?;
    let r3 = db.create_region_if_needed("region3")?;

    // Write to all regions
    r1.write(b"Data for region 1")?;
    r2.write(b"Data for region 2")?;
    r3.write(b"Data for region 3")?;

    // Remove middle region
    r2.remove()?;
    db.flush()?; // Make hole available

    // Verify hole exists
    {
        let layout = db.layout();
        assert_eq!(layout.start_to_hole().len(), 1);
    }

    // Create new region that should reuse the hole
    let r4 = db.create_region_if_needed("region4")?;
    r4.write(b"Fills the hole")?;

    // Verify hole was filled
    {
        let layout = db.layout();
        assert_eq!(layout.start_to_hole().len(), 0);
    }

    // Write large data to trigger region movement (overwrite from start)
    let large = vec![42u8; PAGE_SIZE * 3];
    r4.write_at(&large, 0)?;
    db.flush()?; // Make hole available

    // Verify r4 moved and created a hole
    {
        let layout = db.layout();
        assert!(!layout.start_to_hole().is_empty());
    }

    // Verify all data is still correct
    {
        let reader = r1.create_reader();
        assert_eq!(reader.read_all(), b"Data for region 1");
        drop(reader);
    }

    {
        let reader = r3.create_reader();
        assert_eq!(reader.read_all(), b"Data for region 3");
        drop(reader);
    }

    {
        let reader = r4.create_reader();
        assert_eq!(reader.read_all(), &large[..]);
        drop(reader);
    }

    Ok(())
}

#[test]
fn test_interleaved_operations() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let r1 = db.create_region_if_needed("r1")?;
    let r2 = db.create_region_if_needed("r2")?;
    let r3 = db.create_region_if_needed("r3")?;

    // Interleave writes
    r1.write(b"Start1")?;
    r2.write(b"Start2")?;
    r3.write(b"Start3")?;

    r1.write(b" More1")?;
    r2.write(b" More2")?;

    // Truncate one
    r3.truncate(3)?;

    // Continue writing
    r1.write(b" End1")?;
    r2.write_at(b"X", 0)?;

    // Verify results
    {
        let reader = r1.create_reader();
        assert_eq!(reader.read_all(), b"Start1 More1 End1");
        drop(reader);
    }

    {
        let reader = r2.create_reader();
        assert_eq!(reader.read_all(), b"Xtart2 More2");
        drop(reader);
    }

    {
        let meta = r3.meta();
        assert_eq!(meta.len(), 3);
    }

    Ok(())
}

#[test]
fn test_comprehensive_db_operations() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region1 = db.create_region_if_needed("region1")?;

    {
        let layout = db.layout();

        assert!(layout.start_to_region().len() == 1);

        assert!(layout.start_to_hole().is_empty());

        let regions = db.regions();

        assert!(regions.get("region1").is_some_and(|r| r.ptr_eq(&region1)));

        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.is_empty());
        assert!(region1_meta.reserved() == PAGE_SIZE);
    }

    region1.write(&[0, 1, 2, 3, 4])?;

    {
        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 5);
        assert!(region1_meta.reserved() == PAGE_SIZE);
        assert!(fs::read(db.path().join("data"))?[0..10] == [0, 1, 2, 3, 4, 0, 0, 0, 0, 0]);
    }

    region1.write(&[5, 6, 7, 8, 9])?;

    {
        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 10);
        assert!(region1_meta.reserved() == PAGE_SIZE);
        assert!(fs::read(db.path().join("data"))?[0..10] == [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    region1.write_at(&[1, 2], 0)?;

    {
        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 10);
        assert!(region1_meta.reserved() == PAGE_SIZE);
        assert!(fs::read(db.path().join("data"))?[0..10] == [1, 2, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    region1.write_at(&[10, 11, 12, 13, 14, 15, 16, 17, 18], 4)?;

    {
        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 13);
        assert!(region1_meta.reserved() == PAGE_SIZE);
        assert!(
            fs::read(db.path().join("data"))?[0..20]
                == [
                    1, 2, 2, 3, 10, 11, 12, 13, 14, 15, 16, 17, 18, 0, 0, 0, 0, 0, 0, 0
                ]
        );
    }

    region1.write_at(&[0, 0, 0, 0, 0, 1], 13)?;

    {
        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 19);
        assert!(region1_meta.reserved() == PAGE_SIZE);
        assert!(
            fs::read(db.path().join("data"))?[0..20]
                == [
                    1, 2, 2, 3, 10, 11, 12, 13, 14, 15, 16, 17, 18, 0, 0, 0, 0, 0, 1, 0
                ]
        );
    }

    region1.write_at(&[1; 8000], 0)?;

    {
        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 8000);
        assert!(region1_meta.reserved() == PAGE_SIZE * 2);
        assert!(fs::read(db.path().join("data"))?[0..8000] == [1; 8000]);
        assert!(fs::read(db.path().join("data"))?[8000..8001] == [0]);
    }

    db.flush()?;

    // Repeating truncation and compaction must preserve the same page boundary.
    for _ in 0..2 {
        region1.truncate(10)?;
        db.compact()?;
        {
            let region1_meta = region1.meta();
            assert_eq!(region1_meta.start(), 0);
            assert_eq!(region1_meta.len(), 10);
            assert_eq!(region1_meta.reserved(), PAGE_SIZE * 2);
            // Only whole pages are punched; the preceding byte stays intact.
            assert_eq!(fs::read(db.path().join("data"))?[4095..=4096], [1, 0]);
        }
        db.flush()?;
    }

    region1.remove()?;
    db.compact()?;

    {
        let regions = db.regions();
        assert_eq!(regions.len(), 0);

        let layout = db.layout();
        assert!(layout.start_to_region().is_empty());
        assert!(layout.start_to_hole().len() == 1);
    }

    let region1 = db.create_region_if_needed("region1")?;
    let region2 = db.create_region_if_needed("region2")?;
    let region3 = db.create_region_if_needed("region3")?;

    {
        let regions = db.regions();

        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.is_empty());
        assert!(region1_meta.reserved() == PAGE_SIZE);

        let region2_meta = region2.meta();
        assert!(region2_meta.start() == PAGE_SIZE);
        assert!(region2_meta.is_empty());
        assert!(region2_meta.reserved() == PAGE_SIZE);

        let region3_meta = region3.meta();
        assert!(region3_meta.start() == PAGE_SIZE * 2);
        assert!(region3_meta.is_empty());
        assert!(region3_meta.reserved() == PAGE_SIZE);

        assert!(regions.len() == 3);
        assert!(regions.get("region1").map(|region| region.index()) == Some(0));
        assert!(regions.get("region2").map(|region| region.index()) == Some(1));
        assert!(regions.get("region3").map(|region| region.index()) == Some(2));

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 3);

        assert!(start_to_index.get(&0).unwrap().ptr_eq(&region1));
        assert!(start_to_index.get(&PAGE_SIZE).unwrap().ptr_eq(&region2));
        assert!(
            start_to_index
                .get(&(PAGE_SIZE * 2))
                .unwrap()
                .ptr_eq(&region3)
        );
        assert!(layout.start_to_hole().is_empty());
    }

    region2.remove()?;
    db.compact()?;

    {
        let regions = db.regions();

        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.is_empty());
        assert!(region1_meta.reserved() == PAGE_SIZE);

        let region3_meta = region3.meta();
        assert!(region3_meta.start() == PAGE_SIZE * 2);
        assert!(region3_meta.is_empty());
        assert!(region3_meta.reserved() == PAGE_SIZE);
        assert!(regions.len() == 2);
        assert!(regions.get("region1").map(|region| region.index()) == Some(0));
        assert!(regions.get("region2").is_none());
        assert!(regions.get("region3").map(|region| region.index()) == Some(2));

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 2);
        assert!(start_to_index.get(&0).unwrap().ptr_eq(&region1));
        assert!(
            start_to_index
                .get(&(PAGE_SIZE * 2))
                .unwrap()
                .ptr_eq(&region3)
        );
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.len() == 1);
        assert!(start_to_hole.get(&PAGE_SIZE) == Some(&PAGE_SIZE));
    }

    let region2 = db.create_region_if_needed("region2")?;
    let region2_i = region2.index();
    assert!(region2_i == 1);

    region2.remove()?;
    db.compact()?;

    {
        let regions = db.regions();

        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.is_empty());
        assert!(region1_meta.reserved() == PAGE_SIZE);

        let region3_meta = region3.meta();
        assert!(region3_meta.start() == PAGE_SIZE * 2);
        assert!(region3_meta.is_empty());
        assert!(region3_meta.reserved() == PAGE_SIZE);

        assert!(regions.len() == 2);
        assert!(regions.get("region1").map(|region| region.index()) == Some(0));
        assert!(regions.get("region2").is_none());
        assert!(regions.get("region3").map(|region| region.index()) == Some(2));

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 2);
        assert!(start_to_index.get(&0).unwrap().ptr_eq(&region1));
        assert!(
            start_to_index
                .get(&(PAGE_SIZE * 2))
                .unwrap()
                .ptr_eq(&region3)
        );

        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.len() == 1);
        assert!(start_to_hole.get(&PAGE_SIZE) == Some(&PAGE_SIZE));
    }

    region1.write_at(&[1; 8000], 0)?;

    {
        let regions = db.regions();

        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 8000);
        assert!(region1_meta.reserved() == 2 * PAGE_SIZE);

        let region3_meta = region3.meta();
        assert!(region3_meta.start() == PAGE_SIZE * 2);
        assert!(region3_meta.is_empty());
        assert!(region3_meta.reserved() == PAGE_SIZE);
        assert!(regions.len() == 2);
        assert!(regions.get("region1").map(|region| region.index()) == Some(0));
        assert!(regions.get("region2").is_none());
        assert!(regions.get("region3").map(|region| region.index()) == Some(2));

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 2);
        assert!(start_to_index.get(&0).unwrap().ptr_eq(&region1));
        assert!(
            start_to_index
                .get(&(PAGE_SIZE * 2))
                .unwrap()
                .ptr_eq(&region3)
        );
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.is_empty());
    }

    let region2 = db.create_region_if_needed("region2")?;

    {
        let regions = db.regions();

        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 8000);
        assert!(region1_meta.reserved() == 2 * PAGE_SIZE);

        let region2_meta = region2.meta();
        assert!(region2_meta.start() == PAGE_SIZE * 3);
        assert!(region2_meta.is_empty());
        assert!(region2_meta.reserved() == PAGE_SIZE);

        let region3_meta = region3.meta();
        assert!(region3_meta.start() == PAGE_SIZE * 2);
        assert!(region3_meta.is_empty());
        assert!(region3_meta.reserved() == PAGE_SIZE);
        assert!(regions.len() == 3);
        assert!(regions.get("region1").map(|region| region.index()) == Some(0));
        assert!(regions.get("region2").map(|region| region.index()) == Some(1));
        assert!(regions.get("region3").map(|region| region.index()) == Some(2));

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 3);
        assert!(start_to_index.get(&0).unwrap().ptr_eq(&region1));
        assert!(
            start_to_index
                .get(&(PAGE_SIZE * 2))
                .unwrap()
                .ptr_eq(&region3)
        );
        assert!(
            start_to_index
                .get(&(PAGE_SIZE * 3))
                .unwrap()
                .ptr_eq(&region2)
        );
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.is_empty());
    }

    region3.remove()?;
    db.compact()?;

    {
        let regions = db.regions();

        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 8000);
        assert!(region1_meta.reserved() == 2 * PAGE_SIZE);

        let region2_meta = region2.meta();
        assert!(region2_meta.start() == PAGE_SIZE * 3);
        assert!(region2_meta.is_empty());
        assert!(region2_meta.reserved() == PAGE_SIZE);

        assert!(regions.len() == 2);
        assert!(regions.get("region1").map(|region| region.index()) == Some(0));
        assert!(regions.get("region2").map(|region| region.index()) == Some(1));
        assert!(regions.get("region3").is_none());

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 2);
        assert!(start_to_index.get(&0).unwrap().ptr_eq(&region1));
        assert!(
            start_to_index
                .get(&(PAGE_SIZE * 3))
                .unwrap()
                .ptr_eq(&region2)
        );
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.get(&(PAGE_SIZE * 2)) == Some(&PAGE_SIZE));
    }

    region1.write(&[1; 8000])?;
    db.compact()?;

    {
        let regions = db.regions();

        let region1_meta = region1.meta();
        assert!(region1_meta.start() == PAGE_SIZE * 4);
        assert!(region1_meta.len() == 16_000);
        assert!(region1_meta.reserved() == 4 * PAGE_SIZE);

        let region2_meta = region2.meta();
        assert!(region2_meta.start() == PAGE_SIZE * 3);
        assert!(region2_meta.is_empty());
        assert!(region2_meta.reserved() == PAGE_SIZE);

        assert!(regions.len() == 2);
        assert!(regions.get("region1").map(|region| region.index()) == Some(0));
        assert!(regions.get("region2").map(|region| region.index()) == Some(1));
        assert!(regions.get("region3").is_none());

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 2);
        assert!(
            start_to_index
                .get(&(PAGE_SIZE * 4))
                .unwrap()
                .ptr_eq(&region1)
        );
        assert!(
            start_to_index
                .get(&(PAGE_SIZE * 3))
                .unwrap()
                .ptr_eq(&region2)
        );
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.get(&0) == Some(&(PAGE_SIZE * 3)));
    }

    region2.write(&[1; 6000])?;

    let region4 = db.create_region_if_needed("region4")?;
    region2.remove()?;
    region4.remove()?;

    Ok(())
}
