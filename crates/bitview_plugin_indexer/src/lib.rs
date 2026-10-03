#![doc = include_str!("../README.md")]

use std::{
    collections::BTreeMap,
    fs,
    io::ErrorKind,
    path::Path,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use bitcoin::{block::Header, consensus};
use bitview_plugin::{
    ComputePlugin, ImportContext, Plugin, PluginId, PluginStorage, Publication, UpdateContext,
};
use bitview_traversable::{Traversable, TreeNode};
use brk_error::{Error, Result};
use brk_exit::Exit;
use brk_reader::{Reader, XOR_LEN, XORBytes};
use brk_types::{BlkPosition, BlockHash, Height};
use constants::*;
use lengths::IndexerLengths as _;
use processor::{BlockBuffers, BlockProcessor};
use read_pool::join as join_reads;
use readers::Readers;
use state::State;
use stores::Stores;
use tracing::{debug, error, info, warn};
use vecdb::{
    AnyExportableVec, AnyVec, RawDBError, ReadOnlyClone, ReadableVec, Ro, Rw, StorageMode,
    WritableVec, unlikely,
};
use vecs::{
    AddrsVecs, InputsVecs, OpReturnVecs, OutputsVecs, ScriptsVecs, TransactionCounts,
    TransactionFeaturesVecs, TxFeatureFlags, TxMetadataVecs, Vecs,
};

mod constants;
mod has;
mod lengths;
mod processor;
mod read_pool;
mod readers;
mod safe_lengths;
mod state;
mod stores;
mod vecs;

pub use brk_types::Lengths;

pub use has::HasIndexer;

pub use safe_lengths::SafeLengths;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("indexer"), VERSION);
const EXPORT_HEIGHT_INTERVAL: usize = 100;
const MAX_PENDING_RECORDS: usize = 35_000_000;
pub const ID: PluginId = STORAGE.id();

pub struct Indexer<M: StorageMode = Rw> {
    reader: Reader,
    vecs: Vecs<M>,
    stores: Stores,
    buffers: M::WriteOnly<BlockBuffers>,
    state: Arc<State>,
}

enum ImportValidation {
    Valid(Lengths),
    Reset(&'static str),
}

enum XorMarker {
    Missing,
    Invalid(usize),
    Valid(XORBytes),
}

fn is_export_height(height: Height) -> bool {
    height != 0 && height % EXPORT_HEIGHT_INTERVAL == 0
}

fn is_export_due(height: Height, pending_records: usize) -> bool {
    is_export_height(height) && pending_records >= MAX_PENDING_RECORDS
}

fn read_xor_marker(path: &Path) -> Result<XorMarker> {
    let bytes = match fs::read(path.join("xor.dat")) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(XorMarker::Missing),
        Err(err) => return Err(err.into()),
    };
    Ok(match <[u8; XOR_LEN]>::try_from(bytes) {
        Ok(bytes) => XorMarker::Valid(XORBytes::from(bytes)),
        Err(bytes) => XorMarker::Invalid(bytes.len()),
    })
}

fn validate_reader_source(reader: &Reader) -> Result<()> {
    let current = match read_xor_marker(reader.blocks_dir())? {
        XorMarker::Missing => XORBytes::from([0; XOR_LEN]),
        XorMarker::Invalid(received) => {
            return Err(Error::WrongLength {
                expected: XOR_LEN,
                received,
            });
        }
        XorMarker::Valid(xor) => xor,
    };
    if current != reader.xor_bytes() {
        return Err(Error::Internal(
            "Block source changed after the Reader was created",
        ));
    }
    Ok(())
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
    /// Publication barrier shared by the complete pipeline and its readers.
    pub fn publication(&self) -> &Publication {
        &self.state.publication
    }
    /// Tip block hash at the pipeline-safe ceiling.
    ///
    /// Reads the on-disk blockhash vec at `safe_lengths.height - 1` so
    /// the answer always agrees with `safe_lengths`. The indexer's loop
    /// pushes new hashes per block before `safe_lengths` advances (that
    /// only happens after the compute pass); reading from a live cache
    /// here would mint a tip ahead of every safe-bound endpoint and
    /// cause cache etags to invalidate before the data they cover is
    /// actually queryable.
    pub fn tip_blockhash(&self) -> BlockHash {
        // Retain protection through the row read. Recursive acquisition also
        // permits callers already pinning the prefix while a writer queues.
        let guard = self.state.pin_recursive();
        match guard.lengths().last_height() {
            Some(h) => self
                .vecs
                .blocks
                .blockhash
                .collect_one(h)
                .unwrap_or_default(),
            None => BlockHash::default(),
        }
    }

    /// Copy the pipeline-safe bounds. This does not pin the backing data;
    /// retain [`Self::pin_safe_lengths_for`] protection across dependent reads.
    pub fn safe_lengths(&self) -> Lengths {
        self.state.lengths()
    }

    pub fn try_pin_safe_lengths(&self) -> Option<SafeLengths> {
        self.state.try_pin()
    }

    pub fn pin_safe_lengths_for(&self, timeout: Duration) -> Option<SafeLengths> {
        self.state.pin_for(timeout)
    }

    /// Latest safely published indexed height.
    pub fn indexed_height(&self) -> Height {
        self.safe_lengths().last_height().unwrap_or_default()
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

    fn iter_any_visible(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        self.vecs.iter_any_visible()
    }

    fn collect_series_descriptions<'a>(
        &'a self,
        description_fragments: &mut Vec<&'static str>,
        descriptions: &mut BTreeMap<&'a str, Vec<&'static str>>,
    ) {
        self.vecs
            .collect_series_descriptions(description_fragments, descriptions);
    }
}

