#![doc = include_str!("../README.md")]

use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use brk_error::Result;
use brk_reader::{Reader, XORBytes};
use brk_rpc::Client;
use brk_types::{BlockHash, Height};
use tracing::{debug, error, info};
use vecdb::{
    Exit, RawDBError, ReadOnlyClone, ReadableVec, Ro, Rw, StorageMode, WritableVec, unlikely,
};
mod constants;
mod lengths;
mod processor;
mod readers;
mod safe_lengths;
mod stores;
mod vecs;

use constants::*;
use processor::{BlockBuffers, BlockProcessor};
use readers::Readers;

pub use lengths::Lengths;
pub use safe_lengths::SafeLengths;
pub use stores::Stores;
pub use vecs::*;

fn shared_tip_height(vecs_next_height: Height, stores_next_height: Height) -> Option<Height> {
    vecs_next_height.min(stores_next_height).decremented()
}

pub struct Indexer<M: StorageMode = Rw> {
    path: PathBuf,
    pub vecs: Vecs<M>,
    pub stores: Stores,
    safe_lengths: SafeLengths,
}

impl<M: StorageMode> Indexer<M> {
    /// Tip block hash at the pipeline-safe ceiling.
    ///
    /// Reads the on-disk blockhash vec at `safe_lengths.height - 1` so
    /// the answer always agrees with `safe_lengths`. The indexer's loop
    /// pushes new hashes per block before `safe_lengths` advances (that
    /// only happens after the compute pass via
    /// [`Indexer::advance_safe_lengths`]); reading from a live cache
    /// here would mint a tip ahead of every safe-bound endpoint and
    /// cause cache etags to invalidate before the data they cover is
    /// actually queryable.
    pub fn tip_blockhash(&self) -> BlockHash {
        match self.safe_lengths().height.decremented() {
            Some(h) => self.vecs.blocks.blockhash.collect_one(h).unwrap_or_default(),
            None => BlockHash::default(),
        }
    }

    /// Pipeline-safe `Lengths` snapshot shared with `Query`. Writers
    /// advance and lower this internally; readers clamp non-series
    /// answers against this loaded snapshot.
    pub fn safe_lengths(&self) -> Lengths {
        self.safe_lengths.load()
    }
}

impl Indexer<Ro> {
    /// Live indexer stamp for diagnostics. For data reads use
    /// [`crate::SafeLengths::load`] (via `Query::height`).
    pub fn indexed_height(&self) -> Height {
        Height::from(self.vecs.blocks.blockhash.inner.stamp())
    }
}

impl Indexer {
    pub fn forced_import(outputs_dir: &Path) -> Result<Self> {
        Self::forced_import_inner(outputs_dir, true)
    }

    fn forced_import_inner(outputs_dir: &Path, can_retry: bool) -> Result<Self> {
        info!("Importing indexer...");

        let indexed_path = outputs_dir.join("indexed");

        let try_import = || -> Result<Self> {
            let i = Instant::now();
            let vecs = Vecs::forced_import(&indexed_path, VERSION)?;
            info!("Imported vecs in {:?}", i.elapsed());

            let i = Instant::now();
            let stores = Stores::forced_import(&indexed_path, VERSION)?;
            info!("Imported stores in {:?}", i.elapsed());

            let safe_lengths = SafeLengths::new();
            if let Some(lengths) = Lengths::from_local(&vecs, &stores) {
                safe_lengths.advance(lengths);
            }

            Ok(Self {
                path: indexed_path.clone(),
                vecs,
                stores,
                safe_lengths,
            })
        };

        match try_import() {
            Ok(result) => Ok(result),
            Err(err) if err.is_lock_error() => {
                // Lock errors are transient - another process has the database open.
                // Don't delete data, just return the error.
                Err(err)
            }
            Err(err) if can_retry && err.is_data_error() => {
                // Data corruption or version mismatch - safe to delete and retry
                info!("{err:?}, deleting {indexed_path:?} and retrying");
                fs::remove_dir_all(&indexed_path)?;
                Self::forced_import_inner(outputs_dir, false)
            }
            Err(err) => Err(err),
        }
    }

    /// Fully resets the indexer by deleting stores from disk and reimporting.
    /// Unlike stores.reset() which uses keyspace.clear() (leaving a journal
    /// record that gets replayed on every recovery), this cleanly recreates.
    fn full_reset(&mut self) -> Result<()> {
        info!("Full reset...");
        self.safe_lengths.reset();
        self.vecs.reset()?;
        let stores_path = self.path.join("stores");
        fs::remove_dir_all(&stores_path).ok();
        self.stores = Stores::forced_import(&self.path, VERSION)?;
        Ok(())
    }

    pub fn index(&mut self, reader: &Reader, client: &Client, exit: &Exit) -> Result<()> {
        self.index_(reader, client, exit, false)
    }

    pub fn checked_index(&mut self, reader: &Reader, client: &Client, exit: &Exit) -> Result<()> {
        self.index_(reader, client, exit, true)
    }

