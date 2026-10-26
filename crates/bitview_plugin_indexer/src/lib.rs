#![doc = include_str!("../README.md")]

use std::{collections::BTreeMap, fs, io::ErrorKind, path::Path, sync::Arc, thread, time::Instant};

use bitcoin::{block::Header, consensus};
use bitview_plugin::{
    ComputePlugin, ImportContext, Plugin, PluginId, PluginStorage, UpdateContext,
};
use bitview_primitives::Lengths;
use bitview_traversable::{AnyStoredVec, Traversable, TreeNode};
use brk_error::{Error, Result};
use brk_exit::Exit;
use brk_reader::{Reader, XORBytes};
use brk_types::{BlkPosition, BlockHash, Height};
use crossbeam_channel::{Receiver, bounded};
use tracing::{debug, error, info, warn};
use vecdb::{
    AnyExportableVec, AnyVec, Database, ReadOnlyClone, ReadableVec, Ro, Rw, StorageMode,
    WritableVec, unlikely,
};

use constants::*;
use lengths::IndexerLengths as _;
use processor::{BlockBuffers, BlockProcessor};
use read_pool::join as join_reads;
use readers::Readers;
use rollback_floor::RollbackFloor;
use state::State;
use stores::Stores;
use vecs::{
    AddrsVecs, InputsVecs, OpReturnVecs, OutputsVecs, ScriptsVecs, TransactionCounts,
    TxFeatureFlags, TxMetadataVecs, Vecs,
};

#[macro_use]
mod macros;
mod confirmed;
mod constants;
mod has;
mod lengths;
mod processor;
mod read_pool;
mod readers;
mod rollback_floor;
mod safe_lengths;
mod state;
mod stores;
#[cfg(test)]
mod tests;
mod vecs;

pub use confirmed::{OutputRange, OutputRanges, OutputScripts};
pub use constants::overwritten_output;
pub use has::HasIndexer;
pub use processor::transaction::analysis::policy::FIRST_V29_POLICY_HEIGHT;

pub use safe_lengths::SafeLengths;
pub use stores::UtxoKey;
pub use vecs::{BlockCount, FeatureVecs, FlagView};

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("indexer"), VERSION);
const EXPORT_HEIGHT_INTERVAL: usize = 100;
const MAX_PENDING_RECORDS: usize = 20_000_000;
pub const ID: PluginId = STORAGE.id();

pub struct Indexer<M: StorageMode = Rw> {
    reader: Reader,
    vecs: Vecs<M>,
    stores: Stores,
    buffers: M::WriteOnly<BlockBuffers>,
    floor: M::WriteOnly<RollbackFloor>,
    /// The resume point [`Indexer::reorg`] resolved, which the update's index run consumes.
    resume: M::WriteOnly<Option<ResumePoint>>,
    /// Where dependents recompute from in this update ([`Indexer::compute_start`]).
    compute_start: M::WriteOnly<Lengths>,
    /// The outcome of the update's stores ingestion and checkpoint, running in the background.
    ingestion: M::WriteOnly<Option<Receiver<Result<()>>>>,
    state: Arc<State>,
}

/// Where the node's active chain leaves this index: the lengths to resume at and the hash of the
/// block before them.
struct ResumePoint {
    lengths: Lengths,
    prev_hash: Option<BlockHash>,
}

