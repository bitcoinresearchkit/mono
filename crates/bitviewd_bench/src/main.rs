use benchmark::Benchmark;
use bitview::{Config, ImportContext, RunConfig, UpdateContext, raise_fd_limit};
use bitview_default::DefaultPlugins;
use brk_error::Result;
use brk_exit::Exit;
use brk_logger::init;
use std::process::ExitCode;
use tracing::info;
use vecdb::Budgeted;

mod benchmark;

#[global_allocator]
static GLOBAL: brk_alloc::MiMalloc = brk_alloc::MiMalloc;

fn main() -> ExitCode {
    bitview_default::build_thread_pool();
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let RunConfig {
        reader,
        server,
        cache_budget,
    } = Config::import()?;
    Budgeted::init_global(cache_budget)?;
    init(Some(&server.logs_path()))?;
    raise_fd_limit();
    let data_path = server.data_path;
    let client = reader.client();
    client.wait_for_synced_node()?;

    let chain_height = client.get_last_height()?;
    let benchmark = Benchmark::new(
        &data_path,
        reader.blocks_dir(),
        chain_height,
        cache_budget,
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

    benchmark.measure(
        ImportContext::new(&data_path, &exit),
        UpdateContext::new(&exit),
        |context| DefaultPlugins::import(context, &reader),
    )?;
    info!("Benchmark saved to {}", benchmark.path().display());
    Ok(())
}