impl Indexer {
    /// Imports and validates an indexer for writing against `reader`.
    ///
    /// Any reset happens before this function returns, after all handles from
    /// the failed import attempt have been dropped.
    pub fn import(context: ImportContext<'_>, reader: &Reader) -> Result<Self> {
        validate_reader_source(reader)?;
        Self::import_inner(context, reader, true)
    }

    fn import_inner(context: ImportContext<'_>, reader: &Reader, can_retry: bool) -> Result<Self> {
        info!("Loading indexer data...");

        let plugin_path = STORAGE.path(context);

        let try_import = || -> Result<Self> {
            let i = Instant::now();
            let vecs = Vecs::forced_import(&plugin_path, STORAGE.schema_version())?;
            info!("Loaded indexer vectors in {:.2?}", i.elapsed());

            let i = Instant::now();
            let stores = Stores::forced_import(&plugin_path, STORAGE.schema_version())?;
            info!("Loaded indexer state in {:.2?}", i.elapsed());

            Ok(Self {
                reader: reader.clone(),
                vecs,
                stores,
                buffers: BlockBuffers::default(),
                state: Arc::new(State::new()),
            })
        };

        let mut indexer = match try_import() {
            Ok(indexer) => indexer,
            Err(err) if err.is_lock_error() => {
                // Lock errors are transient - another process has the database open.
                // Don't delete data, just return the error.
                return Err(err);
            }
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

        match indexer.validate_import(&plugin_path)? {
            ImportValidation::Valid(lengths) => {
                indexer.rollback_to(&lengths)?;
                indexer.state.finish_update(lengths);
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

        match read_xor_marker(plugin_path)? {
            XorMarker::Missing if is_empty => write_xor_marker(plugin_path, reader.xor_bytes())?,
            XorMarker::Valid(marker) if marker == reader.xor_bytes() => {}
            XorMarker::Missing | XorMarker::Invalid(_) | XorMarker::Valid(_) => {
                return Ok(ImportValidation::Reset(
                    "Indexer block source marker is missing, invalid, or changed",
                ));
            }
        }

        let Some(hash) = self.vecs.blocks.blockhash.collect_last() else {
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

        reader.client().wait_for_synced_node()?;
        let (height, _) = reader.client().get_closest_valid_height(hash)?;
        match Lengths::resume_at(height.incremented(), &self.vecs, &self.stores)? {
            Some(lengths) => Ok(ImportValidation::Valid(lengths)),
            None => Ok(ImportValidation::Reset(
                "Indexer state cannot resume from the active chain",
            )),
        }
    }

    fn rollback_to(&mut self, starting_lengths: &Lengths) -> Result<()> {
        let local_height = self.vecs.next_height();
        if local_height == starting_lengths.height {
            return Ok(());
        }
        if local_height < starting_lengths.height {
            return Err(Error::Internal("Cannot roll back beyond local state"));
        }

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

    fn index_inner(&mut self, exit: &Exit, check_collisions: bool) -> Result<()> {
        let reader = self.reader.clone();
        validate_reader_source(&reader)?;
        let client = reader.client();
        self.vecs.sync_bg_tasks()?;

        debug!("Starting indexing...");

        let last_blockhash = self.vecs.blocks.blockhash.collect_last();
        // Rollback sim: do not remove
        // let last_blockhash = self
        //     .vecs
        //     .blocks
        //     .blockhash
        //     .collect_one_at(self.vecs.blocks.blockhash.len() - 2);
        debug!("Last block hash found.");

        let (starting_lengths, prev_hash) = if let Some(hash) = last_blockhash {
            let (height, hash) = client.get_closest_valid_height(hash)?;
            match Lengths::resume_at(height.incremented(), &self.vecs, &self.stores)? {
                Some(starting_lengths) => {
                    if starting_lengths.height > client.get_last_height()? {
                        info!("Up to date, nothing to index.");
                        return Ok(());
                    }
                    (starting_lengths, Some(hash))
                }
                None => {
                    return Err(Error::Internal(
                        "Indexer became inconsistent after import; drop and re-import it",
                    ));
                }
            }
        } else {
            (Lengths::default(), None)
        };
        debug!("Starting lengths set.");

        let lock = exit.lock();
        self.state.lower_before(&starting_lengths);
        self.rollback_to(&starting_lengths)?;
        debug!("Rollback done.");
        drop(lock);

        self.buffers.continue_from(prev_hash);

        let mut lengths = starting_lengths;
        let mut pending_export_height = None;
        let mut pending_blocks = 0;
        let mut pending_records = 0;

        let export =
            move |stores: &mut Stores, vecs: &mut Vecs, completed_height: Height| -> Result<()> {
                info!("Saving indexer data...");
                let i = Instant::now();
                let _lock = exit.lock();
                let checkpoint = stores.begin_commit(completed_height)?;
                thread::scope(|s| -> Result<()> {
                    let stores_res = s.spawn(|| {
                        let i = Instant::now();
                        let persisted = stores.persist(checkpoint)?;
                        debug!("Stores persisted in {:?}", i.elapsed());
                        Ok::<_, Error>(persisted)
                    });
                    let vecs_res = s.spawn(|| -> Result<()> {
                        let i = Instant::now();
                        vecs.flush(completed_height)?;
                        debug!("Vecs exported in {:?}", i.elapsed());
                        Ok(())
                    });
                    let persisted = stores_res.join().unwrap()?;
                    vecs_res.join().unwrap()?;
                    // The shared checkpoint is visible only after both databases are durable.
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

        for block in reader.after(prev_hash)?.iter() {
            let block = match block {
                Ok(block) => block,
                Err(e) => {
                    // The reader hit an unrecoverable mid-stream issue
                    // (chain break, parse failure, missing blocks).
                    // Stop cleanly so what we've already indexed gets
                    // flushed in the post-loop export — the next
                    // `index` call will resume from the new tip.
                    error!("Reader stream stopped early: {e}");
                    break;
                }
            };
            let height = block.height();

            if unlikely(height.is_multiple_of(100)) {
                info!("Indexing block {height}...");
            } else {
                debug!("Indexing block {height}...");
            }

            lengths.height = height;

            vecs.blocks.position.push(block.metadata().position());
            block.tx_metadata().iter().for_each(|m| {
                vecs.transactions.position.push(m.position());
            });

            let mut processor = BlockProcessor {
                block: &block,
                height,
                check_collisions,
                lengths: &mut lengths,
                vecs,
                stores,
                readers: &readers,
            };

            processor.process_block_metadata()?;

            let txs = processor.compute_txids()?;
            processor.push_block_size_and_weight(&txs);

            let (txins_result, txouts_result) = join_reads(
                &txs,
                || buffers.inputs.resolve(&processor, &txs),
                || processor.process_outputs(&mut buffers.addresses),
            );
            let txins = txins_result?;
            let txouts = txouts_result?;

            let tx_count = block.txdata.len();
            let input_count = txins.len();
            let output_count = txouts.len();

            processor.analyze_and_finalize_transactions(txs, txouts, txins, &mut buffers.addresses);

            processor
                .lengths
                .add_block(tx_count, input_count, output_count);
            buffers.finish_block(*block.hash());
            pending_export_height = Some(height);
            pending_blocks += 1;
            pending_records += tx_count + input_count + output_count;

            if is_export_due(height, pending_records) {
                debug!("Export batch: {pending_blocks} blocks, {pending_records} records");
                drop(readers);
                export(stores, vecs, height)?;
                readers = Readers::new(vecs);
                // Clear only after a successful export so the final export
                // still covers every pending block.
                pending_export_height = None;
                pending_blocks = 0;
                pending_records = 0;
            }
        }

        drop(readers);

        let Some(completed_height) = pending_export_height else {
            return Ok(());
        };

        let lock = exit.lock();
        let deferred_commit = self.stores.take_deferred_commit(completed_height)?;
        self.vecs.stamped_write(completed_height)?;

        self.vecs.run_bg(move |db| {
            let _lock = lock;

            db.bg_sleep(Duration::from_secs(3));

            info!("Saving indexer data...");
            let total_i = Instant::now();

            let commit_i = Instant::now();
            let persisted = deferred_commit.persist().map_err(RawDBError::other)?;
            debug!("Stores persisted in {:?}", commit_i.elapsed());

            db.compact()?;
            // Keep the checkpoint invalid until the vector write is durable too.
            persisted.publish().map_err(RawDBError::other)?;

            info!("Saved indexer data in {:.2?}", total_i.elapsed());
            Ok(())
        });

        Ok(())
    }

    /// Commits indexed disk state as the pipeline-wide safe-lengths snapshot.
    pub fn commit(&mut self) -> Result<()> {
        self.vecs.sync_bg_tasks()?;
        let lengths = match Lengths::from_local(&self.vecs, &self.stores)? {
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
        };
        self.state.finish_update(lengths);
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

    fn compute(&mut self, (): Self::Dependencies<'_>, context: UpdateContext<'_>) -> Result<()> {
        self.index_inner(context.exit(), cfg!(debug_assertions))
    }
}
