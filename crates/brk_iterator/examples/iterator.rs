use std::time::Instant;

use brk_error::Result;
use brk_iterator::Blocks;
use brk_reader::Reader;
use brk_rpc::ConnectArgs;
use brk_types::Height;

fn main() -> Result<()> {
    let node = ConnectArgs::default();

    let client = node.client()?;

    let reader = Reader::new(node.blocks_dir(), &client);

    let blocks = Blocks::new(&client, &reader);

    let i = Instant::now();
    for block in blocks.range(Height::new(920040), Height::new(920041))? {
        let block = block?;
        dbg!(block.height());
    }
    dbg!(i.elapsed());

    Ok(())
}
