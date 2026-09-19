use std::{fs, iter};

use crate::{Database, PAGE_SIZE, Result};

use super::setup_test_db;

#[test]
fn ordered_batches_handle_empty_single_and_overlapping_writes() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let region = db.create_region_if_needed("batch")?;
    region.write(b"......")?;
    db.flush()?;

    region.batch_write_ordered(iter::empty::<(usize, String)>(), 2, |_, _| {
        panic!("an empty batch must not call the writer")
    });
    assert_eq!(db.flush()?, 0);

    for (values, expected) in [
        (vec![(4, "xy")], b"....xy"),
        (vec![(0, "ab"), (0, "cd"), (1, "ef")], b"cef.xy"),
    ] {
        let mut written = 0;
        let count = values.len();
        region.batch_write_ordered(
            values
                .into_iter()
                .map(|(offset, value)| (offset, value.to_owned())),
            2,
            |value, bytes| {
                bytes.copy_from_slice(value.as_bytes());
                written += 1;
            },
        );
        assert_eq!(written, count);
        assert_eq!(db.flush()?, 1);
        assert_eq!(db.flush()?, 0);
        assert_eq!(region.create_reader().read_all(), expected);
    }
    Ok(())
}

#[test]
fn test_reserve_region_capacity_preserves_data() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("first")?;
    region.write(b"preserved")?;
    let blocker = db.create_region_if_needed("blocker")?;
    let initial_start = region.meta().start();

    region.reserve_capacity(PAGE_SIZE * 3 + 1)?;

    let meta = region.meta();
    assert_ne!(meta.start(), initial_start);
    assert_eq!(meta.len(), b"preserved".len());
    assert_eq!(meta.reserved(), PAGE_SIZE * 4);
    drop(meta);
    assert_eq!(region.create_reader().read_all(), b"preserved");
    assert_eq!(blocker.meta().start(), PAGE_SIZE);

    db.flush()?;
    drop(blocker);
    drop(region);
    drop(db);

    let reopened = Database::open(_temp.path())?;
    let region = reopened.get_region("first").unwrap();
    assert_eq!(region.meta().reserved(), PAGE_SIZE * 4);
    assert_eq!(region.create_reader().read_all(), b"preserved");

    Ok(())
}

#[test]
fn test_write_to_region_within_reserved() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    let data = b"Hello, World!";

    region.write(data)?;

    // Verify data was written
    let meta = region.meta();
    assert_eq!(meta.len(), data.len());
    assert_eq!(meta.reserved(), PAGE_SIZE);
    let start = meta.start();
    drop(meta);

    let mmap = fs::read(db.path().join("data"))?;
    assert_eq!(&mmap[start..start + data.len()], data);

    Ok(())
}

#[test]
fn test_write_append() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;

    region.write(b"Hello")?;
    region.write(b", World!")?;

    let meta = region.meta();
    assert_eq!(meta.len(), 13);
    let start = meta.start();
    drop(meta);

    let mmap = fs::read(db.path().join("data"))?;
    assert_eq!(&mmap[start..(start + 13)], b"Hello, World!");

    Ok(())
}

#[test]
fn test_write_at_position() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;

    region.write(b"Hello, World!")?;
    region.write_at(b"Rust!", 7)?;

    let meta = region.meta();
    let start = meta.start();
    drop(meta);

    let mmap = fs::read(db.path().join("data"))?;
    assert_eq!(&mmap[start..(start + 13)], b"Hello, Rust!!");

    Ok(())
}

#[test]
fn test_write_exceeds_reserved() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;

    // Write more than PAGE_SIZE to trigger expansion
    let large_data = vec![1u8; PAGE_SIZE + 100];
    region.write(&large_data)?;

    let meta = region.meta();
    assert_eq!(meta.len(), large_data.len());
    assert!(meta.reserved() >= PAGE_SIZE * 2);

    Ok(())
}

#[test]
fn test_truncate_region() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;

    region.write(b"Hello, World!")?;

    let meta_before = region.meta();
    assert_eq!(meta_before.len(), 13);
    drop(meta_before);

    region.truncate(5)?;

    let meta_after = region.meta();
    assert_eq!(meta_after.len(), 5);

    Ok(())
}

#[test]
fn test_truncate_errors() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    region.write(b"Hello")?;

    // Truncating beyond length should error
    let result = region.truncate(10);
    assert!(result.is_err());

    // Truncating to same length should be OK
    let result = region.truncate(5);
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_large_write() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("large")?;

    // Write 1MB of data
    let large_data = vec![42u8; 1024 * 1024];
    region.write(&large_data)?;

    let meta = region.meta();
    assert_eq!(meta.len(), large_data.len());
    let start = meta.start();
    drop(meta);

    // Verify data
    let mmap = fs::read(db.path().join("data"))?;
    assert_eq!(&mmap[start..(start + large_data.len())], &large_data[..]);

    Ok(())
}

