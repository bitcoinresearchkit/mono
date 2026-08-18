use super::{Run, run::Ranged};
use crate::{HashSet, KeyRange, Table, TableId};
use std::sync::Arc;

impl Ranged for Table {
    fn key_range(&self) -> &KeyRange {
        &self.metadata.key_range
    }
}

/// One immutable level containing sorted table runs.
#[derive(Clone)]
pub struct Level {
    pub runs: Arc<[Arc<Run<Table>>]>,
}

impl Level {
    pub fn empty() -> Self {
        Self::from_runs(Vec::new())
    }

    pub fn from_runs(runs: Vec<Arc<Run<Table>>>) -> Self {
        Self { runs: runs.into() }
    }

    pub fn len(&self) -> usize {
        self.runs.len()
    }

    pub fn table_count(&self) -> usize {
        self.iter().map(|run| run.len()).sum()
    }

    pub fn run_count(&self) -> usize {
        self.runs.len()
    }

    pub fn is_disjoint(&self) -> bool {
        self.run_count() == 1
    }

    pub fn is_empty(&self) -> bool {
        self.runs.is_empty()
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Arc<Run<Table>>> {
        self.runs.iter()
    }

    pub fn list_ids(&self) -> HashSet<TableId> {
        self.iter()
            .flat_map(|run| run.iter())
            .map(Table::id)
            .collect()
    }

    pub fn first_run(&self) -> Option<&Arc<Run<Table>>> {
        self.runs.first()
    }

    pub fn aggregate_key_range(&self) -> KeyRange {
        if let [run] = self.runs.as_ref() {
            run.aggregate_key_range()
        } else {
            let ranges = self
                .iter()
                .map(|run| run.aggregate_key_range())
                .collect::<Vec<_>>();
            KeyRange::aggregate(ranges.iter())
        }
    }
}
