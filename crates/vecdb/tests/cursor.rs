#![cfg(feature = "pco")]

use rawdb::Database;
use tempfile::TempDir;
use vecdb::{AnyStoredVec, ImportableVec, PcoVec, ReadableVec, StoredVec, Version, WritableVec};

#[test]
fn pco_u8_cursor_crosses_page_and_chunk_boundaries() -> vecdb::Result<()> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    let mut vec = PcoVec::<usize, u8>::import(&db, "u8", Version::ONE)?;

    for index in 0..20_000 {
        vec.push((index % 251) as u8);
    }
    vec.write()?;
    let read_only = vec.read_only_clone();

    for index in 20_000..20_010 {
        vec.push((index % 251) as u8);
    }

    let mut cursor = vec.cursor();
    for index in [0, 4_095, 4_096, 16_383, 16_384, 19_999, 20_009] {
        assert_eq!(cursor.get(index), Some((index % 251) as u8));
        assert_eq!(cursor.position(), 0);
    }
    assert_eq!(cursor.get(20_010), None);

    cursor.advance(16_380);
    for index in 16_380..16_390 {
        assert_eq!(cursor.next(), Some((index % 251) as u8));
    }

    let mut tail = Vec::new();
    cursor.advance(20_000 - cursor.position());
    cursor.for_each(usize::MAX, |value| tail.push(value));
    assert_eq!(
        tail,
        (20_000..20_010)
            .map(|index| (index % 251) as u8)
            .collect::<Vec<_>>()
    );

    let mut read_only_cursor = read_only.cursor();
    read_only_cursor.advance(16_380);
    let values = read_only_cursor.fold(20, Vec::new(), |mut values, value| {
        values.push(value);
        values
    });
    assert_eq!(
        values,
        (16_380..16_400)
            .map(|index| (index % 251) as u8)
            .collect::<Vec<_>>()
    );
    assert_eq!(read_only_cursor.get(20_000), None);

    Ok(())
}

#[test]
fn pco_u64_cursor_crosses_two_page_chunk_boundary() -> vecdb::Result<()> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    let mut vec = PcoVec::<usize, u64>::import(&db, "u64", Version::ONE)?;

    for index in 0..5_000 {
        vec.push((index as u64).wrapping_mul(37));
    }
    vec.write()?;

    let mut cursor = vec.cursor();
    cursor.advance(4_090);
    for index in 4_090..4_110 {
        assert_eq!(cursor.next(), Some((index as u64).wrapping_mul(37)));
    }
    assert_eq!(cursor.get(4_999), Some(4_999_u64 * 37));
    assert_eq!(cursor.get(5_000), None);

    Ok(())
}
