//! Offline snapshots of Bitview's API and storage surface, used as refactoring gates.
//!
//! The tools import every plugin (default composition plus the optional ones) into a
//! temporary directory without a node and render deterministic text. Without arguments they
//! record a local baseline under `snapshots/` (gitignored; the previous one is kept as
//! `<file>.prev`); `--check` compares against it and writes `<file>.new` when they differ.
//! Versions are those of a fresh import: dependency versions applied by computation are
//! not included, so a changed version formula must be reviewed by hand.

mod api;
mod import;
mod snapshot;
mod surface;

pub use api::render_api;
pub use import::{AllPlugins, Imported, import};
pub use snapshot::{Snapshot, run};
pub use surface::render_surface;
