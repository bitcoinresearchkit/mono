use std::{fs, os::unix::fs::FileExt};

use brk_rpc::Auth;
use tempfile::tempdir;

use super::*;

fn reader(path: PathBuf) -> ReaderInner {
    let client = Client::new("http://127.0.0.1:1", Auth::None).unwrap();
    ReaderInner::new(path, &client)
}

fn contents(file: &File) -> [u8; 2] {
    let mut bytes = [0; 2];
    file.read_exact_at(&mut bytes, 0).unwrap();
    bytes
}

#[test]
fn cached_handles_are_bounded_without_invalidating_active_readers() {
    let directory = tempdir().unwrap();
    for index in 0..=MAX_CACHED_FILES as u16 {
        fs::write(
            directory.path().join(format!("blk{index:05}.dat")),
            index.to_le_bytes(),
        )
        .unwrap();
    }
    let reader = reader(directory.path().to_owned());
    let first = reader.open_blk(0).unwrap();
    let lifetime = Arc::downgrade(&first);
    for index in 1..=MAX_CACHED_FILES as u16 {
        let file = reader.open_blk(index).unwrap();
        assert_eq!(contents(&file), index.to_le_bytes());
        assert!(reader.blk_file_cache.read().len() <= MAX_CACHED_FILES);
    }
    assert!(!reader.blk_file_cache.read().contains_key(&0));
    assert_eq!(contents(&first), [0, 0]);
    drop(first);
    assert!(lifetime.upgrade().is_none());
    let cached = reader.open_blk(64).unwrap();
    assert!(Arc::ptr_eq(&cached, &reader.open_blk(64).unwrap()));
    assert!(reader.open_blk(1000).is_err());
    assert_eq!(reader.blk_file_cache.read().len(), MAX_CACHED_FILES);
}

#[test]
fn successful_and_failed_refreshes_discard_cached_inodes() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("blk00000.dat");
    fs::write(&path, [1, 1]).unwrap();
    let reader = reader(directory.path().to_owned());
    let first = reader.open_blk(0).unwrap();
    fs::rename(&path, directory.path().join("saved-first")).unwrap();
    fs::write(&path, [2, 2]).unwrap();
    reader.refresh_paths().unwrap();
    let second = reader.open_blk(0).unwrap();
    assert_eq!(contents(&first), [1, 1]);
    assert_eq!(contents(&second), [2, 2]);

    fs::rename(&path, directory.path().join("saved-second")).unwrap();
    fs::write(&path, [3, 3]).unwrap();
    fs::write(directory.path().join("blkbad.dat"), []).unwrap();
    assert!(reader.refresh_paths().is_err());
    assert!(reader.blk_file_cache.read().is_empty());
    assert_eq!(contents(&second), [2, 2]);
    assert_eq!(contents(&reader.open_blk(0).unwrap()), [3, 3]);
}