#[test]
fn test_truncate_write() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;

    region.write(b"Hello, World!")?;

    let meta_before = region.meta();
    assert_eq!(meta_before.len(), 13);
    drop(meta_before);

    // Truncate write - should set length to exactly the written data
    region.truncate_write(7, b"Rust")?;

    let meta_after = region.meta();
    assert_eq!(meta_after.len(), 11); // 7 + 4
    let start = meta_after.start();
    drop(meta_after);

    let mmap = fs::read(db.path().join("data"))?;
    assert_eq!(&mmap[start..(start + 11)], b"Hello, Rust");

    Ok(())
}

#[test]
fn test_write_at_invalid_position() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    region.write(b"Hello")?;

    // Writing beyond length should fail
    let result = region.write_at(b"World", 10);
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_empty_region_operations() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("empty")?;

    // Reading empty region
    let reader = region.create_reader();
    assert_eq!(reader.read_all(), b"");
    drop(reader);

    // Truncating empty region to 0 should work
    region.truncate(0)?;

    let meta = region.meta();
    assert_eq!(meta.len(), 0);

    Ok(())
}

#[test]
fn test_region_growth_patterns() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("growing")?;

    // Grow gradually
    for i in 0..10 {
        let data = vec![i as u8; 1000];
        region.write(&data)?;
    }

    let meta = region.meta();
    assert_eq!(meta.len(), 10_000);

    // Verify all data
    let reader = region.create_reader();
    let all_data = reader.read_all();
    for i in 0..10 {
        let chunk = &all_data[i * 1000..(i + 1) * 1000];
        assert!(chunk.iter().all(|&b| b == i as u8));
    }

    Ok(())
}

#[test]
fn test_write_at_boundary_conditions() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("boundary")?;

    // Write at start
    region.write(b"0123456789")?;

    // Write at exact length boundary
    region.write_at(b"ABC", 10)?;

    // Write at position 0
    region.write_at(b"X", 0)?;

    let reader = region.create_reader();
    assert_eq!(reader.read_all(), b"X123456789ABC");

    Ok(())
}

#[test]
fn test_mixed_size_writes() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("mixed")?;

    // Write various sizes
    region.write(b"tiny")?;
    region.write(&[1u8; 100])?;
    region.write(&[2u8; 1000])?;
    region.write(&[3u8; 10000])?;

    let meta = region.meta();
    assert_eq!(meta.len(), 4 + 100 + 1000 + 10000);

    // Verify each section
    let reader = region.create_reader();
    assert_eq!(&reader.read(0, 4), b"tiny");
    assert!(reader.read(4, 100).iter().all(|&b| b == 1));
    assert!(reader.read(104, 1000).iter().all(|&b| b == 2));
    assert!(reader.read(1104, 10000).iter().all(|&b| b == 3));

    Ok(())
}

#[test]
fn test_zero_byte_writes() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("empty_writes")?;

    // Write zero bytes
    region.write(b"")?;

    let meta = region.meta();
    assert_eq!(meta.len(), 0);
    drop(meta);

    // Write some data, then write zero bytes again
    region.write(b"Hello")?;
    region.write(b"")?;

    let meta = region.meta();
    assert_eq!(meta.len(), 5);

    Ok(())
}

#[test]
fn test_alternating_write_and_truncate() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("oscillating")?;

    for cycle in 0..10 {
        // Grow
        let data = vec![cycle as u8; 1000];
        region.write(&data)?;

        let meta = region.meta();
        let expected_len = if cycle == 0 { 1000 } else { 100 + 1000 };
        assert_eq!(meta.len(), expected_len);
        drop(meta);

        // Shrink
        region.truncate(100)?;

        let meta = region.meta();
        assert_eq!(meta.len(), 100);
        drop(meta);
    }

    Ok(())
}

#[test]
fn test_partial_overwrites_data_integrity() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("partial")?;

    // Write initial pattern
    let initial = b"AAAAAAAAAA";
    region.write(initial)?;

    // Overwrite middle
    region.write_at(b"BBB", 3)?;

    // Overwrite start
    region.write_at(b"CC", 0)?;

    // Overwrite end
    region.write_at(b"DD", 8)?;

    let reader = region.create_reader();
    assert_eq!(reader.read_all(), b"CCABBBAADD");

    Ok(())
}

#[test]
fn test_write_at_exact_reserved_boundary() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("boundary")?;

    // Fill exactly to PAGE_SIZE
    let data = vec![42u8; PAGE_SIZE];
    region.write(&data)?;

    let meta = region.meta();
    assert_eq!(meta.len(), PAGE_SIZE);
    assert_eq!(meta.reserved(), PAGE_SIZE);
    drop(meta);

    // Writing one more byte should trigger expansion
    region.write(b"X")?;

    let meta = region.meta();
    assert_eq!(meta.len(), PAGE_SIZE + 1);
    assert!(meta.reserved() > PAGE_SIZE);

    Ok(())
}