enum ImportValidation {
    Valid(Lengths),
    Reset(&'static str),
}

fn has_other_plugin_data(context: ImportContext<'_>) -> Result<bool> {
    let path = PluginStorage::plugins_path(context);
    if !path.try_exists()? {
        return Ok(false);
    }
    for entry in fs::read_dir(path)? {
        if entry?.file_name() != ID.as_str() {
            return Ok(true);
        }
    }
    Ok(false)
}

fn write_xor_marker(path: &Path, source_xor: XORBytes) -> Result<()> {
    fs::create_dir_all(path)?;
    let pending = path.join("xor.pending");
    fs::write(&pending, *source_xor)?;
    fs::rename(&pending, path.join("xor.dat"))?;
    Ok(())
}

fn read_block_hash_at(reader: &Reader, position: BlkPosition) -> Result<BlockHash> {
    let bytes = reader.read_raw_bytes(position, Header::SIZE)?;
    let header: Header = consensus::deserialize(&bytes)?;
    Ok(BlockHash::from(header.block_hash()))
}

fn recreate_plugin_dir(path: &Path, source_xor: XORBytes) -> Result<bool> {
    let removed = match fs::remove_dir_all(path) {
        Ok(()) => true,
        Err(err) if err.kind() == ErrorKind::NotFound => false,
        Err(err) => return Err(err.into()),
    };
    write_xor_marker(path, source_xor)?;
    Ok(removed)
}

impl<M: StorageMode> Indexer<M> {
    /// The published tip's hash, recorded with the published lengths: never ahead of what reads
    /// can see, though the indexer appends blocks before every plugin computes them.
    pub fn tip_blockhash(&self) -> BlockHash {
        self.state.pin().tip()
    }

    /// The hash of the last block `lengths` cover (the default hash pre-genesis).
    fn tip_of(&self, lengths: &Lengths) -> BlockHash {
        lengths
            .last_height()
            .and_then(|height| self.vecs.blocks.blockhash.collect_one(height))
            .unwrap_or_default()
    }

    /// Pins the published state; never waits.
    pub fn pin(&self) -> SafeLengths {
        self.state.pin()
    }

    pub fn reader(&self) -> &Reader {
        &self.reader
    }

    #[inline]
    pub fn vecs(&self) -> &Vecs<M> {
        &self.vecs
    }

    #[inline]
    pub fn stores(&self) -> &Stores {
        &self.stores
    }
}

impl<M: StorageMode> Traversable for Indexer<M>
where
    Vecs<M>: Traversable,
{
    fn to_tree_node(&self) -> TreeNode {
        self.vecs.to_tree_node()
    }

    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        self.vecs.iter_any_exportable()
    }

    fn iter_any_stored_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.vecs.iter_any_stored_mut()
    }

    fn iter_any_visible(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        self.vecs.iter_any_visible()
    }

    fn collect_series_descriptions<'a>(
        &'a self,
        description_fragments: &mut Vec<bitview_traversable::Fragment>,
        descriptions: &mut BTreeMap<&'a str, Vec<bitview_traversable::Fragment>>,
    ) {
        self.vecs
            .collect_series_descriptions(description_fragments, descriptions);
    }
}

impl Indexer {
    /// Imports and validates an indexer for writing against `reader`, without contacting the
    /// node: the first index run reconciles the local tip with the active chain.
    ///
    /// Any reset happens before this function returns, after all handles from
    /// the failed import attempt have been dropped. The bootstrap holds the shutdown lock around
    /// the import, so a reset never leaves a half-deleted directory that validation could accept.
    pub fn import(context: ImportContext<'_>, reader: &Reader) -> Result<Self> {
        reader.check_source()?;
        Self::import_inner(context, reader, true)
    }

