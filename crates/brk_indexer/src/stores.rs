use std::{fs, path::Path, time::Instant};

use rustc_hash::FxHashSet;

use brk_cohort::ByAddrType;
use brk_error::Result;
use brk_store::{AnyStore, Kind, Mode, Store, StoreTask};
use brk_types::{
    AddrHash, AddrIndexOutPoint, AddrIndexTxIndex, BlockHashPrefix, Height, OutPoint, OutputType,
    TxIndex, TxOutIndex, TxidPrefix, TypeIndex, Unit, Version, Vout,
};
use fjall::{Database, PersistMode};
use rayon::prelude::*;
use tracing::{debug, info};
use vecdb::{AnyVec, ReadableVec, VecIndex};

use crate::{Lengths, constants::DUPLICATE_TXID_PREFIXES};

use super::Vecs;

#[derive(Clone)]
pub struct Stores {
    pub db: Database,

    pub addr_type_to_addr_hash_to_addr_index: ByAddrType<Store<AddrHash, TypeIndex>>,
    pub addr_type_to_addr_index_and_tx_index: ByAddrType<Store<AddrIndexTxIndex, Unit>>,
    pub addr_type_to_addr_index_and_unspent_outpoint: ByAddrType<Store<AddrIndexOutPoint, Unit>>,
    pub blockhash_prefix_to_height: Store<BlockHashPrefix, Height>,
    pub txid_prefix_to_tx_index: Store<TxidPrefix, TxIndex>,
}

impl Stores {
    pub fn forced_import(parent: &Path, version: Version) -> Result<Self> {
        Self::forced_import_inner(parent, version, true)
    }

    fn forced_import_inner(parent: &Path, version: Version, can_retry: bool) -> Result<Self> {
        let pathbuf = parent.join("stores");
        let path = pathbuf.as_path();

        fs::create_dir_all(&pathbuf)?;

        let database = match brk_store::open_database(path) {
            Ok(database) => database,
            Err(err) if can_retry => {
                info!("Failed to open stores at {path:?}: {err:?}, deleting and retrying");
                fs::remove_dir_all(path)?;
                return Self::forced_import_inner(parent, version, false);
            }
            Err(err) => return Err(err.into()),
        };

        let database_ref = &database;

        let create_addr_hash_to_addr_index_store = |index| {
            Store::import(
                database_ref,
                path,
                &format!("h2i{}", index),
                version,
                Mode::PushOnly,
                Kind::Random,
            )
        };

        let create_addr_index_to_tx_index_store = |index| {
            Store::import(
                database_ref,
                path,
                &format!("a2t{}", index),
                version,
                Mode::PushOnly,
                Kind::Vec,
            )
        };

        let create_addr_index_to_unspent_outpoint_store = |index| {
            Store::import(
                database_ref,
                path,
                &format!("a2u{}", index),
                version,
                Mode::Any,
                Kind::Vec,
            )
        };

        let stores = Self {
            db: database.clone(),

            addr_type_to_addr_hash_to_addr_index: ByAddrType::new_with_index(
                create_addr_hash_to_addr_index_store,
            )?,
            addr_type_to_addr_index_and_tx_index: ByAddrType::new_with_index(
                create_addr_index_to_tx_index_store,
            )?,
            addr_type_to_addr_index_and_unspent_outpoint: ByAddrType::new_with_index(
                create_addr_index_to_unspent_outpoint_store,
            )?,
            blockhash_prefix_to_height: Store::import(
                database_ref,
                path,
                "blockhash_prefix_to_height",
                version,
                Mode::PushOnly,
                Kind::Random,
            )?,
            txid_prefix_to_tx_index: Store::import_cached(
                database_ref,
                path,
                "txid_prefix_to_tx_index",
                version,
                Mode::PushOnly,
                Kind::Recent,
                5,
            )?,
        };

        Ok(stores)
    }

