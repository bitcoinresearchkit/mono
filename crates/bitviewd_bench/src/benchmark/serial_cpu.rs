//! The serial recompute, once the timed run has finished: every plugin but the indexer
//! computed again from zero over the run's indexed chain, the composition's forks run one after
//! the other, so a plugin's process counters (retired instructions, cycles, CPU time) are its own
//! (`bitview_default::serial`).

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use bitview::ImportContext;
use bitview_default::{DefaultPlugins, serial, serial::Counts};
use brk_error::{Error, Result};

use super::process::counts;

/// Drops the timed run's `plugins` with every plugin's data but the indexer's, then runs the
/// bootstrap `sync` again, serially, and writes each plugin's counters from zero, the whole
/// bootstrap's (`total`, the imports included) and what no plugin accounts for (`unattributed`:
/// the imports, the schedule, logging and helpers between steps).
pub fn recompute(
    path: &Path,
    plugins: DefaultPlugins,
    import_context: ImportContext<'_>,
    sync: impl FnOnce() -> Result<DefaultPlugins>,
) -> Result<()> {
    plugins.remove_computed(import_context)?;
    serial::start(|| counts().expect("reading the process counters"));
    let start = counts()?;
    drop(sync()?);
    let total = counts()? - start;
    let steps = serial::take();
    let accounted: Counts = steps.values().copied().sum();
    if accounted.instructions > total.instructions
        || accounted.cycles > total.cycles
        || accounted.cpu_ns > total.cpu_ns
    {
        return Err(Error::Internal("The plugins' serial counters exceed the recompute's"));
    }

    let mut raw = BufWriter::new(File::create(path.join("serial_compute.csv"))?);
    writeln!(raw, "plugin,instructions,cycles,cpu_ms")?;
    for (plugin, spent) in steps.into_iter().chain([
        ("total", total),
        ("unattributed", total - accounted),
    ]) {
        writeln!(
            raw,
            "{plugin},{},{},{:.3}",
            spent.instructions,
            spent.cycles,
            spent.cpu_ns as f64 / 1_000_000.0
        )?;
    }
    Ok(raw.flush()?)
}
