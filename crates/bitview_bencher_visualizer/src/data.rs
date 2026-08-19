use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Clone)]
pub struct DataPoint {
    pub timestamp_ms: u64,
    pub value: f64,
}

/// Per-run cutoff timestamps for fair comparison
pub struct Cutoffs {
    by_id: HashMap<String, u64>,
    default: u64,
}

impl Cutoffs {
    /// Calculate cutoffs from progress runs.
    /// Finds the common max progress, then returns when each run reached it.
    pub fn from_progress(progress_runs: &[Run]) -> Self {
        const TIME_BUFFER_MS: u64 = 10_000;

        if progress_runs.is_empty() {
            return Self {
                by_id: HashMap::new(),
                default: u64::MAX,
            };
        }

        // Find the minimum of max progress values (the common point all runs reached)
        let common_progress = progress_runs
            .iter()
            .map(|r| r.max_value())
            .fold(f64::MAX, f64::min);

        let by_id: HashMap<_, _> = progress_runs
            .iter()
            .map(|run| {
                let cutoff = run
                    .data
                    .iter()
                    .find(|d| d.value >= common_progress)
                    .map(|d| d.timestamp_ms)
                    .unwrap_or_else(|| run.max_timestamp())
                    .saturating_add(TIME_BUFFER_MS);
                (run.id.clone(), cutoff)
            })
            .collect();

        let default = by_id.values().copied().max().unwrap_or(u64::MAX);

        Self { by_id, default }
    }

    pub fn get(&self, id: &str) -> u64 {
        self.by_id.get(id).copied().unwrap_or(self.default)
    }

    pub fn trim_runs(&self, runs: &[Run]) -> Vec<Run> {
        runs.iter().map(|r| r.trimmed(self.get(&r.id))).collect()
    }

    pub fn trim_dual_runs(&self, runs: &[DualRun]) -> Vec<DualRun> {
        runs.iter().map(|r| r.trimmed(self.get(&r.id))).collect()
    }
}

#[derive(Debug, Clone)]
pub struct Run {
    pub id: String,
    pub data: Vec<DataPoint>,
}

impl Run {
    pub fn max_timestamp(&self) -> u64 {
        self.data.iter().map(|d| d.timestamp_ms).max().unwrap_or(0)
    }

    pub fn max_value(&self) -> f64 {
        self.data.iter().map(|d| d.value).fold(0.0, f64::max)
    }

    pub fn trimmed(&self, max_timestamp_ms: u64) -> Self {
        Self {
            id: self.id.clone(),
            data: self
                .data
                .iter()
                .filter(|d| d.timestamp_ms <= max_timestamp_ms)
                .cloned()
                .collect(),
        }
    }
}

/// Two data series from a single run (e.g., memory footprint + peak, or io read + write)
#[derive(Debug, Clone)]
pub struct DualRun {
    pub id: String,
    pub primary: Vec<DataPoint>,
    pub secondary: Vec<DataPoint>,
}

impl DualRun {
    pub fn trimmed(&self, max_timestamp_ms: u64) -> Self {
        Self {
            id: self.id.clone(),
            primary: self
                .primary
                .iter()
                .filter(|d| d.timestamp_ms <= max_timestamp_ms)
                .cloned()
                .collect(),
            secondary: self
                .secondary
                .iter()
                .filter(|d| d.timestamp_ms <= max_timestamp_ms)
                .cloned()
                .collect(),
        }
    }

    pub fn max_value(&self) -> f64 {
        self.primary
            .iter()
            .chain(self.secondary.iter())
            .map(|d| d.value)
            .fold(0.0, f64::max)
    }
}

pub fn read_runs(
    crate_path: &Path,
    filename: &str,
) -> Result<Vec<Run>, Box<dyn std::error::Error>> {
    let mut runs = Vec::new();

    for entry in fs::read_dir(crate_path)? {
        let run_path = entry?.path();
        if !run_path.is_dir() {
            continue;
        }

        let run_id = run_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("Invalid run ID")?
            .to_string();

        // Skip underscore-prefixed or numeric-only directories
        if run_id.starts_with('_') || run_id.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        let csv_path = run_path.join(filename);
        if csv_path.exists()
            && let Ok(data) = read_csv(&csv_path)
        {
            runs.push(Run { id: run_id, data });
        }
    }

    Ok(runs)
}

pub fn read_dual_runs(
    crate_path: &Path,
    filename: &str,
) -> Result<Vec<DualRun>, Box<dyn std::error::Error>> {
    let mut runs = Vec::new();

    for entry in fs::read_dir(crate_path)? {
        let run_path = entry?.path();
        if !run_path.is_dir() {
            continue;
        }

        let run_id = run_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("Invalid run ID")?
            .to_string();

        if run_id.starts_with('_') || run_id.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        let csv_path = run_path.join(filename);
        if csv_path.exists()
            && let Ok((primary, secondary)) = read_dual_csv(&csv_path)
        {
            runs.push(DualRun {
                id: run_id,
                primary,
                secondary,
            });
        }
    }

    Ok(runs)
}

fn read_csv(path: &Path) -> Result<Vec<DataPoint>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let data = content
        .lines()
        .skip(1) // header
        .filter_map(|line| {
            let mut parts = line.split(',');
            let timestamp_ms = parts.next()?.parse().ok()?;
            let value = parts.next()?.parse().ok()?;
            Some(DataPoint {
                timestamp_ms,
                value,
            })
        })
        .collect();
    Ok(data)
}

fn read_dual_csv(
    path: &Path,
) -> Result<(Vec<DataPoint>, Vec<DataPoint>), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut primary = Vec::new();
    let mut secondary = Vec::new();

    for line in content.lines().skip(1) {
        let mut parts = line.split(',');
        if let (Some(ts), Some(v1), Some(v2)) = (parts.next(), parts.next(), parts.next())
            && let (Ok(timestamp_ms), Ok(val1), Ok(val2)) =
                (ts.parse(), v1.parse::<f64>(), v2.parse::<f64>())
        {
            primary.push(DataPoint {
                timestamp_ms,
                value: val1,
            });
            secondary.push(DataPoint {
                timestamp_ms,
                value: val2,
            });
        }
    }

    Ok((primary, secondary))
}
