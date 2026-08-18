#![cfg(feature = "pco")]

use rawdb::Database;
use tempfile::TempDir;
use vecdb::{
    BytesVec, ImportableVec, PcoVec, READ_CHUNK_SIZE, ReadableVec, Result, StoredVec, Version,
};

#[test]
fn compressed_cursor_chunks_are_page_aligned() -> Result<()> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;

    let raw = BytesVec::<usize, u8>::import(&db, "raw", Version::ONE)?;
    assert_eq!(raw.cursor_chunk_size(), READ_CHUNK_SIZE);

    let pco_u8 = PcoVec::<usize, u8>::import(&db, "pco_u8", Version::ONE)?;
    assert_eq!(pco_u8.cursor_chunk_size(), 16 * 1024);
    assert_eq!(pco_u8.read_only_clone().cursor_chunk_size(), 16 * 1024);

    let pco_u64 = PcoVec::<usize, u64>::import(&db, "pco_u64", Version::ONE)?;
    let u64s_per_page = 16 * 1024 / size_of::<u64>();
    assert_eq!(pco_u64.cursor_chunk_size(), u64s_per_page);
    assert_eq!(pco_u64.read_only_clone().cursor_chunk_size(), u64s_per_page);

    Ok(())
}
