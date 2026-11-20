use benchmark::Benchmark;
use bitview::{Config, ImportContext, RunConfig, UpdateContext, bootstrap, raise_fd_limit};
use bitview_default::DefaultPlugins;
use brk_error::Result;
use brk_exit::Exit;
use brk_logger::init;
use brk_reader::Reader;
use std::process::ExitCode;
use tracing::info;
use vecdb::Budgeted;

mod benchmark;
mod quiet;

#[global_allocator]
static GLOBAL: brk_alloc::MiMalloc = brk_alloc::MiMalloc;

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
    let quiet = quiet::acquire()?;
    let RunConfig {
        client,
        blocks_path,
        server,
        cache_budget,
    } = Config::import()?;
    quiet::check_disks(&server.data_path, &blocks_path)?;
    Budgeted::init_global(cache_budget)?;
    init(Some(&server.logs_path()))?;
    raise_fd_limit();
    let data_path = server.data_path;
    let reader = Reader::new(blocks_path, &client)?;
    reader.check_network()?;
    client.wait_for_synced_node()?;

    let chain_height = client.get_last_height()?;
    let benchmark = Benchmark::new(
        &data_path,
        reader.blocks_dir(),
        chain_height,
        cache_budget,
        &quiet.busiest,
    )?;
    info!("Benchmark results: {}", benchmark.path().display());
    let exit = Exit::new();
    exit.install_signal_handlers();

    let cleanup = benchmark.clone();
    exit.register_cleanup(move || {
        if let Err(error) = cleanup.abort() {
            eprintln!("Failed to save interrupted benchmark to {}: {error}", cleanup.path().display());
        }
    });

    let update_context = UpdateContext::new(&exit);
    let mut plugins = benchmark.measure(|| {
        bootstrap(
            ImportContext::new(&data_path, &exit),
            |context| DefaultPlugins::import(context, &reader),
            update_context,
        )
    })?;
    benchmark.replay(&mut plugins, update_context)?;
    info!("Benchmark saved to {}", benchmark.path().display());
    Ok(())
}