    fn import_inner(context: ImportContext<'_>, reader: &Reader, can_retry: bool) -> Result<Self> {
        let plugin_path = STORAGE.path(context);

        let try_import = || -> Result<Self> {
            let i = Instant::now();
            let vecs = Vecs::import(&plugin_path, STORAGE.schema_version())?;
            info!("Loaded indexer vectors in {:.2?}", i.elapsed());

            let i = Instant::now();
            let stores = Stores::import(&plugin_path, STORAGE.schema_version())?;
            info!("Loaded indexer state in {:.2?}", i.elapsed());

            Ok(Self {
                reader: reader.clone(),
                vecs,
                stores,
                buffers: BlockBuffers::default(),
                floor: RollbackFloor::load(&plugin_path)?,
                resume: None,
                compute_start: Lengths::default(),
                ingestion: None,
                state: Arc::default(),
            })
        };

        let mut indexer = match try_import() {
            Ok(indexer) => indexer,
            // Another process has the database open: never reset over a lock.
            Err(err) if err.is_lock_error() => return Err(err),
            Err(err) if can_retry && err.is_data_error() => {
                // The failed attempt has returned, so all of its local database
                // handles have been dropped before the directory is removed.
                let removed = recreate_plugin_dir(&plugin_path, reader.xor_bytes())?;
                if removed {
                    warn!(
                        "Removed invalid indexer data at {} after an import failure: {err}",
                        plugin_path.display()
                    );
                }
                return Self::import_inner(context, reader, false);
            }
            Err(err) => return Err(err),
        };

        indexer.drop_uncommitted_rows()?;
        match indexer.validate_import(&plugin_path)? {
            ImportValidation::Valid(lengths) => {
                // Dependents' rows beside an empty (deleted or reset) indexer can't be vouched for;
                // a fresh install has none, so its first full pass stays resumable.
                if lengths.height.is_zero() && has_other_plugin_data(context)? {
                    indexer.floor.lower_to(Height::ZERO)?;
                }
                // Publish the floor a previous run left: rows past it may come from orphaned blocks.
                let published = indexer.floored(lengths);
                // The stores hold `lengths`: a lower floor leaves their views unavailable.
                let unspent = (published == lengths).then(|| indexer.stores.addr_unspent_views());
                indexer
                    .state
                    .finish_update(published, indexer.tip_of(&published), unspent);
                Ok(indexer)
            }
            ImportValidation::Reset(reason) if can_retry => {
                drop(indexer);
                let removed = recreate_plugin_dir(&plugin_path, reader.xor_bytes())?;
                if removed {
                    warn!(
                        "Removed incompatible indexer data at {}: {reason}",
                        plugin_path.display()
                    );
                }
                Self::import_inner(context, reader, false)
            }
            ImportValidation::Reset(reason) => Err(Error::Internal(reason)),
        }
    }

    /// A quit while the last stores commit waited leaves the vectors ahead of the stores'
    /// checkpoint: roll them back to it, with the rollback floor, so dependents recompute too.
    fn drop_uncommitted_rows(&mut self) -> Result<()> {
        let Some(store_height) = self.stores.next_height()? else {
            return Ok(());
        };
        if store_height >= self.vecs.next_height() || !self.vecs.checkpoints_match() {
            return Ok(());
        }
        let (Some(lengths), Some(completed_height)) = (
            Lengths::at(store_height, &self.vecs),
            store_height.decremented(),
        ) else {
            return Ok(());
        };
        warn!(
            "Indexer vectors at {} ahead of the stores at {store_height}: rolling them back",
            self.vecs.next_height()
        );
        self.floor.lower_to(store_height)?;
        self.vecs.rollback_if_needed(&lengths)?;
        self.vecs.flush(completed_height)
    }

    fn validate_import(&self, plugin_path: &Path) -> Result<ImportValidation> {
        let reader = &self.reader;
        let vec_height = self.vecs.next_height();
        let store_height = self.stores.next_height()?;
        let is_empty = vec_height.is_zero() && store_height == Some(Height::ZERO);
        let local_lengths = if is_empty {
            Lengths::default()
        } else if let Some(lengths) = Lengths::from_local(&self.vecs, &self.stores)? {
            lengths
        } else {
            return Ok(ImportValidation::Reset(
                "Indexer checkpoints are missing, inconsistent, or incomplete",
            ));
        };
        // An interrupted parallel write can leave vectors at different checkpoints, or written
        // vectors beside an empty tip.
        let blockhash = &self.vecs.blocks.blockhash;
        if !self.vecs.checkpoints_match()
            || blockhash.len() != usize::from(vec_height)
            || (blockhash.is_empty() && !self.vecs.all_empty())
        {
            return Ok(ImportValidation::Reset(
                "Indexer vectors stopped at different heights",
            ));
        }

        // The node's mask, recorded beside the data indexed from its blocks.
        match XORBytes::read(plugin_path) {
            Ok(None) if is_empty => write_xor_marker(plugin_path, reader.xor_bytes())?,
            Ok(Some(marker)) if marker == reader.xor_bytes() => {}
            Ok(_) | Err(Error::WrongLength { .. }) => {
                return Ok(ImportValidation::Reset(
                    "Indexer block source marker is missing, invalid, or changed",
                ));
            }
            Err(error) => return Err(error),
        }

        let Some(hash) = blockhash.collect_last() else {
            return Ok(ImportValidation::Valid(local_lengths));
        };

        let tip_height = Height::from(self.vecs.blocks.blockhash.len() - 1);
        let Some(position) = self.vecs.blocks.position.collect_one(tip_height) else {
            return Ok(ImportValidation::Reset(
                "Indexer tip block position is missing",
            ));
        };
        if read_block_hash_at(reader, position)? != hash {
            return Ok(ImportValidation::Reset(
                "Indexer block positions belong to a different block source",
            ));
        }

        Ok(ImportValidation::Valid(local_lengths))
    }