    pub fn next_height(&self) -> Height {
        self.iter_any()
            .map(|store| store.height().map(Height::incremented).unwrap_or_default())
            .min()
            .unwrap()
    }

    fn iter_any(&self) -> impl Iterator<Item = &dyn AnyStore> {
        [
            &self.blockhash_prefix_to_height as &dyn AnyStore,
            &self.txid_prefix_to_tx_index,
        ]
        .into_iter()
        .chain(
            self.addr_type_to_addr_hash_to_addr_index
                .values()
                .map(|s| s as &dyn AnyStore),
        )
        .chain(
            self.addr_type_to_addr_index_and_tx_index
                .values()
                .map(|s| s as &dyn AnyStore),
        )
        .chain(
            self.addr_type_to_addr_index_and_unspent_outpoint
                .values()
                .map(|s| s as &dyn AnyStore),
        )
    }

    fn par_iter_any_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStore> {
        [
            &mut self.blockhash_prefix_to_height as &mut dyn AnyStore,
            &mut self.txid_prefix_to_tx_index,
        ]
        .into_par_iter()
        .chain(
            self.addr_type_to_addr_hash_to_addr_index
                .par_values_mut()
                .map(|s| s as &mut dyn AnyStore),
        )
        .chain(
            self.addr_type_to_addr_index_and_tx_index
                .par_values_mut()
                .map(|s| s as &mut dyn AnyStore),
        )
        .chain(
            self.addr_type_to_addr_index_and_unspent_outpoint
                .par_values_mut()
                .map(|s| s as &mut dyn AnyStore),
        )
    }

    pub fn commit(&mut self, height: Height) -> Result<()> {
        let i = Instant::now();
        self.ingest_and_persist()?;
        self.par_iter_any_mut()
            .try_for_each(|store| store.export_meta_if_needed(height))?;
        debug!("Stores committed in {:?}", i.elapsed());

        Ok(())
    }

    /// Takes all pending puts/dels and returns one ordered background commit.
    pub fn take_pending_commit(&mut self, height: Height) -> Result<StoreTask> {
        let h = height;
        let mut ingests = Vec::new();
        let mut checkpoints = Vec::new();

        macro_rules! take {
            ($store:expr) => {
                let (ingest, checkpoint) = $store.take_pending_ingest_parts(h)?;
                ingests.extend(ingest);
                checkpoints.push(checkpoint);
            };
        }

        take!(self.blockhash_prefix_to_height);
        take!(self.txid_prefix_to_tx_index);

        for store in self.addr_type_to_addr_hash_to_addr_index.values_mut() {
            take!(store);
        }
        for store in self.addr_type_to_addr_index_and_tx_index.values_mut() {
            take!(store);
        }
        for store in self
            .addr_type_to_addr_index_and_unspent_outpoint
            .values_mut()
        {
            take!(store);
        }

        let db = self.db.clone();

        Ok(Box::new(move || {
            if !ingests.is_empty() {
                for ingest in ingests {
                    ingest()?;
                }
                db.persist(PersistMode::SyncData)?;
            }
            for checkpoint in checkpoints {
                checkpoint()?;
            }
            Ok(())
        }))
    }

    /// Compatibility wrapper returning the ordered commit as a single task.
    pub fn take_all_pending_ingests(&mut self, height: Height) -> Result<Vec<StoreTask>> {
        Ok(vec![self.take_pending_commit(height)?])
    }

    /// Rewrites reverse-key entries below the lowered bound. In-flight
    /// readers may briefly see torn state.
    pub fn rollback_if_needed(
        &mut self,
        vecs: &mut Vecs,
        starting_lengths: &Lengths,
    ) -> Result<()> {
        if self.is_empty()? {
            return Ok(());
        }

        debug_assert!(starting_lengths.height != Height::ZERO);
        debug_assert!(starting_lengths.tx_index != TxIndex::ZERO);
        debug_assert!(starting_lengths.txout_index != TxOutIndex::ZERO);

        self.rollback_block_metadata(vecs, starting_lengths)?;
        self.rollback_txids(vecs, starting_lengths);
        self.rollback_outputs_and_inputs(vecs, starting_lengths);

        let rollback_height = starting_lengths.height.decremented().unwrap_or_default();
        self.par_iter_any_mut()
            .try_for_each(|store| store.export_meta_sync(rollback_height))?;
        self.ingest_and_persist()?;

        Ok(())
    }

