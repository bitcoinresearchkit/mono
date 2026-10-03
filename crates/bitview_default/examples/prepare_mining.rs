use bitview_plugin::{ComputePlugin, ImportContext, UpdateContext};
use bitview_plugin_blocks::Vecs as Blocks;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_plugin_mining::{Dependencies, Vecs as Mining};
use bitview_plugin_price::Vecs as Price;
use bitview_plugin_transactions::Vecs as Transactions;
use brk_alloc as _;
use brk_error::Result;
use brk_exit::Exit;
use brk_reader::Reader;
use brk_rpc::ConnectArgs;
use std::{env, path::Path, thread, time::Instant};
use vecdb::{AnyVec, Budgeted};
fn work() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    Budgeted::init_global(bitview_plugin::DEFAULT_CACHE_BUDGET)?;
    let node = ConnectArgs::default();
    let client = node.client()?;
    let reader = Reader::new(node.blocks_dir(), &client);
    let context = ImportContext::new(Path::new(&args[1]));
    let indexer = Indexer::import(context, &reader)?;
    let mappings = Mappings::import(context, &indexer)?;
    let price = Price::import(context, &mappings)?;
    let blocks = Blocks::import(context, &indexer, &mappings)?;
    let transactions = Transactions::import(
        context,
        &indexer,
        &mappings,
        &blocks.lookback.window_starts(),
    )?;
    let mut mining = Mining::import(context, &mappings, &blocks.lookback.window_starts())?;
    let timer = Instant::now();
    mining.compute(
        Dependencies {
            indexer: &indexer,
            blocks: &blocks,
            transactions: &transactions,
            price: &price,
        },
        UpdateContext::new(&Exit::new()),
    )?;
    println!(
        "Mining prerequisite: {} output-volume blocks in {:?}",
        mining.rewards.output_volume.len(),
        timer.elapsed()
    );
    Ok(())
}
fn main() {
    thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(work)
        .unwrap()
        .join()
        .unwrap()
        .unwrap();
}
