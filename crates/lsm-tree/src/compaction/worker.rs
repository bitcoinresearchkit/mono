use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use log::debug;

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
    version::{Level, Run, Set, Version},
};

type MergeStream<K, V> = CompactionStream<ForwardMerger<RunScanner<K, V>, K, V>, K, V>;

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
            .should_decline_compaction(input.table_ids.iter().copied())
        {
            return Ok(());
        }

        let table_ids = input.table_ids.iter().copied().collect::<Vec<_>>();
        self.versions.publish(&self.config.path, |current| {
            Ok(current.with_moved(&table_ids, input.dest_level.into()))
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
            .should_decline_compaction(input.table_ids.iter().copied())
        {
            return Ok(());
        }

        let Some(tables) = input
            .table_ids
            .iter()
            .map(|id| version.get_table(*id).cloned())
            .collect::<Option<Vec<_>>>()
        else {
            return Ok(());
        };

        let table_ids = input.table_ids.iter().copied().collect::<Vec<_>>();
        let Some(stream) = Self::create_stream::<K, V>(version, &table_ids)? else {
            return Ok(());
        };
        let is_last_level = usize::from(input.dest_level) + 1 == version.level_count();
        let mut stream = stream.evict_tombstones(is_last_level);
        let writer = self.prepare_writer::<K, V>(version, input)?;
        let mut compactor = StandardCompaction::new(writer, tables);

        state.hidden_set_mut().hide(table_ids.iter().copied());
        drop(state);

        if let Err(error) = stream.try_for_each(|item| compactor.write(item?)) {
            self.show_tables(&table_ids);
            return Err(error);
        }

        let mut state = self.state();
        let result = compactor.finish(self, input, input.canonical_level.into());
        state.hidden_set_mut().show(table_ids);
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

        if self.config.index_block_partitioning_policy.get(level) {
            writer = writer.use_partitioned_index();
        }
        if self.config.filter_block_partitioning_policy.get(level) {
            writer = writer.use_partitioned_filter();
        }

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

        debug!(
            "Compacting tables {:?} into L{} (canonical L{}), target_size={}",
            input.table_ids, input.dest_level, input.canonical_level, input.target_size,
        );

        Ok(writer
            .use_data_block_restart_interval(
                self.config
                    .data_block_restart_interval_policy
                    .at_level(level),
            )
            .use_data_block_compression(self.config.data_block_compression_policy.at_level(level))
            .use_data_block_size(self.config.data_block_size_policy.at_level(level))
            .use_index_block_compression(self.config.index_block_compression_policy.at_level(level))
            .use_bloom_policy(bloom_policy))
    }

    fn show_tables(&self, table_ids: &[u32]) {
        self.state()
            .hidden_set_mut()
            .show(table_ids.iter().copied());
    }

    fn create_stream<K: RecordBytes, V: RecordBytes>(
        version: &Version,
        table_ids: &[u32],
    ) -> Result<Option<MergeStream<K, V>>> {
        let mut readers = Vec::new();
        let mut found = 0;

        for run in version.iter_levels().flat_map(Level::iter) {
            let Some((start, end)) = Self::run_indexes(run, table_ids) else {
                continue;
            };
            readers.push(RunScanner::culled(run.clone(), (Some(start), Some(end)))?);
            found += end - start + 1;
        }

        Ok((found == table_ids.len()).then(|| CompactionStream::new(ForwardMerger::new(readers))))
    }

    fn run_indexes(run: &Run<Table>, table_ids: &[u32]) -> Option<(usize, usize)> {
        Some((
            run.iter()
                .position(|table| table_ids.contains(&table.id()))?,
            run.iter()
                .rposition(|table| table_ids.contains(&table.id()))?,
        ))
    }
}
