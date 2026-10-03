#![doc = include_str!("../README.md")]

use std::process::ExitCode;

use bitview::run;
use bitview_default::DefaultPlugins;

#[global_allocator]
static GLOBAL: brk_alloc::MiMalloc = brk_alloc::MiMalloc;

fn main() -> ExitCode {
    match run(DefaultPlugins::import) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if error.is_lock_error() {
                eprintln!(
                    "Error: Bitview's data directory is already in use by another process.\n\
                     Stop the other bitviewd instance and try again."
                );
            } else {
                eprintln!("Error: {error}");
            }
            ExitCode::FAILURE
        }
    }
}
