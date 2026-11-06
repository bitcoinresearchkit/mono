//! The oracle over an indexer's blocks, as the price plugin computes it: from height 340,000 (its
//! seed) to `end`, exclusive (974803 runs through 974,802), every block's price in cents, as CSV on stdout (`height,cents`), to score
//! against an exchange tape (the oracle README's accuracy tables). Two builds' outputs compare a
//! change.
//!
//! The indexer is imported for writing, so point it at a copy (an APFS clone) of a data directory,
//! never a running server's: `cargo run -p bitview_plugin_price --example oracle_bench --
//! <bitview dir> <end height>`. The node's RPC (its cookie by default) finds its blocks.

use std::{
    env,
    io::{BufWriter, Write, stdout},
    path::PathBuf,
};

use bitview_plugin::ImportContext;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_price::feed_blocks_with;
use brk_error::{Error, Result};
use brk_exit::Exit;
use brk_oracle::{
    Config, Oracle, START_HEIGHT_FAST, START_HEIGHT_SLOW, bin_to_cents, cents_to_bin,
    pre_oracle_prices_from,
};
use brk_reader::Reader;
use brk_rpc::ConnectArgs;
use vecdb::{AnyVec, Budgeted};

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let dir = PathBuf::from(args.next().expect("a bitview data directory (a copy)"));
    let end: usize = args.next().expect("an end height, exclusive").parse().expect("a height");

    // bitviewd's default vector cache: 1 GiB.
    Budgeted::init_global(1 << 30)?;
    let connect = ConnectArgs::default();
    let client = connect.client()?;
    let reader = Reader::new(connect.blocks_dir()?, &client)?;
    let exit = Exit::new();
    let indexer = Indexer::import(ImportContext::new(&dir, &exit), &reader)?;
    let end = end.min(indexer.vecs().blocks.timestamp.len());

    let seed = pre_oracle_prices_from(START_HEIGHT_SLOW - 1)
        .next()
        .expect("the seed price");
    let mut oracle = Oracle::from_checkpoint(
        cents_to_bin(f64::from(seed)),
        Config::for_height(START_HEIGHT_SLOW),
        |_| Ok::<_, Error>(()),
    )?;
    let mut out = BufWriter::new(stdout().lock());
    writeln!(out, "height,cents")?;
    let mut write = Ok(());
    for segment in Config::segments_for_range(START_HEIGHT_SLOW..end) {
        let segment_end = segment.end;
        feed_blocks_with(&mut oracle, &indexer, segment, None, |height, _, ref_bin| {
            if write.is_ok() {
                write = writeln!(out, "{height},{}", u64::from(bin_to_cents(ref_bin)));
            }
        })?;
        if segment_end == START_HEIGHT_FAST {
            oracle.reconfigure(Config::default());
        }
    }
    write?;
    Ok(out.flush()?)
}
