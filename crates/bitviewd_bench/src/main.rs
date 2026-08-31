use bitview::{Config as RunnerConfig, ImportContext, UpdateContext, bootstrap};
use bitview_default::DefaultPlugins;
use bitviewd::Config;
use brk_error::Result;
use brk_exit::Exit;
use brk_reader::Reader;
use tracing::info;

mod benchmark;

use benchmark::Benchmark;

fn main() -> Result<()> {
    let RunnerConfig {
        client,
        blocks_path,
        server,
    } = Config::import()?;
    let data_path = server.data_path;

    brk_logger::init(Some(&data_path.join("logs")))?;
    client.wait_for_synced_node()?;

    let chain_height = client.get_last_height()?;
    let benchmark = Benchmark::new(&data_path, &blocks_path, chain_height)?;
    let reader = Reader::new(blocks_path, &client);
    let exit = Exit::new();
    exit.set_ctrlc_handler();

    let cleanup = benchmark.clone();
    exit.register_cleanup(move || {
        let _ = cleanup.abort();
    });

    benchmark.measure(|| {
        bootstrap(
            ImportContext::new(&data_path),
            |context| DefaultPlugins::import(context, &reader),
            UpdateContext::new(&exit),
        )
    })?;
    info!("Benchmark saved to {}", benchmark.path().display());
    Ok(())
}
