#![doc = include_str!("../README.md")]

use brk_error::Result;

use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::ErrorKind,
    path::Path,
    thread,
    time::{Duration, Instant},
};

use bitview_plugin::{
    ComputePlugin, ImportContext, Plugin, PluginGate, PluginId, PluginStorage, UpdateContext,
};
use bitview_traversable::{Traversable, TreeNode};
use brk_error::Error;
use brk_reader::{Reader, XOR_LEN, XORBytes};
use brk_types::{BlkPosition, BlockHash, Height};
use tracing::{debug, error, info};
use vecdb::{
    AnyExportableVec, AnyVec, Exit, RawDBError, ReadOnlyClone, ReadableVec, Ro, Rw, StorageMode,
    WritableVec, unlikely,
};
mod constants;
mod has;
mod lengths;
mod processor;
mod readers;
mod state;
mod stores;
mod vecs;

use constants::*;
use lengths::IndexerLengths as _;
use processor::{BlockBuffers, BlockProcessor};
use readers::Readers;
use stores::IndexerStores as _;
use vecs::{IndexerVecs as _, TransactionCounts, TxFeatureFlags};

pub use brk_types::Lengths;
pub use has::HasIndexer;
use stores::Stores;
use vecs::{
    AddrsVecs, InputsVecs, OpReturnVecs, OutputsVecs, ScriptsVecs, TransactionFeaturesVecs,
    TxMetadataVecs, Vecs,
};

use state::State;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("indexer"), VERSION);
const EXPORT_INTERVAL: Duration = Duration::from_secs(60);
const MAX_PENDING_BLOCKS: usize = 10_000;
pub const ID: PluginId = STORAGE.id();

pub struct Indexer<M: StorageMode = Rw> {
    inner: IndexerInner<M>,
}

struct IndexerInner<M: StorageMode> {
    reader: Reader,
    vecs: Vecs<M>,
    stores: Stores,
    buffers: BlockBuffers,
    state: State,
    plugin_gate: PluginGate,
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
    height != 0 && height % SNAPSHOT_BLOCK_RANGE == 0
}

fn is_export_due(height: Height, elapsed: Duration, pending_blocks: usize) -> bool {
    pending_blocks >= MAX_PENDING_BLOCKS || (is_export_height(height) && elapsed >= EXPORT_INTERVAL)
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
    File::open(&pending)?.sync_all()?;
    fs::rename(&pending, path.join("xor.dat"))?;
    File::open(path)?.sync_all()?;
    Ok(())
}

fn read_block_hash_at(reader: &Reader, position: BlkPosition) -> Result<BlockHash> {
    let bytes = reader.read_raw_bytes(position, bitcoin::block::Header::SIZE)?;
    let header: bitcoin::block::Header = bitcoin::consensus::deserialize(&bytes)?;
    Ok(BlockHash::from(header.block_hash()))
}

fn recreate_plugin_dir(path: &Path, source_xor: XORBytes) -> Result<()> {
    match fs::remove_dir_all(path) {
        Ok(()) => {}
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => return Err(err.into()),
    }
    write_xor_marker(path, source_xor)
}

impl<M: StorageMode> Indexer<M> {
    /// Tip block hash at the pipeline-safe ceiling.
    ///
    /// Reads the on-disk blockhash vec at `safe_lengths.height - 1` so
    /// the answer always agrees with `safe_lengths`. The indexer's loop
    /// pushes new hashes per block before `safe_lengths` advances (that
    /// only happens after the compute pass via
    /// [`Indexer::finish_update`]); reading from a live cache
    /// here would mint a tip ahead of every safe-bound endpoint and
    /// cause cache etags to invalidate before the data they cover is
    /// actually queryable.
    pub fn tip_blockhash(&self) -> BlockHash {
        match self.safe_lengths().last_height() {
            Some(h) => self
                .inner
                .vecs
                .blocks
                .blockhash
                .collect_one(h)
                .unwrap_or_default(),
            None => BlockHash::default(),
        }
    }

    /// Pipeline-safe `Lengths` snapshot shared with `Query`. Writers
    /// advance and lower this internally; readers clamp non-series
    /// answers against this loaded snapshot.
    pub fn safe_lengths(&self) -> Lengths {
        self.inner.state.lengths()
    }

