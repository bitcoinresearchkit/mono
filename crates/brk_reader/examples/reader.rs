use std::time::Instant;

use brk_error::Result;
use brk_reader::Reader;
use brk_rpc::ConnectArgs;

fn main() -> Result<()> {
    let node = ConnectArgs::default();

    let client = node.client()?;

    let reader = Reader::new(node.blocks_dir(), &client);

    // Stream all blocks from genesis to the current tip.
    let i = Instant::now();
    for block in reader.after(None)?.iter() {
        let block = block?;
        println!("{}: {}", block.height(), block.hash());
    }
    println!("Full read: {:?}", i.elapsed());

    Ok(())
}
