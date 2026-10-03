use std::{
    collections::HashSet,
    sync::atomic::{AtomicU32, Ordering},
    thread,
    time::Instant,
};

use brk_rpc::ConnectArgs;
use brk_types::BlockHashPrefix;

fn main() {
    let node = ConnectArgs::default();
    let client = node.client().unwrap();

    let tip = u32::from(client.get_last_height().unwrap());
    let num_threads = thread::available_parallelism().unwrap().get();

    println!("Tip: {tip}, Threads: {num_threads}");

    let counter = AtomicU32::new(0);
    let start = Instant::now();

    let results: Vec<Vec<BlockHashPrefix>> = thread::scope(|s| {
        (0..num_threads)
            .map(|t| {
                let client = &client;
                let counter = &counter;
                s.spawn(move || {
                    let mut local = Vec::new();
                    let mut h = t as u32;
                    while h <= tip {
                        let hash = client.get_block_hash(h as u64).unwrap();
                        local.push(BlockHashPrefix::from(hash));
                        let c = counter.fetch_add(1, Ordering::Relaxed);
                        if c.is_multiple_of(50_000) && c > 0 {
                            let rate = c as f64 / start.elapsed().as_secs_f64();
                            println!("  {c}/{tip} ({rate:.0}/s)");
                        }
                        h += num_threads as u32;
                    }
                    local
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect()
    });

    let set: HashSet<BlockHashPrefix> = results.into_iter().flatten().collect();

    let elapsed = start.elapsed();
    let rate = tip as f64 / elapsed.as_secs_f64();

    println!("\nDone in {elapsed:.2?}");
    println!("  {} prefixes at {rate:.0}/s", set.len());
    println!("  ~{:.1} MB", set.len() as f64 * 8.0 / 1_048_576.0);
}
