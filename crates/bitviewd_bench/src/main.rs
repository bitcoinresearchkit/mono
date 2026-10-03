use benchmark::Benchmark;
use bitview::{Config as RunnerConfig, ImportContext, UpdateContext, bootstrap};
use bitview_default::DefaultPlugins;
use bitviewd::Config;
use brk_error::Result;
use brk_exit::Exit;
use brk_logger::init;
use brk_reader::Reader;
use std::process::ExitCode;
use tracing::info;
use vecdb::Budgeted;

mod benchmark;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let RunnerConfig {
        client,
        blocks_path,
        server,
        cache_budget,
    } = Config::import()?;
    Budgeted::init_global(cache_budget)?;
    init(Some(&server.logs_path()))?;
    let data_path = server.data_path;
    client.wait_for_synced_node()?;

    let chain_height = client.get_last_height()?;
    let benchmark = Benchmark::new(&data_path, &blocks_path, chain_height, cache_budget)?;
    info!("Benchmark results: {}", benchmark.path().display());
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
