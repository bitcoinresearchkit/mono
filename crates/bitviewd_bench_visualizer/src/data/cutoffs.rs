use std::collections::HashMap;

use super::{DualRun, Run};

pub struct Cutoffs {
    by_id: HashMap<String, u64>,
    default: u64,
}

impl Cutoffs {
    pub fn from_progress(progress_runs: &[Run]) -> Self {
        const TIME_BUFFER_MS: u64 = 10_000;

        if progress_runs.is_empty() {
            return Self {
                by_id: HashMap::new(),
                default: u64::MAX,
            };
        }

        let common_progress = progress_runs
            .iter()
            .map(Run::max_value)
            .fold(f64::MAX, f64::min);

        let by_id: HashMap<_, _> = progress_runs
            .iter()
            .map(|run| {
                let cutoff = run
                    .data
                    .iter()
                    .find(|point| point.value >= common_progress)
                    .map(|point| point.timestamp_ms)
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
        runs.iter()
            .map(|run| run.trimmed(self.get(&run.id)))
            .collect()
    }

    pub fn trim_dual_runs(&self, runs: &[DualRun]) -> Vec<DualRun> {
        runs.iter()
            .map(|run| run.trimmed(self.get(&run.id)))
            .collect()
    }
}
