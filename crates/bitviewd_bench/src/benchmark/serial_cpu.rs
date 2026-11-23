//! The serial CPU recompute, once the timed run has finished: every plugin but the indexer
//! computed again from zero over the run's indexed chain, the composition's forks run one after
//! the other, so a plugin's process CPU is its own (`bitview_default::serial`).

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use bitview::ImportContext;
use bitview_default::{DefaultPlugins, serial};
use brk_error::{Error, Result};

use super::process::cpu_micros;

/// Drops the timed run's `plugins` with every plugin's data but the indexer's, then runs the
/// bootstrap `sync` again, serially, and writes each plugin's CPU from zero, the whole
/// bootstrap's (`total`, the imports included) and what no plugin accounts for (`unattributed`:
/// the imports, the schedule, logging and helpers between steps).
pub fn recompute(
    path: &Path,
    plugins: DefaultPlugins,
    import_context: ImportContext<'_>,
    sync: impl FnOnce() -> Result<DefaultPlugins>,
) -> Result<()> {
    plugins.remove_computed(import_context)?;
    serial::start(|| cpu_micros().expect("reading the process CPU time") * 1_000);
    let start = cpu_micros()?;
    drop(sync()?);
    let total = (cpu_micros()? - start) * 1_000;
    let steps = serial::take();
    let unattributed = total
        .checked_sub(steps.values().sum::<u64>())
        .ok_or(Error::Internal("The plugins' serial CPU exceeds the recompute's"))?;

    let mut raw = BufWriter::new(File::create(path.join("serial_compute.csv"))?);
    writeln!(raw, "plugin,serial_cpu_ms")?;
    for (plugin, cpu) in steps
        .into_iter()
        .chain([("total", total), ("unattributed", unattributed)])
    {
        writeln!(raw, "{plugin},{:.3}", cpu as f64 / 1_000_000.0)?;
    }
    Ok(raw.flush()?)
}
