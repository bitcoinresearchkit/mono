use std::{fs, path::Path};

use serde_json::{Value, json};

use crate::{Result, data::Run};

/// The page, its data spliced in where the template says `__DATA__`.
const TEMPLATE: &str = include_str!("page.html");

/// Points kept per curve: enough for a screen's width, small enough for one file of every run.
const POINTS: usize = 400;

/// `index.html`: every machine's runs, their summaries and curves, browsable offline.
pub fn write(machines: &[(String, Vec<Run>)], output: &Path) -> Result {
    let data = json!({
        "machines": machines
            .iter()
            .map(|(name, runs)| json!({ "name": name, "runs": runs.iter().map(run).collect::<Vec<_>>() }))
            .collect::<Vec<_>>(),
    });
    // (Inside a script element, no `<` at all: `</script` would end it, `<!--` would change how it ends. JSON
    // has `<` only inside strings, where `\u003c` reads the same.)
    let data = serde_json::to_string(&data)?.replace('<', "\\u003c");
    fs::write(output, TEMPLATE.replace("__DATA__", &data))?;
    Ok(())
}

fn run(run: &Run) -> Value {
    let meta = |key: &str| run.meta.get(key).filter(|value| !value.is_empty());
    let number = |key: &str| meta(key).and_then(|value| value.parse::<f64>().ok());
    let peak = run.peak_memory();
    json!({
        "label": run.label,
        "started_at": run.started_at,
        "status": run.status,
        "duration_s": round(run.end_s(), 1),
        "revision": meta("revision"),
        "dirty": meta("dirty").is_some_and(|dirty| dirty == "true"),
        "host": {
            "cpu": meta("cpu"),
            "ram_bytes": number("ram_bytes"),
            "storage": meta("storage"),
            "cores": number("parallelism"),
            "os": meta("target_os"),
            "cache_bytes": number("cache_budget"),
        },
        "chain_height": number("chain_height"),
        "peak_memory": peak.map(|(name, bytes)| json!({ "name": name, "bytes": bytes })),
        "read_bytes": run.io.growth("read_bytes"),
        "written_bytes": run.io.growth("written_bytes"),
        "cpu_user_s": run.cpu.growth("user_ms").map(|ms| round(ms / 1_000.0, 1)),
        "cpu_system_s": run.cpu.growth("system_ms").map(|ms| round(ms / 1_000.0, 1)),
        "disk_bytes": run.disk.last("physical_bytes"),
        "cycles": run
            .cycles()
            .iter()
            .map(|cycle| json!({ "name": cycle.name, "start": round(cycle.start_s, 1), "end": round(cycle.end_s, 1) }))
            .collect::<Vec<_>>(),
        // Curves as [seconds, value]; the counters (CPU, I/O) cumulative, read as rates by the page.
        "curves": {
            "height": thin(&run.progress),
            "memory": thin(&run.memory.column("physical_bytes")),
            "cpu_user_ms": thin(&run.cpu.column("user_ms")),
            "cpu_system_ms": thin(&run.cpu.column("system_ms")),
            "read_bytes": thin(&run.io.column("read_bytes")),
            "written_bytes": thin(&run.io.column("written_bytes")),
            "disk_bytes": thin(&run.disk.column("physical_bytes")),
        },
        "compute": run.plugin_seconds("compute").into_iter().map(|(plugin, seconds)| json!([plugin, round(seconds, 2)])).collect::<Vec<_>>(),
        "import": run.plugin_seconds("import").into_iter().map(|(plugin, seconds)| json!([plugin, round(seconds, 2)])).collect::<Vec<_>>(),
        "breakdown": run.breakdown.iter().map(|(component, bytes)| json!([component, bytes])).collect::<Vec<_>>(),
    })
}

/// At most `POINTS` of a curve, evenly picked, its first and last kept.
fn thin(points: &[(f64, f64)]) -> Vec<[f64; 2]> {
    let pick = |index: usize| [round(points[index].0, 1), points[index].1];
    if points.len() <= POINTS {
        return (0..points.len()).map(pick).collect();
    }
    (0..POINTS)
        .map(|k| pick((k * (points.len() - 1) + (POINTS - 1) / 2) / (POINTS - 1)))
        .collect()
}

fn round(value: f64, digits: i32) -> f64 {
    let scale = 10f64.powi(digits);
    (value * scale).round() / scale
}