    /// `lengths`, lowered to the rollback floor when one lies below them.
    fn floored(&self, lengths: Lengths) -> Lengths {
        match self.floor.height() {
            Some(floor) if floor < lengths.height => {
                Lengths::at(floor, &self.vecs).unwrap_or_default()
            }
            _ => lengths,
        }
    }

    /// Where dependents recompute from in this update: the indexer's resume point, lowered to
    /// the rollback floor, fixed from the index run until the next one. Publication only guards
    /// readers: a plugin lowering it (to rewrite its own rows) never moves the others' start.
    pub fn compute_start(&self) -> Lengths {
        self.compute_start
    }

    /// Waits for the last stores ingestion and its checkpoint, returning its outcome.
    fn finish_ingestion(&mut self) -> Result<()> {
        self.ingestion.take().map_or(Ok(()), |ingested| {
            ingested
                .recv()
                .unwrap_or(Err(Error::Internal("Indexer stores ingestion panicked")))
        })
    }

    fn resume_point(&mut self) -> Result<ResumePoint> {
        self.finish_ingestion()?;
        self.vecs.db().sync_bg_tasks()?;
        let client = self.reader.client();

        let last_blockhash = self.vecs.blocks.blockhash.collect_last();
        // Rollback sim: do not remove
        // let last_blockhash = self
        //     .vecs
        //     .blocks
        //     .blockhash
        //     .collect_one_at(self.vecs.blocks.blockhash.len() - 2);

        // A node behind our tip (restarting, reindexing) would look like a reorg to its height.
        client.wait_for_synced_node()?;
        let Some(hash) = last_blockhash else {
            return Ok(ResumePoint {
                lengths: Lengths::default(),
                prev_hash: None,
            });
        };
        let (height, hash) = client.get_closest_valid_height(hash)?;
        match Lengths::resume_at(height.incremented(), &self.vecs, &self.stores)? {
            Some(lengths) => Ok(ResumePoint {
                lengths,
                prev_hash: Some(hash),
            }),
            None => Err(Error::Internal(
                "Indexer became inconsistent after import; drop and re-import it",
            )),
        }
    }

    /// The lengths a reorg rolls back to: some only when the node's active chain orphaned blocks
    /// this index holds. The update's index run resumes from the same point, without resolving
    /// it again.
    pub fn reorg(&mut self) -> Result<Option<Lengths>> {
        let resume = self.resume_point()?;
        let starting_lengths = resume.lengths;
        self.resume = Some(resume);
        Ok((starting_lengths.height < self.vecs.next_height()).then_some(starting_lengths))
    }

    /// Rolls back to `starting_lengths`, first publishing the prefix below them once no reader
    /// holds a later publication. Callers hold the exit lock.
    pub fn roll_back_to(&mut self, starting_lengths: &Lengths) -> Result<()> {
        let local_height = self.vecs.next_height();
        if local_height == starting_lengths.height {
            return Ok(());
        }
        if local_height < starting_lengths.height {
            return Err(Error::Internal("Cannot roll back beyond local state"));
        }
        self.floor.lower_to(starting_lengths.height)?;
        self.state
            .lower_before(starting_lengths, self.tip_of(starting_lengths));

        let completed_height = starting_lengths
            .height
            .decremented()
            .ok_or(Error::Internal("Cannot roll back before genesis"))?;
        self.stores
            .rollback_if_needed(&self.vecs, starting_lengths)?;
        self.vecs.rollback_if_needed(starting_lengths)?;

        let checkpoint = self.stores.begin_commit(completed_height)?;
        let persisted = self.stores.persist(checkpoint)?;
        self.vecs.flush(completed_height)?;
        persisted.publish()
    }

