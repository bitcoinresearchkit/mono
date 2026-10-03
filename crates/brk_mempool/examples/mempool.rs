use std::{thread, time::Duration};

use brk_error::Result;
use brk_logger::init;
use brk_mempool::Mempool;
use brk_rpc::ConnectArgs;

fn main() -> Result<()> {
    init(None)?;

    let node = ConnectArgs::default();
    let client = node.client()?;

    let mut mempool = Mempool::new(&client);

    let read_only = mempool.read_only_clone();
    thread::spawn(move || {
        mempool.start();
    });

    loop {
        thread::sleep(Duration::from_secs(5));

        let mempool = read_only.load();
        let info_count = mempool.info().map(|info| info.count);
        let stats = mempool.stats();
        let snapshot = mempool.snapshot();
        let blocks_tx_total: usize = snapshot.blocks.iter().map(Vec::len).sum();

        println!(
            "info.count={:?} txs={} unresolved={} addrs={} outpoints={} \
             graveyard.tombstones={} graveyard.order={} \
             snap.txs.len={} snap.blocks={} snap.blocks_txs={} \
             rebuilds={}",
            info_count,
            stats.txs,
            stats.unresolved,
            stats.addrs,
            stats.outpoint_spends,
            stats.graveyard_tombstones,
            stats.graveyard_order,
            snapshot.txs.len(),
            snapshot.blocks.len(),
            blocks_tx_total,
            stats.rebuilds,
        );
    }
}
