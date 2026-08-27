use std::time::{Duration, Instant};

use vecdb::{
    AnyStoredVec, BytesVec, Database, ImportableVec, LZ4Vec, PcoVec, ReadableVec, StoredVec,
    Version, WritableVec, ZeroCopyVec, ZstdVec,
};

const DEFAULT_VALUE_COUNT: usize = 10_000_000_000; // 10B u64s = 80 GB
const BATCH_SIZE: usize = 10_000_000;
const MAX_RANGE_BYTES: usize = 8 * 1024 * 1024 * 1024; // 8 GB

fn value_count() -> usize {
    std::env::var("BENCH_COUNT")
        .ok()
        .and_then(|s| s.replace('_', "").parse().ok())
        .unwrap_or(DEFAULT_VALUE_COUNT)
}

fn range_passes() -> usize {
    std::env::var("BENCH_PASSES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5)
}

fn range_sizes(count: usize, value_size: usize) -> Vec<usize> {
    let max_elements = (MAX_RANGE_BYTES / value_size).min(count);
    let mut sizes = Vec::new();
    let mut n = 1_000;
    while n <= max_elements {
        sizes.push(n);
        let next5 = n * 5;
        if next5 <= max_elements && next5 != n * 10 {
            sizes.push(next5);
        }
        n *= 10;
    }
    sizes
}

fn repetitions(range_size: usize) -> usize {
    match range_size {
        n if n < 10_000 => 5_000,
        n if n < 100_000 => 1_000,
        n if n < 1_000_000 => 100,
        n if n < 10_000_000 => 20,
        n if n < 100_000_000 => 5,
        _ => 1,
    }
}

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn random_starts(count: usize, max_start: usize) -> Vec<usize> {
    let mut rng = 42u64;
    (0..count)
        .map(|_| xorshift(&mut rng) as usize % max_start.max(1))
        .collect()
}

fn format_bytes(bytes: usize) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.1} GB", bytes as f64 / 1e9)
    } else if bytes >= 1_000_000 {
        format!("{:.1} MB", bytes as f64 / 1e6)
    } else if bytes >= 1_000 {
        format!("{:.1} KB", bytes as f64 / 1e3)
    } else {
        format!("{} B", bytes)
    }
}

fn format_duration(d: Duration) -> String {
    let ns = d.as_nanos();
    if ns >= 1_000_000_000 {
        format!("{:.2} s", d.as_secs_f64())
    } else if ns >= 1_000_000 {
        format!("{:.2} ms", ns as f64 / 1e6)
    } else if ns >= 1_000 {
        format!("{:.2} us", ns as f64 / 1e3)
    } else {
        format!("{} ns", ns)
    }
}

fn throughput_str(bytes: usize, d: Duration) -> String {
    format!("{:.1} GB/s", bytes as f64 / d.as_secs_f64() / 1e9)
}

// --- Page cache eviction ---

fn drop_caches() -> bool {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("purge")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("sync").status().ok();
        std::fs::write("/proc/sys/vm/drop_caches", "3").is_ok()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        false
    }
}

fn check_cache_eviction() -> bool {
    let ok = drop_caches();
    if ok {
        println!("  Cache eviction: available (cold-cache benchmarks)");
    } else {
        println!("  Cache eviction: unavailable (warm-cache benchmarks)");
        println!("  Hint: run with sudo for cold-cache results");
    }
    ok
}

// --- Vec size ---

fn print_vec_size(vec: &impl AnyStoredVec, label: &str) {
    let region_bytes = vec.region().meta().len();
    let logical_bytes = vec.len() * vec.value_type_to_size_of();
    let ratio = if logical_bytes > 0 {
        region_bytes as f64 / logical_bytes as f64
    } else {
        0.0
    };
    eprintln!(
        "  {label} on disk: {} (logical: {}, ratio: {:.2}x)",
        format_bytes(region_bytes),
        format_bytes(logical_bytes),
        ratio,
    );
}

// --- Populate ---

