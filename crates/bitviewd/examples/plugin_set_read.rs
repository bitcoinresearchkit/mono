use brk_error::Result;

use std::{env, path::Path, time::Instant};

use bitview::ImportContext;
use bitview_default::DefaultPlugins;
use bitview_plugin_distribution::HasDistribution;
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
    let context = ImportContext::new(&outputs_dir);

    let plugins = DefaultPlugins::import(context, &reader)?;
    let distribution = plugins.distribution();

    // Test extended_empty_addr_data (underlying OverflowVec) - direct access
    let empty_data = &distribution.addr_state.extended_empty;
    println!(
        "extended_empty_addr_data (OverflowVec) len: {}",
        empty_data.len()
    );

    let start = Instant::now();
    let mut buf = Vec::new();
    empty_data.write_json(Some(empty_data.len() - 1), Some(empty_data.len()), &mut buf)?;
    println!(
        "empty_addr_data last item JSON: {}",
        String::from_utf8_lossy(&buf)
    );
    println!("Time for OverflowVec write_json: {:?}", start.elapsed());

    // Compare with funded versions
    let funded_data = &distribution.addr_state.funded;
    println!(
        "\nfunded_addr_data (OverflowVec) len: {}",
        funded_data.len()
    );

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
    println!("Time for OverflowVec write_json: {:?}", start.elapsed());

    Ok(())
}
