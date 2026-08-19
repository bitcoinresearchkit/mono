use std::{thread, time::Duration};

use brk_mempool::Mempool;
use brk_rpc::{Auth, Client};

fn main() -> brk_error::Result<()> {
    brk_logger::init(None)?;

    let bitcoin_dir = Client::default_bitcoin_path();
    let client = Client::new(
        Client::default_url(),
        Auth::CookieFile(bitcoin_dir.join(".cookie")),
    )?;

    let mempool = Mempool::new(&client);

    let mempool_clone = mempool.clone();
    thread::spawn(move || {
        mempool_clone.start();
    });

    loop {
        thread::sleep(Duration::from_secs(5));

        let info_count = mempool.info().count;
        let stats = mempool.stats();
        let snapshot = mempool.snapshot();
        let blocks_tx_total: usize = snapshot.blocks.iter().map(Vec::len).sum();

        println!(
            "info.count={} txs={} unresolved={} addrs={} outpoints={} \
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
