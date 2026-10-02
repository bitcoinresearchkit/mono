//! Snapshot of the series tree and OpenAPI document (`cargo api [-- --check]`).

use std::process::ExitCode;

fn main() -> ExitCode {
    bitview_devtools::run("api", || {
        bitview_devtools::render_api(&bitview_devtools::import()?.plugins)
    })
}
