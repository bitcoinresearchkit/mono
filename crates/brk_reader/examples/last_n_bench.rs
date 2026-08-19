//! Times `Reader::after` for a handful of tail-clustered catchup
//! sizes. `N ≤ ~1024` lands in the tail strategy (chunked reverse
//! reader); `N = 10_000` falls through to the forward strategy since
//! it's past the 8-newest-files window.
//!
//! Run with:
//!   cargo run --release -p brk_reader --example last_n_bench
//!
//! Requires a running bitcoind with a cookie file at the default path.

use std::time::Instant;

use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use brk_types::Height;

const SCENARIOS: &[u32] = &[1, 10, 100, 1_000, 10_000];

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
    println!("{:>6}  {:>14}  {:>10}", "blocks", "elapsed", "blk/s");
    println!("{}", "-".repeat(36));

    for &n in SCENARIOS {
        let anchor_height = Height::from(tip.saturating_sub(n));
        let anchor_hash = client.get_block_hash(*anchor_height as u64)?;
        let anchor = Some(anchor_hash);

        let start = Instant::now();
        let mut count = 0usize;
        for block in reader.after(anchor)? {
            let _ = block?;
            count += 1;
        }
        let elapsed = start.elapsed();

        let blk_per_s = count as f64 / elapsed.as_secs_f64().max(f64::EPSILON);
        println!("{n:>6}  {elapsed:>14?}  {blk_per_s:>10.0}");
    }

    Ok(())
}
