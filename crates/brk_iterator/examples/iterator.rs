use std::time::Instant;

use brk_iterator::Blocks;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use brk_types::Height;

fn main() -> brk_error::Result<()> {
    let bitcoin_dir = Client::default_bitcoin_path();

    let client = Client::new(
        Client::default_url(),
        Auth::CookieFile(bitcoin_dir.join(".cookie")),
    )?;

    let reader = Reader::new(bitcoin_dir.join("blocks"), &client);

    let blocks = Blocks::new(&client, &reader);

    let i = Instant::now();
    for block in blocks.range(Height::new(920040), Height::new(920041))? {
        let block = block?;
        dbg!(block.height());
    }
    dbg!(i.elapsed());

    Ok(())
}
