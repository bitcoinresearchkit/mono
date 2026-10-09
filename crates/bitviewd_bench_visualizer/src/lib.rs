mod chart;
mod data;
mod format;

use std::{
    error::Error,
    path::{Path, PathBuf},
};

use chart::{Series, TimeAxis, Unit};
use data::Run;

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

pub struct Visualizer {
    root: PathBuf,
}

impl Visualizer {
    /// `root` is a Bitview data directory (or the workspace) holding `benches/bitviewd/`.
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn from_cargo_env() -> Result<Self> {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .ok_or("Failed to find workspace root")?;
        Ok(Self::new(workspace))
    }

    /// One `dashboard.svg` per run, and `compare.svg` across complete runs.
    pub fn generate(&self) -> Result {
        let dir = self.root.join("benches").join("bitviewd");
        if !dir.exists() {
            return Err(format!("{} does not exist", dir.display()).into());
        }
        let runs = data::discover(&dir)?;
        for run in &runs {
            let output = run.path.join("dashboard.svg");
            dashboard(run, &output)?;
            println!("Generated: {}", output.display());
        }
        let complete = runs.iter().filter(|run| run.complete()).collect::<Vec<_>>();
        if !complete.is_empty() {
            let output = dir.join("compare.svg");
            compare(&complete, &output)?;
            println!("Generated: {}", output.display());
        }
        Ok(())
    }
}

