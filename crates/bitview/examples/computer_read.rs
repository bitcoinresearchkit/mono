use brk_error::Result;

use std::{env, path::Path, time::Instant};

use bitview::Computer;
use brk_indexer::Indexer;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use vecdb::{AnySerializableVec, AnyVec};

pub fn main() -> Result<()> {
    brk_logger::init(None)?;

    let outputs_dir = Path::new(&env::var("HOME").unwrap()).join(".bitview");

    let bitcoin_dir = Client::default_bitcoin_path();
    let client = Client::new(
        Client::default_url(),
        Auth::CookieFile(bitcoin_dir.join(".cookie")),
    )?;
    let reader = Reader::new(bitcoin_dir.join("blocks"), &client);
    let indexer = Indexer::import(&outputs_dir, &reader)?;

    let computer = Computer::forced_import(&outputs_dir, &indexer)?;

    // Test empty_addr_data (underlying BytesVec) - direct access
    let empty_data = &computer.distribution.addrs_data.empty;
    println!("empty_addr_data (BytesVec) len: {}", empty_data.len());

    let start = Instant::now();
    let mut buf = Vec::new();
    empty_data.write_json(Some(empty_data.len() - 1), Some(empty_data.len()), &mut buf)?;
    println!(
        "empty_addr_data last item JSON: {}",
        String::from_utf8_lossy(&buf)
    );
    println!("Time for BytesVec write_json: {:?}", start.elapsed());

    // Test empty_addr_index (LazyVec wrapper) - computed access
    let empty_index = &computer.distribution.addrs.empty_index;
    println!("\nempty_addr_index (LazyVec) len: {}", empty_index.len());

    let start = Instant::now();
    let mut buf = Vec::new();
    empty_index.write_json(
        Some(empty_index.len() - 1),
        Some(empty_index.len()),
        &mut buf,
    )?;
    println!(
        "empty_addr_index last item JSON: {}",
        String::from_utf8_lossy(&buf)
    );
    println!("Time for LazyVec write_json: {:?}", start.elapsed());

    // Compare with funded versions
    let funded_data = &computer.distribution.addrs_data.funded;
    println!("\nfunded_addr_data (BytesVec) len: {}", funded_data.len());

    let start = Instant::now();
    let mut buf = Vec::new();
    funded_data.write_json(
        Some(funded_data.len() - 1),
        Some(funded_data.len()),
        &mut buf,
    )?;
    println!(
        "funded_addr_data last item JSON: {}",
        String::from_utf8_lossy(&buf)
    );
    println!("Time for BytesVec write_json: {:?}", start.elapsed());

    Ok(())
}
