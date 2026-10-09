use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use crate::format;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// One recorded benchmark run; every file but `memory.csv` is optional (older runs lack them).
pub struct Run {
    pub path: PathBuf,
    pub label: String,
    pub started_at: u64,
    pub meta: BTreeMap<String, String>,
    pub status: Option<String>,
    pub duration_s: Option<f64>,
    pub memory: Table,
    pub io: Table,
    pub cpu: Table,
    pub disk: Table,
    /// Indexer heights only.
    pub progress: Vec<(f64, f64)>,
    pub timings: Vec<Timing>,
    pub breakdown: Vec<(String, f64)>,
}

pub struct Timing {
    pub phase: String,
    pub plugin: String,
    pub start_s: f64,
    pub duration_s: f64,
}

/// A bootstrap cycle starts with each indexer import: indexing, the full compute, catch-ups.
pub struct Cycle {
    pub start_s: f64,
    pub end_s: f64,
    pub name: &'static str,
}

/// A CSV read by header name; a malformed trailing row (a killed recorder) is dropped.
struct Csv {
    header: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Csv {
    fn read(path: &Path) -> Option<Self> {
        let content = fs::read_to_string(path).ok()?;
        let mut lines = content.lines();
        let header = lines
            .next()?
            .split(',')
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let mut rows = lines
            .filter(|line| !line.is_empty())
            .map(|line| line.split(',').map(str::to_owned).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        if rows
            .last()
            .is_some_and(|row| row.len() != header.len() || row.iter().any(String::is_empty))
        {
            rows.pop();
        }
        Some(Self { header, rows })
    }

    fn index(&self, name: &str) -> Option<usize> {
        self.header.iter().position(|column| column == name)
    }

    fn numbers(&self, path: &Path, name: &str) -> Result<Vec<f64>> {
        let Some(index) = self.index(name) else {
            return Ok(Vec::new());
        };
        self.rows
            .iter()
            .map(|row| {
                let field = row.get(index).map(String::as_str).unwrap_or_default();
                field.parse::<f64>().map_err(|error| {
                    format!("{}: {name} {field:?}: {error}", path.display()).into()
                })
            })
            .collect()
    }

    fn strings(&self, name: &str) -> Vec<String> {
        self.index(name).map_or_else(Vec::new, |index| {
            self.rows
                .iter()
                .map(|row| row.get(index).cloned().unwrap_or_default())
                .collect()
        })
    }
}

/// A time series file: `timestamp_ms` plus value columns, read by header name.
#[derive(Default)]
pub struct Table {
    times: Vec<f64>,
    columns: BTreeMap<String, Vec<f64>>,
}

impl Table {
    fn read(path: &Path) -> Result<Self> {
        let Some(csv) = Csv::read(path) else {
            return Ok(Self::default());
        };
        let times = csv
            .numbers(path, "timestamp_ms")?
            .into_iter()
            .map(|ms| ms / 1_000.0)
            .collect();
        let columns = csv
            .header
            .iter()
            .filter(|column| *column != "timestamp_ms")
            .map(|column| Ok((column.clone(), csv.numbers(path, column)?)))
            .collect::<Result<_>>()?;
        Ok(Self { times, columns })
    }

    /// (seconds, value) points of a column, empty when the run did not record it.
    pub fn column(&self, name: &str) -> Vec<(f64, f64)> {
        self.columns.get(name).map_or_else(Vec::new, |values| {
            self.times
                .iter()
                .copied()
                .zip(values.iter().copied())
                .collect()
        })
    }

    pub fn has(&self, name: &str) -> bool {
        self.columns.contains_key(name) && self.times.len() > 1
    }

    pub fn last(&self, name: &str) -> Option<f64> {
        self.columns.get(name)?.last().copied()
    }

    /// Growth of a cumulative counter over the run: the counters start before bootstrap.
    pub fn growth(&self, name: &str) -> Option<f64> {
        let values = self.columns.get(name)?;
        Some(values.last()? - values.first()?)
    }

    pub fn max(&self, name: &str) -> Option<f64> {
        self.columns.get(name)?.iter().copied().reduce(f64::max)
    }
}

impl Run {
    pub fn complete(&self) -> bool {
        self.status.as_deref() == Some("complete")
    }

    pub fn end_s(&self) -> f64 {
        self.duration_s.unwrap_or_else(|| {
            self.memory
                .column("physical_bytes")
                .last()
                .map_or(0.0, |(time, _)| *time)
        })
    }

    /// Indexing when the indexer takes most of the cycle; the first other cycle computes, later
    /// ones catch up.
    pub fn cycles(&self) -> Vec<Cycle> {
        let starts = self
            .timings
            .iter()
            .filter(|timing| timing.phase == "import" && timing.plugin == "indexer")
            .map(|timing| timing.start_s)
            .collect::<Vec<_>>();
        let end = self.end_s();
        let mut computed = false;
        starts
            .iter()
            .enumerate()
            .map(|(index, &start_s)| {
                let end_s = starts.get(index + 1).copied().unwrap_or(end).max(start_s);
                let indexing = self
                    .timings
                    .iter()
                    .filter(|timing| {
                        timing.phase == "compute"
                            && timing.plugin == "indexer"
                            && timing.start_s >= start_s
                            && timing.start_s < end_s
                    })
                    .map(|timing| timing.duration_s)
                    .fold(0.0, |total, seconds| total + seconds);
                let name = if indexing * 2.0 >= end_s - start_s {
                    "indexing"
                } else if computed {
                    "catch-up"
                } else {
                    computed = true;
                    "compute"
                };
                Cycle {
                    start_s,
                    end_s,
                    name,
                }
            })
            .collect()
    }

    /// Seconds spent in cycles of that name.
    pub fn cycle_seconds(&self, name: &str) -> f64 {
        self.cycles()
            .iter()
            .filter(|cycle| cycle.name == name)
            .fold(0.0, |total, cycle| total + cycle.end_s - cycle.start_s)
    }

    /// Seconds per plugin and phase, summed over cycles, longest first.
    pub fn plugin_seconds(&self, phase: &str) -> Vec<(String, f64)> {
        let mut totals = BTreeMap::<&str, f64>::new();
        for timing in self.timings.iter().filter(|timing| timing.phase == phase) {
            *totals.entry(&timing.plugin).or_default() += timing.duration_s;
        }
        let mut totals = totals
            .into_iter()
            .map(|(plugin, seconds)| (plugin.to_owned(), seconds))
            .collect::<Vec<_>>();
        totals.sort_by(|a, b| b.1.total_cmp(&a.1));
        totals
    }
}

/// Every directory at or below `dir` holding a `memory.csv`, oldest first. A run that cannot be
/// read is reported and skipped, so it never hides the others.
pub fn discover(dir: &Path) -> Result<Vec<Run>> {
    let mut paths = Vec::new();
    collect(dir, &mut paths)?;
    let mut runs = paths
        .into_iter()
        .filter_map(|path| match read(dir, path.clone()) {
            Ok(run) => Some(run),
            Err(error) => {
                eprintln!("Skipping {}: {error}", path.display());
                None
            }
        })
        .collect::<Vec<_>>();
    runs.sort_by(|a, b| a.started_at.cmp(&b.started_at).then(a.label.cmp(&b.label)));
    Ok(runs)
}

fn collect(dir: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    if dir.join("memory.csv").exists() {
        paths.push(dir.to_path_buf());
    }
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(&path, paths)?;
        }
    }
    Ok(())
}