fn populate<V: WritableVec<usize, u64> + ImportableVec + AnyStoredVec>(
    db: &Database,
    label: &str,
    count: usize,
) -> V {
    eprint!("  Populating {label} with {count} values...");
    flush();
    let start = Instant::now();
    let mut vec: V = V::import(db, "bench", Version::ONE).unwrap();
    let mut written = 0;
    while written < count {
        let end = (written + BATCH_SIZE).min(count);
        for i in written..end {
            vec.push(i as u64);
        }
        vec.write().unwrap();
        written = end;
        eprint!(
            "\r  Populating {label}: {:.0}%  ",
            written as f64 / count as f64 * 100.0
        );
    }
    db.flush().unwrap();
    eprintln!("\r  Populated {label} ({:?})       ", start.elapsed());
    print_vec_size(&vec, label);
    vec
}

// --- Benchmark helpers ---

// --- Benchmark runners ---

struct BenchResult {
    name: &'static str,
    duration: Duration,
}

fn print_full_results(results: &[BenchResult], total_bytes: usize) {
    let best = results.iter().min_by_key(|r| r.duration).unwrap().duration;
    for r in results {
        let pct = if r.duration > best {
            let overhead = (r.duration.as_secs_f64() / best.as_secs_f64() - 1.0) * 100.0;
            format!("(+{overhead:.0}%)")
        } else {
            String::new()
        };
        println!(
            "  {:<20} {} ({}) {}",
            r.name,
            format_duration(r.duration),
            throughput_str(total_bytes, r.duration),
            pct,
        );
    }
}

fn print_range_header(columns: &[&str]) {
    print!("{:>12} {:>10}", "range", "bytes");
    for col in columns {
        print!(" {:>14}", col);
    }
    println!("  {:<8}", "winner");
    let width = 24 + columns.len() * 15 + 10;
    println!("{}", "-".repeat(width));
}

fn print_range_row(range_size: usize, results: &[BenchResult]) {
    let range_bytes = range_size * 8;
    print!("{:>12} {:>10}", range_size, format_bytes(range_bytes));
    let best = results.iter().min_by_key(|r| r.duration).unwrap();
    for r in results {
        print!(" {:>14}", format_duration(r.duration));
    }
    println!("  {:<8}", best.name);
}

// --- StoredFold trait for IO/mmap access ---

trait StoredFold {
    fn fold_stored_io_sum(&self, from: usize, to: usize, acc: u64) -> u64;
    fn fold_stored_mmap_sum(&self, from: usize, to: usize, acc: u64) -> u64;
}

impl StoredFold for BytesVec<usize, u64> {
    fn fold_stored_io_sum(&self, from: usize, to: usize, acc: u64) -> u64 {
        self.fold_stored_io(from, to, acc, |a, v: u64| a.wrapping_add(v))
    }
    fn fold_stored_mmap_sum(&self, from: usize, to: usize, acc: u64) -> u64 {
        self.fold_stored_mmap(from, to, acc, |a, v: u64| a.wrapping_add(v))
    }
}

impl StoredFold for ZeroCopyVec<usize, u64> {
    fn fold_stored_io_sum(&self, from: usize, to: usize, acc: u64) -> u64 {
        self.fold_stored_io(from, to, acc, |a, v: u64| a.wrapping_add(v))
    }
    fn fold_stored_mmap_sum(&self, from: usize, to: usize, acc: u64) -> u64 {
        self.fold_stored_mmap(from, to, acc, |a, v: u64| a.wrapping_add(v))
    }
}

impl StoredFold for LZ4Vec<usize, u64> {
    fn fold_stored_io_sum(&self, from: usize, to: usize, acc: u64) -> u64 {
        self.fold_stored_io(from, to, acc, |a, v: u64| a.wrapping_add(v))
    }
    fn fold_stored_mmap_sum(&self, from: usize, to: usize, acc: u64) -> u64 {
        self.fold_stored_mmap(from, to, acc, |a, v: u64| a.wrapping_add(v))
    }
}

impl StoredFold for PcoVec<usize, u64> {
    fn fold_stored_io_sum(&self, from: usize, to: usize, acc: u64) -> u64 {
        self.fold_stored_io(from, to, acc, |a, v: u64| a.wrapping_add(v))
    }
    fn fold_stored_mmap_sum(&self, from: usize, to: usize, acc: u64) -> u64 {
        self.fold_stored_mmap(from, to, acc, |a, v: u64| a.wrapping_add(v))
    }
}