    /// Latest safely published indexed height.
    pub fn indexed_height(&self) -> Height {
        self.safe_lengths().last_height().unwrap_or_default()
    }

    pub fn reader(&self) -> &Reader {
        &self.inner.reader
    }

    #[inline]
    pub fn vecs(&self) -> &Vecs<M> {
        &self.inner.vecs
    }

    #[inline]
    pub fn stores(&self) -> &Stores {
        &self.inner.stores
    }
}

impl<M: StorageMode> Traversable for Indexer<M>
where
    Vecs<M>: Traversable,
{
    fn to_tree_node(&self) -> TreeNode {
        self.inner.vecs.to_tree_node()
    }

    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        self.inner.vecs.iter_any_exportable()
    }

    fn iter_any_visible(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        self.inner.vecs.iter_any_visible()
    }

    fn collect_series_descriptions<'a>(
        &'a self,
        description_fragments: &mut Vec<&'static str>,
        descriptions: &mut BTreeMap<&'a str, Vec<&'static str>>,
    ) {
        self.inner
            .vecs
            .collect_series_descriptions(description_fragments, descriptions);
    }
}

impl Indexer {
    /// Imports and validates an indexer for writing against `reader`.
    ///
    /// Any reset happens before this function returns, after all handles from
    /// the failed import attempt have been dropped.
    pub fn import(context: ImportContext<'_>, reader: &Reader) -> Result<Self> {
        Ok(Self {
            inner: IndexerInner::import(context, reader)?,
        })
    }

    pub fn index(&mut self, exit: &Exit) -> Result<()> {
        self.begin_update();
        self.inner.index(exit, false)
    }

    pub fn checked_index(&mut self, exit: &Exit) -> Result<()> {
        self.begin_update();
        self.inner.index(exit, true)
    }

    pub fn begin_update(&self) {
        self.inner.plugin_gate.begin_update();
    }

    /// Publish disk state as the new safe-lengths snapshot. Drains pending
    /// bg ingest first so stores are queryable at the new bound.
    pub fn finish_update(&mut self) -> Result<()> {
        self.commit()?;
        self.inner.plugin_gate.finish_update();
        Ok(())
    }

    /// Commits indexed disk state as the pipeline-wide safe-lengths snapshot.
    pub fn commit(&mut self) -> Result<()> {
        self.inner.commit()
    }
}

impl IndexerInner<Rw> {
    fn import(context: ImportContext<'_>, reader: &Reader) -> Result<Self> {
        validate_reader_source(reader)?;
        Self::import_inner(context, reader, true)
    }

