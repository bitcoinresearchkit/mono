//! Snapshot of plugin versions, vectors and on-disk layout (`cargo surface [-- --check]`).

use std::process::ExitCode;

fn main() -> ExitCode {
    bitview_devtools::run("surface", || {
        bitview_devtools::render_surface(bitview_devtools::import()?)
    })
}