    fn index_inner(&mut self, exit: &Exit) -> Result<()> {
        let reader = self.reader.clone();
        reader.check_source()?;

        let ResumePoint {
            lengths: starting_lengths,
            prev_hash,
        } = match self.resume.take() {
            Some(resume) => resume,
            None => self.resume_point()?,
        };

        let lock = exit.lock();
        self.roll_back_to(&starting_lengths)?;
        drop(lock);
        self.compute_start = self.floored(starting_lengths);

        self.buffers.continue_from(prev_hash);

        let mut lengths = starting_lengths;
        let mut pending_export_height = None;
        let mut pending_blocks = 0;
        let mut pending_records = 0;

        let export =
            move |stores: &mut Stores, vecs: &mut Vecs, completed_height: Height| -> Result<()> {
                let i = Instant::now();
                let _lock = exit.lock();
                let checkpoint = stores.begin_commit(completed_height)?;
                thread::scope(|s| -> Result<()> {
                    let stores_res = s.spawn(|| stores.persist(checkpoint));
                    let vecs_res = s.spawn(|| -> Result<()> {
                        let i = Instant::now();
                        vecs.flush(completed_height)?;
                        debug!("Vecs exported in {:?}", i.elapsed());
                        Ok(())
                    });
                    let persisted = stores_res.join().unwrap()?;
                    vecs_res.join().unwrap()?;
                    // The shared checkpoint becomes visible only once both databases are written.
                    persisted.publish()?;
                    Ok(())
                })?;
                info!("Saved indexer data in {:.2?}", i.elapsed());
                Ok(())
            };

        let mut readers = Readers::new(&self.vecs);

        let vecs = &mut self.vecs;
        let stores = &mut self.stores;
        let buffers = &mut self.buffers;

        for block in reader.after(prev_hash)? {
            let block = match block {
                Ok(block) => block,
                Err(e) => {
                    // A chain break, parse failure or missing block: the blocks indexed so far
                    // are saved below and the next update resumes after them.
                    error!("Reader stream stopped early: {e}");
                    break;
                }
            };
            let height = block.height();

            if unlikely(height.is_multiple_of(100)) {
                info!("Indexing block {height}...");
            }

            lengths.height = height;

            vecs.blocks.position.push(block.position());
            block
                .tx_positions()
                .for_each(|position| vecs.transactions.position.push(position));

            let mut processor = BlockProcessor {
                block: &block,
                height,
                lengths: &mut lengths,
                vecs,
                stores,
                readers: &readers,
            };

            processor.process_block_metadata()?;

            let txs = processor.compute_txids()?;
            processor.push_block_size_and_weight(&txs);

            let BlockBuffers {
                inputs,
                addresses,
                outputs,
                analyses,
                ..
            } = &mut *buffers;
            let (txins_result, txouts_result) = join_reads(
                &txs,
                || inputs.resolve(&processor, &txs),
                || processor.process_outputs(&txs, addresses, outputs),
            );
            let txins = txins_result?;
            txouts_result?;

            let tx_count = block.txdata.len();
            let input_count = txins.len();
            let output_count = outputs.len();

            processor.analyze_and_finalize_transactions(txs, outputs, txins, addresses, analyses);

            processor
                .lengths
                .add_block(tx_count, input_count, output_count);
            buffers.finish_block(*block.hash());
            pending_export_height = Some(height);
            pending_blocks += 1;
            pending_records += tx_count + input_count + output_count;

            if height != 0
                && height % EXPORT_HEIGHT_INTERVAL == 0
                && pending_records >= MAX_PENDING_RECORDS
            {
                debug!("Export batch: {pending_blocks} blocks, {pending_records} records");
                drop(readers);
                export(stores, vecs, height)?;
                readers = Readers::new(vecs);
                pending_export_height = None;
                pending_blocks = 0;
                pending_records = 0;
            }
        }

        drop(readers);

        let Some(completed_height) = pending_export_height else {
            return Ok(());
        };

        let deferred_commit = {
            let _lock = exit.lock();
            let deferred_commit = self.stores.take_deferred_commit(completed_height);
            self.vecs.stamped_write(completed_height)?;
            deferred_commit
        };

        // Ingestion starts beside the dependents' compute. A quit before it takes the lock keeps
        // the stores at their last checkpoint (import drops the vectors' newer rows); once it
        // holds the lock, the quit waits for the new checkpoint. The commit waits for it; the
        // vectors' compaction runs after the update, off the publication path.
        let exit = exit.clone();
        let (done, ingested) = bounded(1);
        self.ingestion = Some(ingested);
        self.vecs.db().run_bg(move |_| {
            let _lock = exit.lock();
            let i = Instant::now();
            let result = deferred_commit
                .persist()
                .and_then(|persisted| persisted.publish());
            if result.is_ok() {
                info!("Saved indexer data in {:.2?}", i.elapsed());
            }
            let _ = done.send(result);
            Ok(())
        });

        Ok(())
    }

