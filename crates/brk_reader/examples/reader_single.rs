use std::time::Instant;

use brk_error::Result;
use brk_reader::Reader;
use brk_rpc::ConnectArgs;
use brk_types::Height;

fn main() -> Result<()> {
    let node = ConnectArgs::default();

    let client = node.client()?;

    let reader = Reader::new(node.blocks_dir(), &client);

    let heights = [0, 100_000, 158_251, 173_195, 840_000];

    for &h in &heights {
        let height = Height::new(h);
        let i = Instant::now();

        if let Some(block) = reader.range(height, height)?.iter().next() {
            let block = block?;
            println!(
                "height={} hash={} txs={} coinbase=\"{:?}\" ({:?})",
                block.height(),
                block.hash(),
                block.txdata.len(),
                block.coinbase_tag(),
                i.elapsed(),
            );
        }
    }

    Ok(())
}