fn read(root: &Path, path: PathBuf) -> Result<Run> {
    let meta = fs::read_to_string(path.join("metadata.txt"))
        .unwrap_or_default()
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect::<BTreeMap<_, _>>();
    let relative = path.strip_prefix(root).unwrap_or(&path);
    let started_at = meta
        .get("started_at_unix")
        .and_then(|value| value.parse().ok())
        .or_else(|| {
            relative
                .file_name()?
                .to_str()?
                .strip_prefix("run-")?
                .parse::<u64>()
                .ok()
                .map(|ms| ms / 1_000)
        })
        .unwrap_or(0);
    let label = label(relative, started_at);

    let run = Csv::read(&path.join("run.csv"));
    let status = run.as_ref().and_then(|run| run.strings("status").pop());
    let duration_s = match &run {
        Some(run) => run
            .numbers(&path, "duration_ms")?
            .pop()
            .map(|ms| ms / 1_000.0),
        None => None,
    };

    Ok(Run {
        memory: Table::read(&path.join("memory.csv"))?,
        io: Table::read(&path.join("io.csv"))?,
        cpu: Table::read(&path.join("cpu.csv"))?,
        disk: Table::read(&path.join("disk.csv"))?,
        progress: progress(&path.join("progress.csv"))?,
        timings: timings(&path.join("timings.csv"))?,
        breakdown: breakdown(&path.join("disk_breakdown.csv"))?,
        path,
        label,
        started_at,
        meta,
        status,
        duration_s,
    })
}