    /// Runs `rewrite`, which changes rows past `height` in place (a plugin's rollback landing
    /// below where it resumes, a truncation, or a reset), after publishing the prefix below
    /// `height` and waiting until no reader holds a later publication: every destructive
    /// rewrite goes through here. A no-op lowering never waits. The rollback floor stays: a
    /// plugin rewriting its own rows invalidates no chain.
    pub fn rewrite_from<T>(
        &self,
        height: Height,
        rewrite: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        let lengths = Lengths::at(height, &self.vecs).unwrap_or_default();
        self.state.lower_before(&lengths, self.tip_of(&lengths));
        rewrite()
    }

    /// Finishes the indexer's own writes and returns the lengths a commit publishes.
    pub fn prepare_commit(&mut self) -> Result<Lengths> {
        self.finish_ingestion()?;
        Ok(match Lengths::from_local(&self.vecs, &self.stores)? {
            Some(lengths) => lengths,
            None if self.vecs.next_height().is_zero()
                && self.stores.next_height()? == Some(Height::ZERO) =>
            {
                Lengths::default()
            }
            None => {
                return Err(Error::Internal(
                    "Indexer checkpoints became inconsistent during the update",
                ));
            }
        })
    }

    /// Publishes `lengths` from [`Self::prepare_commit`] as the pipeline-wide safe-lengths
    /// snapshot. A `complete` update (every plugin computed) also clears the rollback floor.
    pub fn publish(&mut self, lengths: Lengths, complete: bool) -> Result<()> {
        // `prepare_commit` waited for the stores' ingestion: they hold exactly `lengths`.
        let unspent = self.stores.addr_unspent_views();
        self.state
            .finish_update(lengths, self.tip_of(&lengths), Some(unspent));
        if complete {
            self.floor.clear()?;
        }
        Ok(())
    }
}

impl ReadOnlyClone for Indexer {
    type ReadOnly = Indexer<Ro>;

    fn read_only_clone(&self) -> Indexer<Ro> {
        Indexer {
            reader: self.reader.clone(),
            vecs: self.vecs.read_only_clone(),
            stores: self.stores.clone(),
            buffers: (),
            floor: (),
            resume: (),
            compute_start: (),
            ingestion: (),
            state: self.state.clone(),
        }
    }
}

impl<M: StorageMode> Plugin for Indexer<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}

impl ComputePlugin for Indexer {
    type Dependencies<'a> = ();

    fn database(&self) -> Option<&Database> {
        Some(self.vecs.db())
    }

    fn compute_state(
        &mut self,
        (): Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        self.index_inner(context.exit())
    }
}
