#![doc = include_str!("../README.md")]

use std::process::ExitCode;

use bitview::run;
use bitview_default::DefaultPlugins;

#[global_allocator]
static GLOBAL: brk_alloc::MiMalloc = brk_alloc::MiMalloc;

fn main() -> ExitCode {
    bitview::set_etag_build_tag(env!("BITVIEW_SOURCE_TAG"));
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