    fn ingest_and_persist(&mut self) -> Result<()> {
        self.par_iter_any_mut()
            .try_for_each(|store| store.ingest_pending())?;

        let i = Instant::now();
        self.db.persist(PersistMode::SyncData)?;
        debug!("Stores persisted in {:?}", i.elapsed());

        Ok(())
    }

    fn is_empty(&self) -> Result<bool> {
        Ok(self.blockhash_prefix_to_height.is_empty()?
            && self.txid_prefix_to_tx_index.is_empty()?
            && self
                .addr_type_to_addr_hash_to_addr_index
                .values()
                .try_fold(true, |acc, s| s.is_empty().map(|empty| acc && empty))?
            && self
                .addr_type_to_addr_index_and_tx_index
                .values()
                .try_fold(true, |acc, s| s.is_empty().map(|empty| acc && empty))?
            && self
                .addr_type_to_addr_index_and_unspent_outpoint
                .values()
                .try_fold(true, |acc, s| s.is_empty().map(|empty| acc && empty))?)
    }

    fn rollback_block_metadata(
        &mut self,
        vecs: &mut Vecs,
        starting_lengths: &Lengths,
    ) -> Result<()> {
        vecs.blocks.blockhash.for_each_range_at(
            starting_lengths.height.to_usize(),
            vecs.blocks.blockhash.len(),
            |blockhash| {
                self.blockhash_prefix_to_height
                    .remove(BlockHashPrefix::from(blockhash));
            },
        );

        for addr_type in OutputType::ADDR_TYPES {
            for hash in vecs.iter_addr_hashes_from(addr_type, starting_lengths.height)? {
                self.addr_type_to_addr_hash_to_addr_index
                    .get_mut_unwrap(addr_type)
                    .remove(hash);
            }
        }

        Ok(())
    }

    fn rollback_txids(&mut self, vecs: &mut Vecs, starting_lengths: &Lengths) {
        let start = starting_lengths.tx_index.to_usize();
        let end = vecs.transactions.txid.len();
        let mut current_index = start;
        vecs.transactions
            .txid
            .for_each_range_at(start, end, |txid| {
                let tx_index = TxIndex::from(current_index);
                let txid_prefix = TxidPrefix::from(&txid);

                let is_known_dup =
                    DUPLICATE_TXID_PREFIXES
                        .iter()
                        .any(|(dup_prefix, dup_tx_index)| {
                            tx_index == *dup_tx_index && txid_prefix == *dup_prefix
                        });

                if !is_known_dup {
                    self.txid_prefix_to_tx_index.remove(txid_prefix);
                }
                current_index += 1;
            });

        self.txid_prefix_to_tx_index.clear_caches();
    }