impl StoredFold for ZstdVec<usize, u64> {
    fn fold_stored_io_sum(&self, from: usize, to: usize, acc: u64) -> u64 {
        self.fold_stored_io(from, to, acc, |a, v: u64| a.wrapping_add(v))
    }
    fn fold_stored_mmap_sum(&self, from: usize, to: usize, acc: u64) -> u64 {
        self.fold_stored_mmap(from, to, acc, |a, v: u64| a.wrapping_add(v))
    }
}

// --- Unified benchmark ---

fn bench_vec<
    V: ReadableVec<usize, u64> + StoredFold + AnyStoredVec + StoredVec<I = usize, T = u64>,
>(
    vec: &V,
    label: &str,
    count: usize,
    can_purge: bool,
) {
    let total_bytes = count * 8;
    let disk_bytes = vec.region().meta().len();
    let ratio = if total_bytes > 0 {
        disk_bytes as f64 / total_bytes as f64
    } else {
        0.0
    };
    let ranges = range_sizes(count, 8);

    println!(
        "\n=== {label} — {count} values ({}, disk: {}, {:.2}x) ===\n",
        format_bytes(total_bytes),
        format_bytes(disk_bytes),
        ratio,
    );

    // Full scan — multiple passes in random method order to eliminate cache bias.
    let full_passes = range_passes();
    println!("--- Full scan ({full_passes} passes, random order) ---");

    let method_names: [&str; 6] = [
        "fold_stored_io",
        "fold_stored_mmap",
        "fold_range",
        "try_fold_range",
        "for_each_dyn",
        "for_each",
    ];
    let mut times = [Duration::ZERO; 6];
    let mut sum = 0u64;
    let mut rng = 0xCAFEu64;

    for _pass in 0..full_passes {
        let mut order = [0usize, 1, 2, 3, 4, 5];
        for i in (1..6).rev() {
            let j = xorshift(&mut rng) as usize % (i + 1);
            order.swap(i, j);
        }

        for &method in &order {
            if can_purge {
                drop_caches();
            }
            let t = Instant::now();
            match method {
                0 => sum = vec.fold_stored_io_sum(0, count, sum),
                1 => sum = vec.fold_stored_mmap_sum(0, count, sum),
                2 => sum = vec.fold_range(0, count, sum, |a, v: u64| a.wrapping_add(v)),
                3 => {
                    sum = vec
                        .try_fold_range(0, count, sum, |a, v: u64| Ok::<_, ()>(a.wrapping_add(v)))
                        .unwrap()
                }
                4 => vec.for_each_range_dyn(0, count, &mut |v: u64| sum = sum.wrapping_add(v)),
                5 => vec.for_each_range(0, count, |v: u64| sum = sum.wrapping_add(v)),
                _ => unreachable!(),
            }
            times[method] += t.elapsed();
        }
    }

    std::hint::black_box(sum);

    let results: Vec<BenchResult> = (0..6)
        .map(|i| BenchResult {
            name: method_names[i],
            duration: times[i] / full_passes as u32,
        })
        .collect();
    print_full_results(&results, total_bytes);

    // Range scans — multiple passes in random method order to eliminate cache bias.
    let passes = range_passes();
    println!("\n--- Range scans ({passes} passes, random order) ---");
    let columns: Vec<&str> = vec!["IO", "Mmap", "fold", "try_fold", "dyn", "static"];
    print_range_header(&columns);

    for &range_size in &ranges {
        let reps = repetitions(range_size);
        let max_start = count.saturating_sub(range_size);
        let starts = random_starts(reps, max_start);

        let mut times = [Duration::ZERO; 6];
        let mut sum = 0u64;
        let mut rng = range_size as u64 ^ 0xDEAD;

        for _pass in 0..passes {
            // Shuffle method order using Fisher-Yates.
            let mut order = [0usize, 1, 2, 3, 4, 5];
            for i in (1..6).rev() {
                let j = xorshift(&mut rng) as usize % (i + 1);
                order.swap(i, j);
            }

            for &method in &order {
                if can_purge {
                    drop_caches();
                }
                let t = Instant::now();
                for &s in &starts {
                    let from = s;
                    let to = s + range_size;
                    match method {
                        0 => sum = vec.fold_stored_io_sum(from, to, sum),
                        1 => sum = vec.fold_stored_mmap_sum(from, to, sum),
                        2 => sum = vec.fold_range(from, to, sum, |a, v: u64| a.wrapping_add(v)),
                        3 => {
                            sum = vec
                                .try_fold_range(from, to, sum, |a, v: u64| {
                                    Ok::<_, ()>(a.wrapping_add(v))
                                })
                                .unwrap()
                        }
                        4 => vec
                            .for_each_range_dyn(from, to, &mut |v: u64| sum = sum.wrapping_add(v)),
                        5 => vec.for_each_range(from, to, |v: u64| sum = sum.wrapping_add(v)),
                        _ => unreachable!(),
                    }
                }
                times[method] += t.elapsed();
            }
        }

        std::hint::black_box(sum);

        let divisor = (reps * passes) as u32;
        let row = vec![
            BenchResult {
                name: "IO",
                duration: times[0] / divisor,
            },
            BenchResult {
                name: "Mmap",
                duration: times[1] / divisor,
            },
            BenchResult {
                name: "fold",
                duration: times[2] / divisor,
            },
            BenchResult {
                name: "try_fold",
                duration: times[3] / divisor,
            },
            BenchResult {
                name: "dyn",
                duration: times[4] / divisor,
            },
            BenchResult {
                name: "static",
                duration: times[5] / divisor,
            },
        ];
        print_range_row(range_size, &row);
    }

    // --- Read-only clone benchmarks ---

    let ro = vec.read_only_clone();

    let ro_passes = range_passes();
    println!("\n--- Read-only full scan ({ro_passes} passes, random order) ---");

    let ro_method_names: [&str; 4] = [
        "ro_fold_range",
        "ro_try_fold_range",
        "ro_for_each_dyn",
        "ro_for_each",
    ];
    let mut ro_times = [Duration::ZERO; 4];
    let mut sum = 0u64;
    let mut rng = 0xBEEFu64;

    for _pass in 0..ro_passes {
        let mut order = [0usize, 1, 2, 3];
        for i in (1..4).rev() {
            let j = xorshift(&mut rng) as usize % (i + 1);
            order.swap(i, j);
        }

        for &method in &order {
            if can_purge {
                drop_caches();
            }
            let t = Instant::now();
            match method {
                0 => sum = ro.fold_range(0, count, sum, |a, v: u64| a.wrapping_add(v)),
                1 => {
                    sum = ro
                        .try_fold_range(0, count, sum, |a, v: u64| Ok::<_, ()>(a.wrapping_add(v)))
                        .unwrap()
                }
                2 => ro.for_each_range_dyn(0, count, &mut |v: u64| sum = sum.wrapping_add(v)),
                3 => ro.for_each_range(0, count, |v: u64| sum = sum.wrapping_add(v)),
                _ => unreachable!(),
            }
            ro_times[method] += t.elapsed();
        }
    }

    std::hint::black_box(sum);

    let ro_results: Vec<BenchResult> = (0..4)
        .map(|i| BenchResult {
            name: ro_method_names[i],
            duration: ro_times[i] / ro_passes as u32,
        })
        .collect();
    print_full_results(&ro_results, total_bytes);

    let ro_passes = range_passes();
    println!("\n--- Read-only range scans ({ro_passes} passes, random order) ---");
    let ro_columns: Vec<&str> = vec!["fold", "try_fold", "dyn", "static"];
    print_range_header(&ro_columns);

    for &range_size in &ranges {
        let reps = repetitions(range_size);
        let max_start = count.saturating_sub(range_size);
        let starts = random_starts(reps, max_start);

        let mut times = [Duration::ZERO; 4];
        let mut sum = 0u64;
        let mut rng = range_size as u64 ^ 0xFACE;

        for _pass in 0..ro_passes {
            let mut order = [0usize, 1, 2, 3];
            for i in (1..4).rev() {
                let j = xorshift(&mut rng) as usize % (i + 1);
                order.swap(i, j);
            }

            for &method in &order {
                if can_purge {
                    drop_caches();
                }
                let t = Instant::now();
                for &s in &starts {
                    let from = s;
                    let to = s + range_size;
                    match method {
                        0 => sum = ro.fold_range(from, to, sum, |a, v: u64| a.wrapping_add(v)),
                        1 => {
                            sum = ro
                                .try_fold_range(from, to, sum, |a, v: u64| {
                                    Ok::<_, ()>(a.wrapping_add(v))
                                })
                                .unwrap()
                        }
                        2 => {
                            ro.for_each_range_dyn(from, to, &mut |v: u64| sum = sum.wrapping_add(v))
                        }
                        3 => ro.for_each_range(from, to, |v: u64| sum = sum.wrapping_add(v)),
                        _ => unreachable!(),
                    }
                }
                times[method] += t.elapsed();
            }
        }

        std::hint::black_box(sum);

        let divisor = (reps * ro_passes) as u32;
        let row = vec![
            BenchResult {
                name: "fold",
                duration: times[0] / divisor,
            },
            BenchResult {
                name: "try_fold",
                duration: times[1] / divisor,
            },
            BenchResult {
                name: "dyn",
                duration: times[2] / divisor,
            },
            BenchResult {
                name: "static",
                duration: times[3] / divisor,
            },
        ];
        print_range_row(range_size, &row);
    }
}

