//! Small repeatable storage workloads; run with `cargo bench -p rawdb --bench storage`.
use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use rawdb::{Database, PAGE_SIZE, Result};
use tempfile::TempDir;

fn main() -> Result<()> {
    for _ in 0..5 {
        let dir = TempDir::new()?;
        let db = Database::open(dir.path())?;
        let region = db.create_region_if_needed("values")?;
        region.reserve_capacity(64 * 1024 * 1024)?;
        let block = vec![7u8; 64 * 1024];
        let time = Instant::now();
        for _ in 0..1024 {
            region.write(black_box(&block))?;
        }
        let append = time.elapsed();
        let time = Instant::now();
        region.batch_write_ordered(
            (0..1_000_000u64).map(|i| (i as usize * 8, i)),
            8,
            |v, out| out.copy_from_slice(&v.to_le_bytes()),
        );
        let batch = time.elapsed();
        let time = Instant::now();
        for _ in 0..100_000 {
            black_box(region.with_read_bytes(|bytes| bytes[0]));
        }
        let scoped = time.elapsed();
        let reader = region.create_reader();
        let time = Instant::now();
        for _ in 0..8 {
            black_box(
                reader
                    .read_all()
                    .iter()
                    .fold(0u64, |sum, &b| sum + u64::from(b)),
            );
        }
        let scan = time.elapsed();
        drop(reader);
        let time = Instant::now();
        db.flush()?;
        let flush = time.elapsed();
        let allocation = reuse_holes()?;
        let growth = grow_regions()?;
        let small_append = append_small_records()?;
        let fragmented = merge_fragmented_writes()?;
        let (all_dirty, clean) = flush_regions(1)?;
        let (sparse_dirty, _) = flush_regions(16)?;
        let (lookup, reopen, retain, compact) = registry_operations()?;
        println!(
            "append_ms={:.3} batch_ms={:.3} scoped_ms={:.3} scan_ms={:.3} flush_ms={:.3} allocation_ms={:.3} growth_ms={:.3} small_append_ms={:.3} fragmented_ms={:.3} all_dirty_flush_ms={:.3} sparse_dirty_flush_ms={:.3} clean_flush_ms={:.3} lookup_ms={:.3} reopen_ms={:.3} retain_ms={:.3} compact_ms={:.3}",
            append.as_secs_f64() * 1e3,
            batch.as_secs_f64() * 1e3,
            scoped.as_secs_f64() * 1e3,
            scan.as_secs_f64() * 1e3,
            flush.as_secs_f64() * 1e3,
            allocation.as_secs_f64() * 1e3,
            growth.as_secs_f64() * 1e3,
            small_append.as_secs_f64() * 1e3,
            fragmented.as_secs_f64() * 1e3,
            all_dirty.as_secs_f64() * 1e3,
            sparse_dirty.as_secs_f64() * 1e3,
            clean.as_secs_f64() * 1e3,
            lookup.as_secs_f64() * 1e3,
            reopen.as_secs_f64() * 1e3,
            retain.as_secs_f64() * 1e3,
            compact.as_secs_f64() * 1e3
        );
    }
    Ok(())
}

fn registry_operations() -> Result<(Duration, Duration, Duration, Duration)> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    let ids: Vec<_> = (0..4096).map(|index| index.to_string()).collect();
    for id in &ids {
        db.create_region_if_needed(id)?.write(&[7])?;
    }
    db.flush()?;
    let time = Instant::now();
    for _ in 0..64 {
        for id in &ids {
            drop(black_box(db.get_region(black_box(id)).unwrap()));
        }
    }
    let lookup = time.elapsed();
    drop(db);

    let time = Instant::now();
    let db = Database::open(dir.path())?;
    let reopen = time.elapsed();
    for id in &ids[..2048] {
        assert!(db.get_region(id).is_some());
    }
    let time = Instant::now();
    db.retain_accessed_regions()?;
    let retain = time.elapsed();
    let time = Instant::now();
    db.compact()?;
    let compact = time.elapsed();
    assert!(db.get_region(&ids[2048]).is_none());
    assert_eq!(
        db.get_region(&ids[0]).unwrap().create_reader().read_all(),
        &[7]
    );
    Ok((lookup, reopen, retain, compact))
}

fn grow_regions() -> Result<Duration> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    db.set_min_len(64 * 1024 * 1024)?;
    let mut regions = Vec::new();
    for index in 0..2048 {
        let region = db.create_region_if_needed(&index.to_string())?;
        region.write(&[7; 8])?;
        regions.push(region);
    }
    let mut kept = Vec::new();
    for (index, region) in regions.into_iter().enumerate() {
        if index % 2 == 0 {
            kept.push(region);
        } else {
            region.remove()?;
        }
    }
    db.flush()?;
    let time = Instant::now();
    for region in &kept {
        region.reserve_capacity(2 * PAGE_SIZE)?;
    }
    for region in &kept {
        region.reserve_capacity(4 * PAGE_SIZE)?;
    }
    let elapsed = time.elapsed();
    for region in &kept {
        assert_eq!(region.create_reader().read_all(), &[7; 8]);
    }
    Ok(elapsed)
}

fn flush_regions(stride: usize) -> Result<(Duration, Duration)> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    let mut regions = Vec::new();
    for index in 0..1024 {
        let region = db.create_region_if_needed(&index.to_string())?;
        region.write(&[1])?;
        regions.push(region);
    }
    db.flush()?;
    for region in regions.iter().step_by(stride) {
        region.write_at(&[2], 0)?;
    }
    let time = Instant::now();
    let flushed = db.flush()?;
    let dirty = time.elapsed();
    assert_eq!(flushed, regions.len().div_ceil(stride));
    let time = Instant::now();
    for _ in 0..32 {
        assert_eq!(black_box(db.flush()?), 0);
    }
    Ok((dirty, time.elapsed()))
}

fn append_small_records() -> Result<Duration> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    let region = db.create_region_if_needed("records")?;
    region.reserve_capacity(100_000 * 8)?;
    let time = Instant::now();
    for value in 0..100_000u64 {
        region.write(black_box(&value.to_le_bytes()))?;
    }
    Ok(time.elapsed())
}

fn merge_fragmented_writes() -> Result<Duration> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    let region = db.create_region_if_needed("fragmented")?;
    let bytes = vec![0; 8192 * 16];
    region.write(&bytes)?;
    db.flush()?;
    let time = Instant::now();
    for index in 0..8192 {
        region.write_at(&[1; 8], index * 16)?;
    }
    region.write_at(black_box(&bytes), 0)?;
    Ok(time.elapsed())
}

fn reuse_holes() -> Result<Duration> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    for index in 0..4096 {
        drop(db.create_region_if_needed(&index.to_string())?);
    }
    for index in (0..4096).step_by(2) {
        db.remove_region(&index.to_string())?;
    }
    db.flush()?;
    let time = Instant::now();
    for index in (0..4096).step_by(2) {
        drop(black_box(db.create_region_if_needed(&index.to_string())?));
    }
    Ok(time.elapsed())
}
