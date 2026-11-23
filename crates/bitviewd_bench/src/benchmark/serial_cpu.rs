//! The serial CPU runs, once the timed run has finished, each with the composition's forks run one
//! after the other, so a plugin's process CPU is its own (`bitview_default::serial`): the replay of
//! the run's last `UNDO_TAIL` blocks one at a time, then every plugin but the indexer computed again
//! from zero over the run's indexed chain.

use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use bitview::{ImportContext, UpdateContext, update};
use bitview_compute::UNDO_TAIL;
use bitview_default::{DefaultPlugins, serial};
use brk_error::{Error, Result};
use brk_types::Height;

use super::process::cpu_micros;

pub fn run(
    path: &Path,
    mut plugins: DefaultPlugins,
    import_context: ImportContext<'_>,
    context: UpdateContext<'_>,
    sync: impl FnOnce() -> Result<DefaultPlugins>,
) -> Result<()> {
    serial::start(|| cpu_micros().expect("reading the process CPU time") * 1_000);
    replay(path, &mut plugins, context)?;
    plugins.remove_computed(import_context)?;
    recompute(path, sync)
}

fn replay(path: &Path, plugins: &mut DefaultPlugins, context: UpdateContext<'_>) -> Result<()> {
    let tip = usize::from(
        plugins
            .tip_height()
            .ok_or(Error::Internal("Replay needs an indexed chain"))?,
    );
    if tip < UNDO_TAIL {
        return Err(Error::Internal(
            "Replay needs a chain at least as long as the undo tail",
        ));
    }
    // The first block restores the plugins' live state and is not measured.
    let first = tip + 1 - UNDO_TAIL;
    plugins.rewind_for_replay(Height::from(first), context)?;

    let mut raw = BufWriter::new(File::create(path.join("serial_cpu_samples.csv"))?);
    writeln!(raw, "height,plugin,serial_cpu_ms")?;
    for height in first..=tip {
        let tip_height = Height::from(height);
        plugins.cap_tip(tip_height);
        let start = cpu_micros()?;
        let published = update(plugins, context)?;
        let total = (cpu_micros()? - start) * 1_000;
        let steps = serial::take();
        if !published || plugins.tip_height() != Some(tip_height) {
            return Err(Error::Internal("Replay did not publish the next block"));
        }
        if height != first {
            rows(&mut raw, &format!("{height},"), steps, total)?;
        }
    }
    Ok(raw.flush()?)
}

/// Each plugin's CPU from zero: the bootstrap `sync` runs on the indexed chain alone, so every
/// other plugin computes all of it again, one at a time.
fn recompute(path: &Path, sync: impl FnOnce() -> Result<DefaultPlugins>) -> Result<()> {
    let start = cpu_micros()?;
    drop(sync()?);
    let total = (cpu_micros()? - start) * 1_000;
    let mut raw = BufWriter::new(File::create(path.join("serial_compute.csv"))?);
    writeln!(raw, "plugin,serial_cpu_ms")?;
    rows(&mut raw, "", serial::take(), total)?;
    Ok(raw.flush()?)
}

/// One row per plugin, then `total` (the whole span's process CPU) and `unattributed` (`total`
/// less the plugins: the schedule, imports, logging and helper threads between steps).
fn rows(
    raw: &mut impl Write,
    prefix: &str,
    steps: BTreeMap<&'static str, u64>,
    total: u64,
) -> Result<()> {
    let unattributed = total
        .checked_sub(steps.values().sum::<u64>())
        .ok_or(Error::Internal("The plugins' serial CPU exceeds the whole span's"))?;
    for (plugin, cpu) in steps
        .into_iter()
        .chain([("total", total), ("unattributed", unattributed)])
    {
        writeln!(raw, "{prefix}{plugin},{:.3}", cpu as f64 / 1_000_000.0)?;
    }
    Ok(())
}