    fn import_inner(context: ImportContext<'_>, reader: &Reader, can_retry: bool) -> Result<Self> {
        info!("Importing indexer...");

        let plugin_path = STORAGE.path(context);

        let try_import = || -> Result<Self> {
            let i = Instant::now();
            let vecs = Vecs::forced_import(&plugin_path, STORAGE.schema_version())?;
            info!("Imported vecs in {:?}", i.elapsed());

            let i = Instant::now();
            let stores = Stores::forced_import(&plugin_path, STORAGE.schema_version())?;
            info!("Imported stores in {:?}", i.elapsed());

            Ok(Self {
                reader: reader.clone(),
                vecs,
                stores,
                buffers: BlockBuffers::default(),
                state: State::new(),
                plugin_gate: PluginGate::new(),
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
                info!("{err:?}, deleting {plugin_path:?} and retrying");
                recreate_plugin_dir(&plugin_path, reader.xor_bytes())?;
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
                info!("{reason}, deleting {plugin_path:?} and retrying");
                drop(indexer);
                recreate_plugin_dir(&plugin_path, reader.xor_bytes())?;
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

    fn index(&mut self, exit: &Exit, check_collisions: bool) -> Result<()> {
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
        let mut last_export = Instant::now();

        let export =
            move |stores: &mut Stores, vecs: &mut Vecs, completed_height: Height| -> Result<()> {
                info!("Exporting...");
                let i = Instant::now();
                let _lock = exit.lock();
                let checkpoint = stores.begin_commit(completed_height)?;
                thread::scope(|s| -> Result<()> {
                    let stores_res = s.spawn(|| {
                        let i = Instant::now();
                        let persisted = stores.persist(checkpoint)?;
                        debug!("Stores persisted in {:?}", i.elapsed());
                        Ok::<_, brk_error::Error>(persisted)
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
                info!("Exported in {:?}", i.elapsed());
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

            let (txins_result, txouts_result) = rayon::join(
                || processor.process_inputs(&txs, &mut buffers.inputs),
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

            if is_export_due(height, last_export.elapsed(), pending_blocks) {
                drop(readers);
                export(stores, vecs, height)?;
                readers = Readers::new(vecs);
                // Clear only after a successful export so the final export
                // still covers every pending block.
                pending_export_height = None;
                pending_blocks = 0;
                last_export = Instant::now();
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

            info!("Exporting...");
            let total_i = Instant::now();

            let commit_i = Instant::now();
            let persisted = deferred_commit.persist().map_err(RawDBError::other)?;
            debug!("Stores persisted in {:?}", commit_i.elapsed());

            db.compact()?;
            // Keep the checkpoint invalid until the vector write is durable too.
            persisted.publish().map_err(RawDBError::other)?;

            info!("Exported in {:?}", total_i.elapsed());
            Ok(())
        });

        Ok(())
    }

    fn commit(&mut self) -> Result<()> {
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
            inner: IndexerInner {
                reader: self.inner.reader.clone(),
                vecs: self.inner.vecs.read_only_clone(),
                stores: self.inner.stores.clone(),
                buffers: BlockBuffers::default(),
                state: self.inner.state.clone(),
                plugin_gate: self.inner.plugin_gate.clone(),
            },
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

    fn gate(&self) -> &PluginGate {
        &self.inner.plugin_gate
    }
}

impl ComputePlugin for Indexer {
    type Dependencies<'a> = ();
    type Output = ();

    fn compute(
        &mut self,
        (): Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<Self::Output> {
        self.inner.index(context.exit(), cfg!(debug_assertions))
    }
}

#[cfg(test)]
mod import_tests {
    use std::path::PathBuf;

    use brk_rpc::{Auth, Client};
    use brk_types::BlockHashPrefix;

    use super::*;

    fn plugin_data_path(outputs_dir: &Path) -> PathBuf {
        STORAGE.path(ImportContext::new(outputs_dir))
    }

    fn empty_reader(path: &Path) -> Reader {
        let client = Client::new("http://127.0.0.1:1", Auth::None).unwrap();
        Reader::new_without_rlimit(path.join("blocks"), &client)
    }

    #[test]
    fn export_is_due_at_the_time_or_block_limit() {
        let snapshot_height = Height::from(SNAPSHOT_BLOCK_RANGE);

        assert!(!is_export_due(
            Height::ZERO,
            EXPORT_INTERVAL,
            MAX_PENDING_BLOCKS - 1
        ));
        assert!(!is_export_due(
            snapshot_height.incremented(),
            EXPORT_INTERVAL,
            MAX_PENDING_BLOCKS - 1
        ));
        assert!(!is_export_due(
            snapshot_height,
            EXPORT_INTERVAL - Duration::from_nanos(1),
            MAX_PENDING_BLOCKS - 1
        ));
        assert!(is_export_due(snapshot_height, EXPORT_INTERVAL, 0));
        assert!(is_export_due(
            snapshot_height.incremented(),
            Duration::ZERO,
            MAX_PENDING_BLOCKS
        ));
    }

    #[test]
    fn recreate_drops_old_contents_and_seeds_source_identity() {
        let dir = tempfile::tempdir().unwrap();
        let plugin = plugin_data_path(dir.path());
        fs::create_dir_all(&plugin).unwrap();
        fs::write(plugin.join("stale"), b"stale").unwrap();
        let source_xor = XORBytes::from([7_u8; 8]);

        recreate_plugin_dir(&plugin, source_xor).unwrap();

        assert!(!plugin.join("stale").exists());
        assert!(matches!(
            read_xor_marker(&plugin).unwrap(),
            XorMarker::Valid(marker) if marker == source_xor
        ));
    }

    #[test]
    fn empty_import_writes_identity_marker() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let reader = empty_reader(dir.path());

        drop(Indexer::import(ImportContext::new(dir.path()), &reader)?);

        assert!(matches!(
            read_xor_marker(&plugin_data_path(dir.path()))?,
            XorMarker::Valid(marker) if marker == XORBytes::from([0; XOR_LEN])
        ));
        Ok(())
    }

    #[test]
    fn malformed_xor_marker_recreates_the_index() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let plugin = plugin_data_path(dir.path());
        let reader = empty_reader(dir.path());
        drop(Indexer::import(ImportContext::new(dir.path()), &reader)?);
        fs::write(plugin.join("xor.dat"), [0_u8; 3])?;
        fs::write(plugin.join("stale"), b"stale")?;

        drop(Indexer::import(ImportContext::new(dir.path()), &reader)?);

        assert!(!plugin.join("stale").exists());
        assert!(matches!(
            read_xor_marker(&plugin)?,
            XorMarker::Valid(marker) if marker == reader.xor_bytes()
        ));
        Ok(())
    }

    #[test]
    fn malformed_source_xor_never_deletes_data() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let plugin = plugin_data_path(dir.path());
        let reader = empty_reader(dir.path());
        drop(Indexer::import(ImportContext::new(dir.path()), &reader)?);
        fs::write(plugin.join("stale"), b"stale")?;
        fs::create_dir_all(dir.path().join("blocks"))?;
        fs::write(dir.path().join("blocks/xor.dat"), [0_u8; 3])?;
        let reader = empty_reader(dir.path());

        assert!(Indexer::import(ImportContext::new(dir.path()), &reader).is_err());
        assert!(plugin.join("stale").exists());
        Ok(())
    }

    #[test]
    fn xor_marker_io_error_never_deletes_data() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let plugin = plugin_data_path(dir.path());
        let marker = plugin.join("xor.dat");
        let reader = empty_reader(dir.path());
        drop(Indexer::import(ImportContext::new(dir.path()), &reader)?);
        fs::remove_file(&marker)?;
        fs::create_dir(&marker)?;
        fs::write(plugin.join("stale"), b"stale")?;

        assert!(Indexer::import(ImportContext::new(dir.path()), &reader).is_err());
        assert!(plugin.join("stale").exists());
        Ok(())
    }

    #[test]
    fn checkpoint_io_error_never_deletes_data() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let plugin = plugin_data_path(dir.path());
        let checkpoint = plugin.join("stores/height");
        let reader = empty_reader(dir.path());
        drop(Indexer::import(ImportContext::new(dir.path()), &reader)?);
        fs::remove_file(&checkpoint)?;
        fs::create_dir(&checkpoint)?;
        fs::write(plugin.join("stale"), b"stale")?;

        assert!(Indexer::import(ImportContext::new(dir.path()), &reader).is_err());
        assert!(plugin.join("stale").exists());
        Ok(())
    }

    #[test]
    fn block_position_is_verified_against_its_header() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let blocks = dir.path().join("blocks");
        fs::create_dir(&blocks)?;
        let genesis = bitcoin::blockdata::constants::genesis_block(bitcoin::Network::Bitcoin);
        fs::write(
            blocks.join("blk00000.dat"),
            bitcoin::consensus::serialize(&genesis.header),
        )?;
        let reader = empty_reader(dir.path());

        assert_eq!(
            read_block_hash_at(&reader, BlkPosition::new(0, 0))?,
            BlockHash::from(genesis.block_hash())
        );
        Ok(())
    }

    #[test]
    fn fjall_lock_never_triggers_deletion() {
        let error = Error::from(fjall::Error::Locked);

        assert!(error.is_lock_error());
        assert!(!error.is_data_error());
    }

    #[test]
    fn invalid_checkpoint_drops_handles_and_recreates_entire_index() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let plugin = plugin_data_path(dir.path());
        let reader = empty_reader(dir.path());

        {
            let mut indexer = Indexer::import(ImportContext::new(dir.path()), &reader)?;
            indexer
                .inner
                .stores
                .insert_block_height(BlockHashPrefix::from(1_u64), Height::ZERO);
            let checkpoint = indexer.inner.stores.begin_commit(Height::ZERO)?;
            let persisted = indexer.inner.stores.persist(checkpoint)?;
            drop(persisted);
        }
        fs::write(plugin.join("stale"), b"stale")?;

        let indexer = Indexer::import(ImportContext::new(dir.path()), &reader)?;

        assert!(!plugin.join("stale").exists());
        assert_eq!(indexer.vecs().next_height(), Height::ZERO);
        assert_eq!(indexer.stores().next_height()?, Some(Height::ZERO));
        Ok(())
    }
}
