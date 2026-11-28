#![doc = include_str!("../README.md")]

use std::process::ExitCode;

use bitview::run;
use bitview_default::DefaultPlugins;

#[global_allocator]
static GLOBAL: brk_alloc::MiMalloc = brk_alloc::MiMalloc;

/// A build with debug assertions says so at `/version`: only it checks the invariants the
/// release builds leave out (hash identity among them), so whoever watches a server for that check
/// reads it from the server itself.
const VERSION: &str = if cfg!(debug_assertions) {
    concat!(env!("BITVIEW_VERSION"), "-debug")
} else {
    env!("BITVIEW_VERSION")
};

fn main() -> ExitCode {
    bitview_default::build_thread_pool();
    bitview::set_build(VERSION, env!("BITVIEW_SOURCE_TAG"));
    match run(DefaultPlugins::import) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            if error.is_lock_error() {
                eprintln!("If another bitviewd instance holds the lock, stop it and try again.");
            }
            ExitCode::FAILURE
        }
    }
}