enum Panel {
    Header(String, Vec<String>),
    Timeline,
    Lines(&'static str, Unit, Vec<Series>),
    Bars(&'static str, Unit, Vec<(String, f64)>),
    Table(
        &'static str,
        Vec<String>,
        Vec<Vec<String>>,
        Vec<plotters::style::RGBColor>,
    ),
}

/// Comparison charts draw the most recent complete runs; the summary table lists them all.
const COMPARED_LINES: usize = 6;

fn height(panel: &Panel, run: Option<&Run>) -> u32 {
    match panel {
        Panel::Header(_, lines) => chart::header_height(lines.len()),
        Panel::Timeline => run.map_or(0, chart::timeline_height),
        Panel::Lines(_, _, series) => chart::lines_height(series.len()),
        Panel::Bars(_, _, rows) => chart::bars_height(rows.len()),
        Panel::Table(_, _, body, _) => chart::table_height(body.len()),
    }
}

fn draw(
    panels: &[Panel],
    output: &Path,
    run: Option<&Run>,
    axis: &TimeAxis,
    cycles: &[data::Cycle],
) -> Result {
    let heights = panels
        .iter()
        .map(|panel| height(panel, run))
        .collect::<Vec<_>>();
    chart::render(output, &heights, |index, area| match &panels[index] {
        Panel::Header(title, lines) => chart::header(area, title, lines),
        Panel::Timeline => run.map_or(Ok(()), |run| chart::timeline(area, axis, run)),
        Panel::Lines(title, unit, series) => chart::lines(area, title, axis, cycles, *unit, series),
        Panel::Bars(title, unit, rows) => chart::bars(area, title, *unit, rows, chart::color(0)),
        Panel::Table(title, header, body, colors) => {
            chart::table(area, title, header, body, colors)
        }
    })
}

fn summary(run: &Run) -> Vec<String> {
    let meta = |key: &str| run.meta.get(key).filter(|value| !value.is_empty());
    let bytes = |key: &str| {
        meta(key)
            .and_then(|value| value.parse::<f64>().ok())
            .map(format::bytes)
    };
    let host = [
        meta("hostname").map(|host| format!("host {host}")),
        meta("cpu").map(|cpu| format!("cpu {cpu}")),
        bytes("ram_bytes").map(|ram| format!("ram {ram}")),
        meta("parallelism").map(|cores| format!("{cores} cores")),
        meta("revision").map(|revision| {
            let dirty = meta("dirty").is_some_and(|dirty| dirty == "true");
            format!(
                "revision {}{}",
                revision.get(..7).unwrap_or(revision),
                if dirty { " (dirty)" } else { "" }
            )
        }),
        meta("profile").map(|profile| format!("{profile} build")),
        bytes("cache_budget").map(|cache| format!("cache {cache}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    let mut totals = vec![format!(
        "{} in {}",
        run.status.as_deref().unwrap_or("unknown status"),
        format::duration(run.end_s())
    )];
    if let Some(height) = meta("chain_height") {
        totals.push(format!("chain height {height}"));
    }
    if let Some(peak) = run.memory.max("peak_physical_bytes") {
        totals.push(format!("peak footprint {}", format::bytes(peak)));
    }
    if let (Some(read), Some(written)) =
        (run.io.growth("read_bytes"), run.io.growth("written_bytes"))
    {
        totals.push(format!(
            "read {} / written {}",
            format::bytes(read),
            format::bytes(written)
        ));
    }
    if let Some(disk) = run.disk.last("physical_bytes") {
        totals.push(format!("data dir {}", format::bytes(disk)));
    }
    let cycles = run
        .cycles()
        .iter()
        .map(|cycle| {
            format!(
                "{} {}",
                cycle.name,
                format::duration(cycle.end_s - cycle.start_s)
            )
        })
        .collect::<Vec<_>>();
    let mut lines = vec![host.join(" · "), totals.join(" · ")];
    if !cycles.is_empty() {
        lines.push(cycles.join(" → "));
    }
    lines.retain(|line| !line.is_empty());
    lines
}

fn dashboard(run: &Run, output: &Path) -> Result {
    let axis = TimeAxis::new(run.end_s());
    let cycles = run.cycles();
    let mut panels = vec![Panel::Header(run.label.clone(), summary(run))];
    if !run.timings.is_empty() {
        panels.push(Panel::Timeline);
    }
    if run.progress.len() > 1 {
        panels.push(Panel::Lines(
            "Indexed height",
            Unit::Plain("height"),
            vec![Series {
                label: format!(
                    "indexer ({})",
                    format::count(run.progress.last().map_or(0.0, |(_, h)| *h))
                ),
                color: chart::color(0),
                dashed: false,
                points: run.progress.clone(),
            }],
        ));
    }
    let memory = [
        ("physical_bytes", "footprint", false, 0),
        ("peak_physical_bytes", "peak footprint", true, 0),
        ("resident_bytes", "resident", false, 3),
        ("swap_used_bytes", "swap (system)", false, 2),
    ]
    .into_iter()
    .filter(|(column, ..)| run.memory.has(column))
    .map(|(column, label, dashed, color)| Series {
        label: format!(
            "{label} (max {})",
            format::bytes(run.memory.max(column).unwrap_or_default())
        ),
        color: chart::color(color),
        dashed,
        points: run.memory.column(column),
    })
    .collect::<Vec<_>>();
    if !memory.is_empty() {
        panels.push(Panel::Lines("Memory", Unit::Bytes, memory));
    }
    if run.cpu.has("user_ms") {
        let cores = |column: &str| {
            chart::rate(&run.cpu.column(column))
                .into_iter()
                .map(|(time, ms)| (time, ms / 1_000.0))
                .collect::<Vec<_>>()
        };
        let total = |column: &str| run.cpu.last(column).unwrap_or_default() / 1_000.0;
        panels.push(Panel::Lines(
            "CPU (cores busy, 60 s average)",
            Unit::Plain("cores"),
            vec![
                Series {
                    label: format!("user ({})", format::duration(total("user_ms"))),
                    color: chart::color(4),
                    dashed: false,
                    points: cores("user_ms"),
                },
                Series {
                    label: format!("system ({})", format::duration(total("system_ms"))),
                    color: chart::color(2),
                    dashed: false,
                    points: cores("system_ms"),
                },
            ],
        ));
    }
    let io = [("read_bytes", "read", 0), ("written_bytes", "written", 1)]
        .into_iter()
        .filter(|(column, ..)| run.io.has(column))
        .map(|(column, label, color)| {
            let total = run.io.growth(column).unwrap_or_default();
            Series {
                label: format!("{label} ({})", format::bytes(total)),
                color: chart::color(color),
                dashed: false,
                points: chart::rate(&run.io.column(column)),
            }
        })
        .collect::<Vec<_>>();
    if !io.is_empty() {
        panels.push(Panel::Lines(
            "Disk I/O (60 s average)",
            Unit::BytesPerSecond,
            io,
        ));
    }
    if run.io.has("page_ins") {
        panels.push(Panel::Lines(
            "Page-ins (faults read from disk, 60 s average)",
            Unit::Plain("per second"),
            vec![Series {
                label: format!(
                    "page-ins ({})",
                    format::count(run.io.growth("page_ins").unwrap_or_default())
                ),
                color: chart::color(6),
                dashed: false,
                points: chart::rate(&run.io.column("page_ins")),
            }],
        ));
    }
    // Two samples (start and end) would draw a misleading straight line.
    if run.disk.column("physical_bytes").len() > 2 {
        panels.push(Panel::Lines(
            "Data directory size",
            Unit::Bytes,
            vec![Series {
                label: format!(
                    "data dir ({})",
                    format::bytes(run.disk.last("physical_bytes").unwrap_or_default())
                ),
                color: chart::color(5),
                dashed: false,
                points: run.disk.column("physical_bytes"),
            }],
        ));
    }
    let computes = run.plugin_seconds("compute");
    if !computes.is_empty() {
        panels.push(Panel::Bars(
            "Compute time per plugin",
            Unit::Plain("seconds"),
            computes,
        ));
    }
    let imports = run.plugin_seconds("import");
    if imports.iter().any(|(_, seconds)| *seconds >= 1.0) {
        panels.push(Panel::Bars(
            "Import time per plugin",
            Unit::Plain("seconds"),
            imports,
        ));
    }
    if !run.breakdown.is_empty() {
        panels.push(Panel::Bars(
            "Data directory by component",
            Unit::Bytes,
            run.breakdown.clone(),
        ));
    }
    draw(&panels, output, Some(run), &axis, &cycles)
}

fn compare(runs: &[&Run], output: &Path) -> Result {
    let charted = runs.len().saturating_sub(COMPARED_LINES);
    let axis = TimeAxis::new(
        runs[charted..]
            .iter()
            .map(|run| run.end_s())
            .fold(0.0, f64::max),
    );
    let colors = (0..runs.len()).map(chart::color).collect::<Vec<_>>();
    let name = |index: usize| format!("#{} {}", index + 1, runs[index].label);
    let series = |points: &dyn Fn(&Run) -> Vec<(f64, f64)>, value: &dyn Fn(&Run) -> String| {
        runs.iter()
            .enumerate()
            .skip(charted)
            .map(|(index, run)| Series {
                label: format!("#{} {}", index + 1, value(run)),
                color: colors[index],
                dashed: false,
                points: points(run),
            })
            .filter(|series| series.points.len() > 1)
            .collect::<Vec<_>>()
    };

    let mut panels = vec![Panel::Header(
        format!("bitviewd benchmarks: {} complete runs", runs.len()),
        vec![format!(
            "Charts show the {} most recent; times are from bootstrap start.",
            runs.len().min(COMPARED_LINES)
        )],
    )];
    let header = [
        "run",
        "duration",
        "indexing",
        "compute",
        "peak",
        "read",
        "written",
        "data dir",
        "cpu user/sys",
    ]
    .map(str::to_owned)
    .to_vec();
    let body = runs
        .iter()
        .enumerate()
        .map(|(index, run)| {
            let bytes = |value: Option<f64>| value.map_or("-".to_owned(), format::bytes);
            vec![
                name(index),
                format::duration(run.end_s()),
                format::duration(run.cycle_seconds("indexing")),
                format::duration(run.cycle_seconds("compute") + run.cycle_seconds("catch-up")),
                bytes(run.memory.max("peak_physical_bytes")),
                bytes(run.io.growth("read_bytes")),
                bytes(run.io.growth("written_bytes")),
                bytes(run.disk.last("physical_bytes")),
                match (run.cpu.growth("user_ms"), run.cpu.growth("system_ms")) {
                    (Some(user), Some(system)) => format!(
                        "{} / {}",
                        format::duration(user / 1_000.0),
                        format::duration(system / 1_000.0)
                    ),
                    _ => "-".to_owned(),
                },
            ]
        })
        .collect::<Vec<_>>();
    panels.push(Panel::Table("Summary", header, body, colors.clone()));

    let progress = series(&|run| run.progress.clone(), &|run| {
        format::count(run.progress.last().map_or(0.0, |(_, h)| *h))
    });
    if !progress.is_empty() {
        panels.push(Panel::Lines(
            "Indexed height",
            Unit::Plain("height"),
            progress,
        ));
    }
    panels.push(Panel::Lines(
        "Memory footprint (60 s average)",
        Unit::Bytes,
        series(
            &|run| chart::mean(&run.memory.column("physical_bytes")),
            &|run| {
                format!(
                    "peak {}",
                    format::bytes(run.memory.max("peak_physical_bytes").unwrap_or_default())
                )
            },
        ),
    ));
    let cpu = series(
        &|run| {
            let user = run.cpu.column("user_ms");
            let system = run.cpu.column("system_ms");
            let total = user
                .iter()
                .zip(&system)
                .map(|((time, user), (_, system))| (*time, user + system))
                .collect::<Vec<_>>();
            chart::rate(&total)
                .into_iter()
                .map(|(time, ms)| (time, ms / 1_000.0))
                .collect()
        },
        &|_| "user + system".to_owned(),
    );
    if !cpu.is_empty() {
        panels.push(Panel::Lines(
            "CPU (cores busy, 60 s average)",
            Unit::Plain("cores"),
            cpu,
        ));
    }
    panels.push(Panel::Lines(
        "Disk read (60 s average)",
        Unit::BytesPerSecond,
        series(&|run| chart::rate(&run.io.column("read_bytes")), &|run| {
            format::bytes(run.io.growth("read_bytes").unwrap_or_default())
        }),
    ));
    panels.push(Panel::Lines(
        "Disk write (60 s average)",
        Unit::BytesPerSecond,
        series(
            &|run| chart::rate(&run.io.column("written_bytes")),
            &|run| format::bytes(run.io.growth("written_bytes").unwrap_or_default()),
        ),
    ));

    // Plugins by their longest compute across the charted runs; one column per run.
    let mut plugins = Vec::<(String, f64)>::new();
    for run in &runs[charted..] {
        for (plugin, seconds) in run.plugin_seconds("compute") {
            match plugins.iter_mut().find(|(name, _)| *name == plugin) {
                Some((_, max)) => *max = max.max(seconds),
                None => plugins.push((plugin, seconds)),
            }
        }
    }
    plugins.sort_by(|a, b| b.1.total_cmp(&a.1));
    if !plugins.is_empty() {
        let header = std::iter::once("plugin (compute)".to_owned())
            .chain((charted..runs.len()).map(|index| format!("#{}", index + 1)))
            .collect::<Vec<_>>();
        let body = plugins
            .iter()
            .map(|(plugin, _)| {
                std::iter::once(plugin.clone())
                    .chain(runs[charted..].iter().map(|run| {
                        run.plugin_seconds("compute")
                            .iter()
                            .find(|(name, _)| name == plugin)
                            .map_or("-".to_owned(), |(_, seconds)| format::duration(*seconds))
                    }))
                    .collect()
            })
            .collect::<Vec<_>>();
        panels.push(Panel::Table(
            "Compute time per plugin",
            header,
            body,
            Vec::new(),
        ));
    }
    let panels = panels
        .into_iter()
        .filter(|panel| !matches!(panel, Panel::Lines(_, _, series) if series.is_empty()))
        .collect::<Vec<_>>();
    draw(&panels, output, None, &axis, &[])
}