/// The run's directories with 40-hex commits shortened, plus its start time, which tells apart
/// runs of one commit.
fn label(relative: &Path, started_at: u64) -> String {
    let hash = |part: &str| part.len() == 40 && part.bytes().all(|byte| byte.is_ascii_hexdigit());
    let parts = relative
        .iter()
        .filter_map(|part| part.to_str())
        .map(|part| match part.rsplit_once('-') {
            Some((head, commit)) if hash(commit) => format!("{head}-{}", &commit[..7]),
            _ if hash(part) => part[..7].to_owned(),
            _ => part.to_owned(),
        })
        .filter(|part| !part.starts_with("run-") || relative.iter().count() == 1)
        .collect::<Vec<_>>();
    let name = if parts.is_empty() {
        "run".to_owned()
    } else {
        parts.join("/")
    };
    if started_at == 0 {
        name
    } else {
        format!("{name} ({})", format::datetime(started_at))
    }
}

/// Indexer heights: the `source` column when recorded, else the rows before the first drop
/// (older files mixed compute-phase logs in after indexing).
fn progress(path: &Path) -> Result<Vec<(f64, f64)>> {
    let Some(csv) = Csv::read(path) else {
        return Ok(Vec::new());
    };
    let times = csv.numbers(path, "timestamp_ms")?;
    let heights = csv.numbers(path, "height")?;
    let sources = csv.strings("source");
    let mut points = Vec::new();
    for (index, (time, height)) in times.into_iter().zip(heights).enumerate() {
        if sources.is_empty() {
            if points.last().is_some_and(|(_, last)| height < *last) {
                break;
            }
        } else if sources[index] != "indexer" {
            continue;
        }
        points.push((time / 1_000.0, height));
    }
    Ok(points)
}

fn timings(path: &Path) -> Result<Vec<Timing>> {
    let Some(csv) = Csv::read(path) else {
        return Ok(Vec::new());
    };
    let starts = csv.numbers(path, "start_ms")?;
    let durations = csv.numbers(path, "duration_ms")?;
    Ok(csv
        .strings("phase")
        .into_iter()
        .zip(csv.strings("plugin"))
        .zip(starts.into_iter().zip(durations))
        .map(|((phase, plugin), (start_ms, duration_ms))| Timing {
            phase,
            plugin,
            start_s: start_ms / 1_000.0,
            duration_s: duration_ms / 1_000.0,
        })
        .collect())
}

fn breakdown(path: &Path) -> Result<Vec<(String, f64)>> {
    let Some(csv) = Csv::read(path) else {
        return Ok(Vec::new());
    };
    let mut rows = csv
        .strings("component")
        .into_iter()
        .zip(csv.numbers(path, "physical_bytes")?)
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| b.1.total_cmp(&a.1));
    Ok(rows)
}
