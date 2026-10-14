use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use log::debug;
use rustc_hash::FxHashSet;

use crate::{
    Config, RecordBytes, Result, SequenceNumberCounter, Table, Tree,
    compaction::{
        Choice, Input, flavour::StandardCompaction, leveled::Strategy, state::CompactionState,
        stream::CompactionStream,
    },
    config::FilterPolicyEntry,
    file::TABLES_FOLDER,
    merge::ForwardMerger,
    run_scanner::RunScanner,
    table::{filter::BloomConstructionPolicy, multi_writer::MultiWriter},
    version::{Level, Set, Version},
};

type MergeStream<K, V> = CompactionStream<ForwardMerger<RunScanner<K, V>, K, V>, K, V>;

/// A merged scan over the selected tables, and the tables it reads.
type MergeInput<K, V> = (MergeStream<K, V>, Vec<Table>);

pub struct Worker {
    pub tree_id: u32,
    pub table_id_generator: SequenceNumberCounter,
    pub config: Arc<Config>,
    pub versions: Arc<Set>,
    pub compaction_state: Arc<Mutex<CompactionState>>,
}

impl Worker {
    pub fn new(tree: &Tree) -> Self {
        Self {
            tree_id: tree.id,
            table_id_generator: tree.table_id_counter.clone(),
            config: tree.config.clone(),
            versions: tree.versions.clone(),
            compaction_state: tree.compaction_state.clone(),
        }
    }

    pub fn run<K: RecordBytes, V: RecordBytes>(&self) -> Result<()> {
        loop {
            let state = self.state();
            let version = self.versions.load();

            match Strategy::choose(&version, &state) {
                Choice::Merge(input) => self.merge_tables::<K, V>(state, &version, &input)?,
                Choice::Move(input) => self.move_tables(&state, &input)?,
                Choice::DoNothing => return Ok(()),
            }
        }
    }

    fn state(&self) -> MutexGuard<'_, CompactionState> {
        self.compaction_state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn move_tables(&self, state: &CompactionState, input: &Input) -> Result<()> {
        if state
            .hidden_set()
            .is_blocked(input.table_ids.iter().copied())
        {
            return Ok(());
        }

        self.versions.publish(&self.config.path, |current| {
            Ok(current.with_moved(&input.table_ids, input.dest_level.into()))
        })
    }

    fn merge_tables<K: RecordBytes, V: RecordBytes>(
        &self,
        mut state: MutexGuard<'_, CompactionState>,
        version: &Version,
        input: &Input,
    ) -> Result<()> {
        if state
            .hidden_set()
            .is_blocked(input.table_ids.iter().copied())
        {
            return Ok(());
        }

        let Some((stream, tables)) = Self::create_stream::<K, V>(version, &input.table_ids)? else {
            return Ok(());
        };
        let is_last_level = usize::from(input.dest_level) + 1 == version.level_count();
        let mut stream = stream.evict_tombstones(is_last_level);
        let writer = self.prepare_writer::<K, V>(version, input)?;
        let mut compactor = StandardCompaction::new(writer, tables);

        state.hidden_set_mut().hide(input.table_ids.iter().copied());
        drop(state);

        if let Err(error) = stream.try_for_each(|item| compactor.write(item?)) {
            self.show_tables(&input.table_ids);
            return Err(error);
        }

        let mut state = self.state();
        let result = compactor.finish(self, input, input.canonical_level.into());
        state.hidden_set_mut().show(input.table_ids.iter().copied());
        result
    }

    fn prepare_writer<K: RecordBytes, V: RecordBytes>(
        &self,
        version: &Version,
        input: &Input,
    ) -> Result<MultiWriter<K, V>> {
        let level = usize::from(input.canonical_level);
        let mut writer = MultiWriter::new(
            self.config.path.join(TABLES_FOLDER),
            self.table_id_generator.clone(),
            input.target_size,
        )?;

        let is_last_level = usize::from(input.dest_level) + 1 == version.level_count();
        let bloom_policy = if is_last_level && self.config.expect_point_read_hits {
            BloomConstructionPolicy::BitsPerKey(0.0)
        } else {
            match self
                .config
                .filter_policy
                .at_level(usize::from(input.dest_level))
            {
                FilterPolicyEntry::Bloom(policy) => policy,
                FilterPolicyEntry::None => BloomConstructionPolicy::BitsPerKey(0.0),
            }
        };
        // The final policy goes first so partitioned filters size themselves once.
        writer = writer
            .use_bloom_policy(bloom_policy)
            .use_data_block_restart_interval(
                self.config
                    .data_block_restart_interval_policy
                    .at_level(level),
            );

        if self.config.index_block_partitioning_policy.get(level) {
            writer = writer.use_partitioned_index();
        }
        if self.config.filter_block_partitioning_policy.get(level) {
            writer = writer.use_partitioned_filter();
        }

        debug!(
            "Compacting tables {:?} into L{} (canonical L{}), target_size={}",
            input.table_ids, input.dest_level, input.canonical_level, input.target_size,
        );

        Ok(writer)
    }

    fn show_tables(&self, table_ids: &FxHashSet<u32>) {
        self.state()
            .hidden_set_mut()
            .show(table_ids.iter().copied());
    }

    /// Collects the selected tables and a merged scan over them in one pass over the version.
    ///
    /// Returns `None` if a selected table is no longer part of `version`.
    fn create_stream<K: RecordBytes, V: RecordBytes>(
        version: &Version,
        table_ids: &FxHashSet<u32>,
    ) -> Result<Option<MergeInput<K, V>>> {
        let selected = |table: &Table| table_ids.contains(&table.id());
        let mut readers = Vec::new();
        let mut tables = Vec::with_capacity(table_ids.len());

        for run in version.iter_levels().flat_map(Level::iter) {
            let (Some(start), Some(end)) = (
                run.iter().position(selected),
                run.iter().rposition(selected),
            ) else {
                continue;
            };
            let span = run.get(start..=end).unwrap_or_default();
            debug_assert!(
                span.iter().all(selected),
                "compaction selects contiguous tables of a run"
            );
            tables.extend_from_slice(span);
            readers.push(RunScanner::culled(run.clone(), (Some(start), Some(end)))?);
        }

        Ok((tables.len() == table_ids.len())
            .then(|| (CompactionStream::new(ForwardMerger::new(readers)), tables)))
    }
}
