//! Scaling benchmark to find maximum concurrent read/write performance.
//!
//! Run with: cargo test --package vecdb --test scaling_bench --features pco --release -- --ignored --nocapture

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use rawdb::Database;
use tempfile::TempDir;
use vecdb::{AnyStoredVec, AnyVec, ImportableVec, ReadableVec, StoredVec, Version, WritableVec};

#[cfg(feature = "pco")]
use vecdb::PcoVec;

fn setup_test_db() -> vecdb::Result<(Database, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db = Database::open(temp_dir.path())?;
    Ok((db, temp_dir))
}

/// Test configuration
#[cfg(feature = "pco")]
struct Config {
    num_readers: usize,
    write_interval_us: u64,
    batch_size: usize,
    duration_secs: u64,
}

/// Run a single benchmark with the given configuration
#[cfg(feature = "pco")]
fn run_benchmark(config: &Config) -> vecdb::Result<(usize, usize, usize)> {
    let (db, _temp) = setup_test_db()?;
    let version = Version::ONE;

    let mut writer: PcoVec<usize, u64> = PcoVec::forced_import(&db, "bench_vec", version)?;

    // Write initial data
    for i in 0..1000u64 {
        writer.push(i);
    }
    writer.write()?;

    let stop = Arc::new(AtomicBool::new(false));
    let reads_completed = Arc::new(AtomicUsize::new(0));
    let read_errors = Arc::new(AtomicUsize::new(0));

    // Spawn readers
    let reader_handles: Vec<_> = (0..config.num_readers)
        .map(|_| {
            let reader = writer.read_only_clone();
            let stop = stop.clone();
            let reads = reads_completed.clone();
            let errs = read_errors.clone();

            thread::spawn(move || {
                let mut local_reads = 0usize;
                let mut local_errors = 0usize;

                while !stop.load(Ordering::Relaxed) {
                    let len = reader.len();
                    if len > 0 {
                        let idx = len - 1;
                        if let Some(v) = reader.collect_one(idx) {
                            if v != idx as u64 {
                                eprintln!(
                                    "ERROR: idx={} expected={} got={} len={}",
                                    idx, idx, v, len
                                );
                                local_errors += 1;
                            }
                            local_reads += 1;
                        } else {
                            eprintln!("ERROR: empty collect at idx={} len={}", idx, len);
                            local_errors += 1;
                        }
                    }
                    // Tight loop - no sleep
                }

                reads.fetch_add(local_reads, Ordering::Relaxed);
                errs.fetch_add(local_errors, Ordering::Relaxed);
            })
        })
        .collect();

    // Writer
    let start = Instant::now();
    let target_duration = Duration::from_secs(config.duration_secs);
    let mut current_idx = 1000u64;
    let mut writes = 0usize;

    while start.elapsed() < target_duration {
        for _ in 0..config.batch_size {
            writer.push(current_idx);
            current_idx += 1;
        }
        writer.write()?;
        writes += 1;

        if config.write_interval_us > 0 {
            thread::sleep(Duration::from_micros(config.write_interval_us));
        }
    }

    // Stop readers
    stop.store(true, Ordering::Relaxed);
    thread::sleep(Duration::from_millis(50));

    for handle in reader_handles {
        handle.join().unwrap();
    }

    let final_reads = reads_completed.load(Ordering::Relaxed);
    let final_errors = read_errors.load(Ordering::Relaxed);

    Ok((writes, final_reads, final_errors))
}

#[test]
#[ignore]
#[cfg(feature = "pco")]
fn test_scale_readers() -> vecdb::Result<()> {
    println!("\n=== Scaling Readers (fixed write interval: 10ms, batch: 100) ===\n");
    println!(
        "{:>8} {:>12} {:>12} {:>8}",
        "Readers", "Writes", "Reads", "Errors"
    );
    println!("{:-<44}", "");

    for num_readers in [1, 2, 4, 8, 16, 32, 64] {
        let config = Config {
            num_readers,
            write_interval_us: 10_000, // 10ms
            batch_size: 100,
            duration_secs: 3,
        };

        let (writes, reads, errors) = run_benchmark(&config)?;
        println!(
            "{:>8} {:>12} {:>12} {:>8}",
            num_readers, writes, reads, errors
        );

        if errors > 0 {
            println!(">>> ERRORS DETECTED - stopping scale test");
            break;
        }
    }

    Ok(())
}

#[test]
#[ignore]
#[cfg(feature = "pco")]
fn test_scale_write_frequency() -> vecdb::Result<()> {
    println!("\n=== Scaling Write Frequency (fixed readers: 4, batch: 100) ===\n");
    println!(
        "{:>12} {:>12} {:>12} {:>8}",
        "Interval(us)", "Writes", "Reads", "Errors"
    );
    println!("{:-<48}", "");

    for write_interval_us in [100_000, 50_000, 10_000, 5_000, 1_000, 500, 100, 0] {
        let config = Config {
            num_readers: 4,
            write_interval_us,
            batch_size: 100,
            duration_secs: 3,
        };

        let (writes, reads, errors) = run_benchmark(&config)?;
        println!(
            "{:>12} {:>12} {:>12} {:>8}",
            write_interval_us, writes, reads, errors
        );

        if errors > 0 {
            println!(">>> ERRORS DETECTED - stopping scale test");
            break;
        }
    }

    Ok(())
}

#[test]
#[ignore]
#[cfg(feature = "pco")]
fn test_scale_batch_size() -> vecdb::Result<()> {
    println!("\n=== Scaling Batch Size (fixed readers: 4, interval: 1ms) ===\n");
    println!(
        "{:>12} {:>12} {:>12} {:>8}",
        "Batch Size", "Writes", "Reads", "Errors"
    );
    println!("{:-<48}", "");

    for batch_size in [10, 50, 100, 500, 1000, 5000, 10000] {
        let config = Config {
            num_readers: 4,
            write_interval_us: 1_000, // 1ms
            batch_size,
            duration_secs: 3,
        };

        let (writes, reads, errors) = run_benchmark(&config)?;
        println!(
            "{:>12} {:>12} {:>12} {:>8}",
            batch_size, writes, reads, errors
        );

        if errors > 0 {
            println!(">>> ERRORS DETECTED - stopping scale test");
            break;
        }
    }

    Ok(())
}

#[test]
#[ignore]
#[cfg(feature = "pco")]
fn test_extreme_stress() -> vecdb::Result<()> {
    println!("\n=== Extreme Stress Test ===\n");
    println!("64 readers, no write delay, batch size 100, 10 seconds\n");

    let config = Config {
        num_readers: 64,
        write_interval_us: 0,
        batch_size: 100,
        duration_secs: 10,
    };

    let (writes, reads, errors) = run_benchmark(&config)?;

    println!("Results:");
    println!("  Writes: {}", writes);
    println!("  Reads:  {}", reads);
    println!("  Errors: {}", errors);
    println!("  Reads/sec: {:.0}", reads as f64 / 10.0);
    println!("  Writes/sec: {:.0}", writes as f64 / 10.0);

    assert_eq!(errors, 0, "Data integrity errors detected!");

    Ok(())
}
