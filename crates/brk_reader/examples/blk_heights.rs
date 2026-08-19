use brk_reader::{BlkIndexToBlkPath, Reader};
use brk_rpc::{Auth, Client};

fn main() -> brk_error::Result<()> {
    let bitcoin_dir = Client::default_bitcoin_path();
    let client = Client::new(
        Client::default_url(),
        Auth::CookieFile(bitcoin_dir.join(".cookie")),
    )?;

    let reader = Reader::new(bitcoin_dir.join("blocks"), &client);
    let xor_bytes = reader.xor_bytes();
    let blk_map = BlkIndexToBlkPath::scan(reader.blocks_dir())?;

    let mut prev_height: Option<u32> = None;
    let mut max_drop: u32 = 0;
    let mut max_drop_at: u16 = 0;

    for (&blk_index, blk_path) in blk_map.iter() {
        match reader.first_block_height(blk_path, xor_bytes) {
            Ok(height) => {
                let h = u32::from(height);
                let drop = prev_height.map(|p| p.saturating_sub(h)).unwrap_or(0);
                if drop > max_drop {
                    max_drop = drop;
                    max_drop_at = blk_index;
                }
                let blocks_per_file = prev_height.map(|p| h.saturating_sub(p));
                println!(
                    "blk{blk_index:05}.dat  first_height={h:>7}  gap={:>5}  drop={drop}",
                    blocks_per_file.map(|b| b.to_string()).unwrap_or("-".into()),
                );
                prev_height = Some(h);
            }
            Err(e) => {
                println!("blk{blk_index:05}.dat  ERROR: {e}");
            }
        }
    }

    println!("\nMax backwards drop: {max_drop} blocks (at blk{max_drop_at:05}.dat)");
    println!("Files scanned: {}", blk_map.len());

    Ok(())
}
