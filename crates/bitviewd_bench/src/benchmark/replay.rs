//! The serial CPU replay: once the timed run has finished, its last `UNDO_TAIL` blocks are
//! rolled back and applied again one at a time, each update with the composition's forks run one
//! after the other, so a plugin's process CPU is its own (`bitview_default::serial`).

use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use bitview::{UpdateContext, update};
use bitview_compute::UNDO_TAIL;
use bitview_default::{DefaultPlugins, serial};
use brk_error::{Error, Result};
use brk_types::Height;

use super::process::cpu_micros;

/// The rows of a block's samples besides the plugins': the whole update's CPU, and what no
/// plugin's step accounts for (the schedule, logging, helper threads between steps).
const TOTAL: &str = "total";
const UNATTRIBUTED: &str = "unattributed";

pub fn run(
    path: &Path,
    plugins: &mut DefaultPlugins,
    context: UpdateContext<'_>,
) -> Result<()> {
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
    serial::start(|| cpu_micros().expect("reading the process CPU time") * 1_000);

    let mut samples = BTreeMap::<String, Vec<u64>>::new();
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
        if height == first {
            continue;
        }
        let unattributed = total
            .checked_sub(steps.values().sum::<u64>())
            .ok_or(Error::Internal("The plugins' serial CPU exceeds the update's"))?;
        let rows = steps
            .into_iter()
            .chain([(TOTAL, total), (UNATTRIBUTED, unattributed)]);
        for (plugin, cpu) in rows {
            writeln!(raw, "{height},{plugin},{}", millis(cpu))?;
            samples.entry(plugin.to_owned()).or_default().push(cpu);
        }
    }
    raw.flush()?;

    let mut summary = BufWriter::new(File::create(path.join("serial_cpu.csv"))?);
    writeln!(
        summary,
        "plugin,first_height,last_height,samples,p50_serial_cpu_ms,p95_serial_cpu_ms"
    )?;
    for (plugin, mut cpu) in samples {
        cpu.sort_unstable();
        writeln!(
            summary,
            "{plugin},{},{tip},{},{},{}",
            first + 1,
            cpu.len(),
            millis(nearest_rank(&cpu, 50)),
            millis(nearest_rank(&cpu, 95)),
        )?;
    }
    Ok(summary.flush()?)
}

/// The value at `percentile` of the sorted `values`, nearest rank.
fn nearest_rank(values: &[u64], percentile: usize) -> u64 {
    values[(values.len() * percentile).div_ceil(100) - 1]
}

fn millis(ns: u64) -> String {
    format!("{:.3}", ns as f64 / 1_000_000.0)
}