    fn rollback_outputs_and_inputs(&mut self, vecs: &mut Vecs, starting_lengths: &Lengths) {
        let tx_index_to_first_txout_index_reader = vecs.transactions.first_txout_index.reader();
        let txout_index_to_output_type_reader = vecs.outputs.output_type.reader();
        let txout_index_to_type_index_reader = vecs.outputs.type_index.reader();

        let mut addr_index_tx_index_to_remove: FxHashSet<(OutputType, TypeIndex, TxIndex)> =
            FxHashSet::default();

        let rollback_start = starting_lengths.txout_index.to_usize();
        let rollback_end = vecs.outputs.output_type.len();

        let tx_indexes: Vec<TxIndex> = vecs
            .outputs
            .tx_index
            .collect_range_at(rollback_start, rollback_end);

        for (i, txout_index) in (rollback_start..rollback_end).enumerate() {
            let output_type = txout_index_to_output_type_reader.get(txout_index);
            if !output_type.is_addr() {
                continue;
            }

            let addr_type = output_type;
            let addr_index = txout_index_to_type_index_reader.get(txout_index);
            let tx_index = tx_indexes[i];

            addr_index_tx_index_to_remove.insert((addr_type, addr_index, tx_index));

            let vout = Vout::from(
                txout_index
                    - tx_index_to_first_txout_index_reader
                        .get(tx_index.to_usize())
                        .to_usize(),
            );
            let outpoint = OutPoint::new(tx_index, vout);

            self.addr_type_to_addr_index_and_unspent_outpoint
                .get_mut_unwrap(addr_type)
                .remove(AddrIndexOutPoint::from((addr_index, outpoint)));
        }

        let start = starting_lengths.txin_index.to_usize();
        let end = vecs.inputs.outpoint.len();
        let outpoints: Vec<OutPoint> = vecs.inputs.outpoint.collect_range_at(start, end);
        let spending_tx_indexes: Vec<TxIndex> = vecs.inputs.tx_index.collect_range_at(start, end);

        let outputs_to_unspend: Vec<_> = outpoints
            .into_iter()
            .zip(spending_tx_indexes)
            .filter_map(|(outpoint, spending_tx_index)| {
                if outpoint.is_coinbase() {
                    return None;
                }

                let output_tx_index = outpoint.tx_index();
                let vout = outpoint.vout();
                let txout_index =
                    tx_index_to_first_txout_index_reader.get(output_tx_index.to_usize()) + vout;

                if txout_index < starting_lengths.txout_index {
                    let output_type = txout_index_to_output_type_reader.get(txout_index.to_usize());
                    let type_index = txout_index_to_type_index_reader.get(txout_index.to_usize());
                    Some((outpoint, output_type, type_index, spending_tx_index))
                } else {
                    None
                }
            })
            .collect();

        for (outpoint, output_type, type_index, spending_tx_index) in outputs_to_unspend {
            if output_type.is_addr() {
                let addr_type = output_type;
                let addr_index = type_index;

                addr_index_tx_index_to_remove.insert((addr_type, addr_index, spending_tx_index));

                self.addr_type_to_addr_index_and_unspent_outpoint
                    .get_mut_unwrap(addr_type)
                    .insert(AddrIndexOutPoint::from((addr_index, outpoint)), Unit);
            }
        }

        for (addr_type, addr_index, tx_index) in addr_index_tx_index_to_remove {
            self.addr_type_to_addr_index_and_tx_index
                .get_mut_unwrap(addr_type)
                .remove(AddrIndexTxIndex::from((addr_index, tx_index)));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, process};

    use super::*;

    #[test]
    fn pending_commit_persists_data_and_all_checkpoints() -> Result<()> {
        let path = std::env::temp_dir().join(format!(
            "brk-indexer-empty-store-checkpoints-{}",
            process::id()
        ));
        let _ = fs::remove_dir_all(&path);

        let height = Height::new(42);
        let key = BlockHashPrefix::from(7_u64);
        let mut stores = Stores::forced_import(&path, Version::ZERO)?;
        stores.blockhash_prefix_to_height.insert(key, height);
        let mut commits = stores.take_all_pending_ingests(height)?;
        assert_eq!(commits.len(), 1);
        commits.pop().unwrap()()?;

        assert_eq!(stores.next_height(), height.incremented());
        assert_eq!(
            stores
                .blockhash_prefix_to_height
                .get(&key)?
                .map(|value| *value),
            Some(height)
        );
        drop(stores);

        let stores = Stores::forced_import(&path, Version::ZERO)?;
        assert_eq!(stores.next_height(), height.incremented());
        assert_eq!(
            stores
                .blockhash_prefix_to_height
                .get(&key)?
                .map(|value| *value),
            Some(height)
        );
        drop(stores);
        fs::remove_dir_all(path)?;

        Ok(())
    }
}
