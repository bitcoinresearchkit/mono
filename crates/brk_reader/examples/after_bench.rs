//! Benchmark `Reader::after` / `Reader::after_with` across three
//! parser-thread counts (1 / 4 / 16).
//!
//! Three phases:
//!
//! 1. **Tail scenarios** — pick an anchor `N` blocks below the chain
//!    tip and run each config `REPEATS` times. Exercises the tail
//!    (≤10) and forward (>10) code paths under realistic catchup
//!    ranges.
//! 2. **Partial reindex** — anchor=`None` but stop after
//!    `PARTIAL_LIMIT` blocks. Exercises the early-chain blk files
//!    where blocks are small and dense-parsing isn't the bottleneck.
//! 3. **Full reindex** — anchor=`None` (genesis to tip), one run per
//!    config. Exercises every blk file once and shows steady-state
//!    throughput on the densest possible workload.
//!
//! Run with:
//!   cargo run --release -p brk_reader --example after_bench
//!
//! Requires a running bitcoind with a cookie file at the default path.

use std::time::{Duration, Instant};

use brk_reader::{BlockReceiver, Reader};
use brk_rpc::{Auth, Client};
use brk_types::Height;

const SCENARIOS: &[usize] = &[5, 10, 100, 1_000, 10_000];
const REPEATS: usize = 3;
const PARTIAL_LIMIT: usize = 400_000;
const PARSER_COUNTS: &[usize] = &[1, 4, 16];

fn main() -> brk_error::Result<()> {
    let bitcoin_dir = Client::default_bitcoin_path();
    let client = Client::new(
        Client::default_url(),
        Auth::CookieFile(bitcoin_dir.join(".cookie")),
    )?;
    let reader = Reader::new(bitcoin_dir.join("blocks"), &client);

    let tip = client.get_last_height()?;
    println!("Tip: {tip}");
    println!();
    println!(
        "{:>10}  {:>12}  {:>12}  {:>12}  {:>10}",
        "blocks", "parsers", "best", "avg", "blk/s"
    );
    println!("{}", "-".repeat(64));

    for &n in SCENARIOS {
        let anchor_height = Height::from(tip.saturating_sub(n as u32));
        let anchor_hash = client.get_block_hash(*anchor_height as u64)?;
        let anchor = Some(anchor_hash);

        let mut first: Option<RunStats> = None;
        for &p in PARSER_COUNTS {
            let stats = bench(REPEATS, || reader.after_with(anchor, p))?;
            print_row(n, p, &stats);
            if let Some(baseline) = &first {
                sanity_check(n, baseline, &stats);
            } else {
                first = Some(stats);
            }
        }
        println!();
    }

    println!();
    println!("Partial reindex (genesis → {PARTIAL_LIMIT} blocks), one run per config:");
    println!(
        "{:>10}  {:>12}  {:>12}  {:>10}",
        "blocks", "parsers", "elapsed", "blk/s"
    );
    println!("{}", "-".repeat(50));
    let mut partial_baseline: Option<FullRun> = None;
    for &p in PARSER_COUNTS {
        let run = run_bounded(PARTIAL_LIMIT, || reader.after_with(None, p))?;
        print_full_row(p, &run);
        if let Some(baseline) = &partial_baseline {
            sanity_check_full(baseline, &run);
        } else {
            partial_baseline = Some(run);
        }
    }

    println!();
    println!("Full reindex (genesis → tip), one run per config:");
    println!(
        "{:>10}  {:>12}  {:>12}  {:>10}",
        "blocks", "parsers", "elapsed", "blk/s"
    );
    println!("{}", "-".repeat(50));
    let mut full_baseline: Option<FullRun> = None;
    for &p in PARSER_COUNTS {
        let run = run_once(|| reader.after_with(None, p))?;
        print_full_row(p, &run);
        if let Some(baseline) = &full_baseline {
            sanity_check_full(baseline, &run);
        } else {
            full_baseline = Some(run);
        }
    }

    Ok(())
}

struct RunStats {
    best: Duration,
    avg: Duration,
    count: usize,
}

fn bench<F>(repeats: usize, mut f: F) -> brk_error::Result<RunStats>
where
    F: FnMut() -> brk_error::Result<BlockReceiver>,
{
    let mut best = Duration::MAX;
    let mut total = Duration::ZERO;
    let mut count = 0;

    for _ in 0..repeats {
        let start = Instant::now();
        let recv = f()?;
        let mut n = 0;
        for block in recv.iter() {
            let block = block?;
            std::hint::black_box(block.height());
            n += 1;
        }
        let elapsed = start.elapsed();
        if elapsed < best {
            best = elapsed;
        }
        total += elapsed;
        count = n;
    }

    Ok(RunStats {
        best,
        avg: total / repeats as u32,
        count,
    })
}

fn print_row(requested: usize, parsers: usize, s: &RunStats) {
    let blk_per_s = if s.best.is_zero() {
        0.0
    } else {
        s.count as f64 / s.best.as_secs_f64()
    };
    println!(
        "{:>10}  {:>12}  {:>12?}  {:>12?}  {:>10.0}",
        requested, parsers, s.best, s.avg, blk_per_s
    );
}

fn sanity_check(requested: usize, baseline: &RunStats, stats: &RunStats) {
    if baseline.count != stats.count {
        println!(
            "  ⚠ block count mismatch: {} vs {}",
            baseline.count, stats.count
        );
    } else if baseline.count != requested {
        println!(
            "  (note: got {} blocks, requested {}; tip may have advanced)",
            baseline.count, requested
        );
    }
}

struct FullRun {
    elapsed: Duration,
    count: usize,
}

fn run_once<F>(mut f: F) -> brk_error::Result<FullRun>
where
    F: FnMut() -> brk_error::Result<BlockReceiver>,
{
    let start = Instant::now();
    let recv = f()?;
    let mut count = 0;
    for block in recv.iter() {
        let block = block?;
        std::hint::black_box(block.height());
        count += 1;
    }
    Ok(FullRun {
        elapsed: start.elapsed(),
        count,
    })
}

/// Runs the pipeline starting from genesis but stops consuming once
/// `limit` blocks have been received. Dropping the receiver then closes
/// the channel, which unblocks and unwinds the reader's spawned worker.
fn run_bounded<F>(limit: usize, mut f: F) -> brk_error::Result<FullRun>
where
    F: FnMut() -> brk_error::Result<BlockReceiver>,
{
    let start = Instant::now();
    let recv = f()?;
    let mut count = 0;
    for block in recv.iter().take(limit) {
        let block = block?;
        std::hint::black_box(block.height());
        count += 1;
    }
    let elapsed = start.elapsed();
    // Explicit drop so the reader worker sees the channel close before
    // the next bench config spins up another one.
    drop(recv);
    Ok(FullRun { elapsed, count })
}

fn print_full_row(parsers: usize, run: &FullRun) {
    let blk_per_s = if run.elapsed.is_zero() {
        0.0
    } else {
        run.count as f64 / run.elapsed.as_secs_f64()
    };
    println!(
        "{:>10}  {:>12}  {:>12?}  {:>10.0}",
        run.count, parsers, run.elapsed, blk_per_s
    );
}

fn sanity_check_full(baseline: &FullRun, run: &FullRun) {
    if baseline.count != run.count {
        println!(
            "  ⚠ block count mismatch: {} vs {}",
            baseline.count, run.count
        );
    }
}