    fn index_(
        &mut self,
        reader: &Reader,
        client: &Client,
        exit: &Exit,
        check_collisions: bool,
    ) -> Result<()> {
        self.vecs.db.sync_bg_tasks()?;

        self.check_xor_bytes(reader)?;

        debug!("Starting indexing...");

        let last_blockhash = shared_tip_height(self.vecs.next_height(), self.stores.next_height())
            .and_then(|height| {
                self.vecs
                    .blocks
                    .blockhash
                    .collect_one_at(usize::from(height))
            })
            .or_else(|| self.vecs.blocks.blockhash.collect_last());
        // Rollback sim
        // let last_blockhash = self
        //     .vecs
        //     .blocks
        //     .blockhash
        //     .collect_one_at(self.vecs.blocks.blockhash.len() - 2);
        debug!("Last block hash found.");

        let (starting_lengths, prev_hash) = if let Some(hash) = last_blockhash {
            let (height, hash) = client.get_closest_valid_height(hash)?;
            match Lengths::resume_at(height.incremented(), &self.vecs, &self.stores) {
                Some(starting_lengths) => {
                    if starting_lengths.height > client.get_last_height()? {
                        info!("Up to date, nothing to index.");
                        return Ok(());
                    }
                    (starting_lengths, Some(hash))
                }
                None => {
                    info!("Data inconsistency detected, resetting indexer...");
                    self.full_reset()?;
                    (Lengths::default(), None)
                }
            }
        } else {
            (Lengths::default(), None)
        };
        debug!("Starting lengths set.");

        let lock = exit.lock();
        self.safe_lengths.lower_before(&starting_lengths);
        self.stores
            .rollback_if_needed(&mut self.vecs, &starting_lengths)?;
        debug!("Rollback stores done.");
        self.vecs.rollback_if_needed(&starting_lengths)?;
        debug!("Rollback vecs done.");
        drop(lock);

        let mut lengths = starting_lengths;

        let is_export_height =
            |height: Height| -> bool { height != 0 && height % SNAPSHOT_BLOCK_RANGE == 0 };

        let export = move |stores: &mut Stores, vecs: &mut Vecs, height: Height| -> Result<()> {
            info!("Exporting...");
            let i = Instant::now();
            let _lock = exit.lock();
            vecs.flush(height)?;
            stores.commit(height)?;
            info!("Exported in {:?}", i.elapsed());
            Ok(())
        };

        let mut readers = Readers::new(&self.vecs);
        let mut buffers = BlockBuffers::default();

        let vecs = &mut self.vecs;
        let stores = &mut self.stores;

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

            processor.push_block_size_and_weight(&txs)?;

            let (txins_result, txouts_result) = rayon::join(
                || processor.process_inputs(&txs, &mut buffers.txid_prefix_map),
                || processor.process_outputs(),
            );
            let txins = txins_result?;
            let txouts = txouts_result?;

            let tx_count = block.txdata.len();
            let input_count = txins.len();
            let output_count = txouts.len();

            BlockProcessor::collect_same_block_spent_outpoints(
                &txins,
                &mut buffers.same_block_spent,
            );

            processor.check_txid_collisions(&txs)?;

            let sigops = processor.compute_sigops(&txins, &txouts);

            processor.finalize_and_store_metadata(
                txs,
                txouts,
                txins,
                sigops,
                &buffers.same_block_spent,
                &mut buffers.already_added_addrs,
                &mut buffers.same_block_output_info,
            )?;

            processor
                .lengths
                .add_block(tx_count, input_count, output_count);

            if is_export_height(height) {
                drop(readers);
                export(stores, vecs, height)?;
                readers = Readers::new(vecs);
            }
        }

        drop(readers);

        let lock = exit.lock();
        let commit = self.stores.take_pending_commit(lengths.height)?;
        self.vecs.stamped_write(lengths.height)?;

        self.vecs.db.run_bg(move |db| {
            let _lock = lock;

            db.bg_sleep(Duration::from_secs(3));

            info!("Exporting...");
            let i = Instant::now();

            db.compact()?;
            commit().map_err(RawDBError::other)?;

            info!("Exported in {:?}", i.elapsed());
            Ok(())
        });

        Ok(())
    }

    fn check_xor_bytes(&mut self, reader: &Reader) -> Result<()> {
        let current = reader.xor_bytes();
        let cached = XORBytes::from(self.path.as_path());

        if cached == current {
            return Ok(());
        }

        self.full_reset()?;

        fs::write(self.path.join("xor.dat"), *current)?;

        Ok(())
    }

    /// Publish disk state as the new safe-lengths snapshot. Drains pending
    /// bg ingest first so stores are queryable at the new bound.
    pub fn advance_safe_lengths(&mut self) -> Result<()> {
        self.vecs.db.sync_bg_tasks()?;
        if let Some(lengths) = Lengths::from_local(&self.vecs, &self.stores) {
            self.safe_lengths.advance(lengths);
        }
        Ok(())
    }
}

impl ReadOnlyClone for Indexer {
    type ReadOnly = Indexer<Ro>;

    fn read_only_clone(&self) -> Indexer<Ro> {
        Indexer {
            path: self.path.clone(),
            vecs: self.vecs.read_only_clone(),
            stores: self.stores.clone(),
            safe_lengths: self.safe_lengths.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_tip_uses_lower_checkpoint() {
        assert_eq!(
            shared_tip_height(Height::new(43), Height::new(40)),
            Some(Height::new(39))
        );
        assert_eq!(
            shared_tip_height(Height::new(40), Height::new(43)),
            Some(Height::new(39))
        );
        assert_eq!(shared_tip_height(Height::ZERO, Height::new(43)), None);
    }
}