/// Fixed bench directory — cleaned up at start so Ctrl+C leftovers don't accumulate.
fn bench_dir() -> std::path::PathBuf {
    std::env::temp_dir().join("vecdb_bench")
}

fn cleanup_bench_dir() {
    let dir = bench_dir();
    if dir.exists() {
        eprint!("  Cleaning up previous bench data...");
        flush();
        std::fs::remove_dir_all(&dir).ok();
        eprintln!(" done");
    }
}

fn run_bench<
    V: ReadableVec<usize, u64> + StoredFold + AnyStoredVec + StoredVec<I = usize, T = u64>,
>(
    label: &str,
    count: usize,
    can_purge: bool,
) {
    cleanup_bench_dir();
    let dir = bench_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let db = Database::open(&dir).unwrap();
    let vec = populate::<V>(&db, label, count);
    bench_vec(&vec, label, count, can_purge);
    drop(vec);
    drop(db);
    std::fs::remove_dir_all(&dir).ok();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("all");
    let count = value_count();

    println!("BENCH_COUNT={count} (set env to override, e.g. BENCH_COUNT=1_000_000)");
    let can_purge = check_cache_eviction();
    println!();

    let run_bytes = matches!(mode, "all" | "bytes" | "raw");
    let run_zerocopy = matches!(mode, "all" | "zerocopy" | "raw");
    let run_lz4 = matches!(mode, "all" | "lz4" | "compressed");
    let run_pco = matches!(mode, "all" | "pco" | "compressed");
    let run_zstd = matches!(mode, "all" | "zstd" | "compressed");

    if run_bytes {
        run_bench::<BytesVec<usize, u64>>("BytesVec<usize, u64>", count, can_purge);
    }
    if run_zerocopy {
        run_bench::<ZeroCopyVec<usize, u64>>("ZeroCopyVec<usize, u64>", count, can_purge);
    }
    if run_lz4 {
        run_bench::<LZ4Vec<usize, u64>>("LZ4Vec<usize, u64>", count, can_purge);
    }
    if run_pco {
        run_bench::<PcoVec<usize, u64>>("PcoVec<usize, u64>", count, can_purge);
    }
    if run_zstd {
        run_bench::<ZstdVec<usize, u64>>("ZstdVec<usize, u64>", count, can_purge);
    }

    cleanup_bench_dir();
}

fn flush() {
    std::io::Write::flush(&mut std::io::stderr()).ok();
    std::io::Write::flush(&mut std::io::stdout()).ok();
}
