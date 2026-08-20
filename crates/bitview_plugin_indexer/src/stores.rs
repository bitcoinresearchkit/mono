use brk_error::Result;

use std::{fs, ops::Range, path::Path, time::Instant};

use rustc_hash::FxHashSet;

use bitview_cohort::ByAddrType;
use brk_error::{Error, OptionData};
use brk_store::{AnyStore, Kind, Mode, PendingIngest, Store};
use brk_types::{
    AddrHash, AddrIndexOutPoint, AddrIndexTxIndex, BlockHashPrefix, Height, OutPoint, OutputType,
    TxIndex, TxOutIndex, TxidPrefix, TypeIndex, Unit, Version, Vout,
};
use fjall::Database;
use rayon::prelude::*;
use tracing::debug;
use vecdb::{AnyVec, ReadableVec, VecIndex};

use crate::{Lengths, constants::DUPLICATE_TXID_PREFIXES, vecs::IndexerVecs as _};

use super::Vecs;

mod checkpoint;

use checkpoint::{
    DeferredStoresCommit, PendingStoresCheckpoint, PersistedStoresCheckpoint, StoresCheckpoint,
};

#[derive(Clone)]
pub struct Stores {
    inner: StoresInner,
}

#[derive(Clone)]
struct StoresInner {
    db: Database,
    checkpoint: StoresCheckpoint,

    addr_type_to_addr_hash_to_addr_index: ByAddrType<Store<AddrHash, TypeIndex>>,
    addr_type_to_addr_index_and_tx_index: ByAddrType<Store<AddrIndexTxIndex, Unit>>,
    addr_type_to_addr_index_and_unspent_outpoint: ByAddrType<Store<AddrIndexOutPoint, Unit>>,
    blockhash_prefix_to_height: Store<BlockHashPrefix, Height>,
    txid_prefix_to_tx_index: Store<TxidPrefix, TxIndex>,
}

pub struct TransactionStoresMut<'a> {
    pub addr_hashes: &'a mut ByAddrType<Store<AddrHash, TypeIndex>>,
    pub addr_tx_indexes: &'a mut ByAddrType<Store<AddrIndexTxIndex, Unit>>,
    pub addr_unspent_outpoints: &'a mut ByAddrType<Store<AddrIndexOutPoint, Unit>>,
    pub txid_prefixes: &'a mut Store<TxidPrefix, TxIndex>,
}

pub trait IndexerStores: Sized {
    fn forced_import(parent: &Path, version: Version) -> Result<Self>;
    fn next_height(&self) -> Result<Option<Height>>;
    fn begin_commit(&self, completed_height: Height) -> Result<PendingStoresCheckpoint>;
    fn persist(&mut self, checkpoint: PendingStoresCheckpoint)
    -> Result<PersistedStoresCheckpoint>;
    fn take_deferred_commit(&mut self, completed_height: Height) -> Result<DeferredStoresCommit>;
    fn rollback_if_needed(&mut self, vecs: &Vecs, starting_lengths: &Lengths) -> Result<()>;
    fn insert_block_height(&mut self, prefix: BlockHashPrefix, height: Height);
    fn transaction_stores_mut(&mut self) -> TransactionStoresMut<'_>;
}

impl Stores {
    #[inline]
    pub fn addr_index(&self, addr_type: OutputType, hash: &AddrHash) -> Result<Option<TypeIndex>> {
        Ok(self
            .inner
            .addr_type_to_addr_hash_to_addr_index
            .get(addr_type)
            .data()?
            .get(hash)?
            .map(|index| index.into_owned()))
    }

    pub fn addr_hash_range(
        &self,
        addr_type: OutputType,
        range: Range<AddrHash>,
    ) -> Result<impl DoubleEndedIterator<Item = (AddrHash, TypeIndex)> + '_> {
        Ok(self
            .inner
            .addr_type_to_addr_hash_to_addr_index
            .get(addr_type)
            .data()?
            .range(range))
    }

    pub fn addr_tx_indexes(
        &self,
        addr_type: OutputType,
        addr_index: TypeIndex,
    ) -> Result<impl DoubleEndedIterator<Item = TxIndex> + '_> {
        Ok(self
            .inner
            .addr_type_to_addr_index_and_tx_index
            .get(addr_type)
            .data()?
            .prefix(addr_index)
            .map(|(key, _)| key.tx_index()))
    }

    pub fn addr_tx_indexes_before(
        &self,
        addr_type: OutputType,
        addr_index: TypeIndex,
        before: TxIndex,
    ) -> Result<impl DoubleEndedIterator<Item = TxIndex> + '_> {
        let min = AddrIndexTxIndex::min_for_addr(addr_index);
        let cursor = AddrIndexTxIndex::from((addr_index, before));
        Ok(self
            .inner
            .addr_type_to_addr_index_and_tx_index
            .get(addr_type)
            .data()?
            .range(min..cursor)
            .map(|(key, _)| key.tx_index()))
    }

    pub fn addr_unspent_outpoints(
        &self,
        addr_type: OutputType,
        addr_index: TypeIndex,
    ) -> Result<impl DoubleEndedIterator<Item = (TxIndex, Vout)> + '_> {
        Ok(self
            .inner
            .addr_type_to_addr_index_and_unspent_outpoint
            .get(addr_type)
            .data()?
            .prefix(addr_index)
            .map(|(key, _)| (key.tx_index(), key.vout())))
    }

    #[inline]
    pub fn block_height(&self, prefix: &BlockHashPrefix) -> Result<Option<Height>> {
        Ok(self
            .inner
            .blockhash_prefix_to_height
            .get(prefix)?
            .map(|height| height.into_owned()))
    }

    #[inline]
    pub fn tx_index(&self, prefix: &TxidPrefix) -> Result<Option<TxIndex>> {
        Ok(self
            .inner
            .txid_prefix_to_tx_index
            .get(prefix)?
            .map(|index| index.into_owned()))
    }
}

impl StoresInner {
    fn open(parent: &Path, version: Version) -> Result<Self> {
        let pathbuf = parent.join("stores");
        let path = pathbuf.as_path();

        fs::create_dir_all(&pathbuf)?;
        let database = brk_store::open_database(path)?;

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
            checkpoint: StoresCheckpoint::new(path),

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

        if stores.checkpoint.next_height()?.is_none() && stores.is_empty()? {
            stores.checkpoint.initialize_empty()?;
        }

        Ok(stores)
    }

    fn checkpoint_height(&self) -> Result<Option<Height>> {
        self.checkpoint.next_height()
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

    fn prepare_checkpoint(&self, completed_height: Height) -> Result<PendingStoresCheckpoint> {
        self.checkpoint.begin(completed_height)
    }

    fn persist_checkpoint(
        &mut self,
        checkpoint: PendingStoresCheckpoint,
    ) -> Result<PersistedStoresCheckpoint> {
        let db = self.db.clone();

        let i = Instant::now();
        let persisted = checkpoint.persist(&db, || {
            self.par_iter_any_mut()
                .try_for_each(|store| store.ingest_pending())
        })?;
        debug!("Stores persisted in {:?}", i.elapsed());

        Ok(persisted)
    }

    /// Takes all pending puts/dels from every store and returns closures
    /// that can ingest them on a background thread.
    fn take_pending_ingests(&mut self) -> Vec<PendingIngest> {
        let mut tasks = Vec::new();

        macro_rules! take {
            ($store:expr) => {
                tasks.extend($store.take_pending_ingest());
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

        tasks
    }

    fn defer_commit(&mut self, completed_height: Height) -> Result<DeferredStoresCommit> {
        let checkpoint = self.checkpoint.begin(completed_height)?;
        let ingests = self.take_pending_ingests();
        Ok(DeferredStoresCommit::new(
            self.db.clone(),
            ingests,
            checkpoint,
        ))
    }

    /// Stages reverse-key entries below the lowered bound for persistence.
    fn rollback(&mut self, vecs: &Vecs, starting_lengths: &Lengths) -> Result<()> {
        if self.is_empty()? {
            return Ok(());
        }

        debug_assert!(starting_lengths.height != Height::ZERO);
        debug_assert!(starting_lengths.tx_index != TxIndex::ZERO);
        debug_assert!(starting_lengths.txout_index != TxOutIndex::ZERO);

        self.rollback_block_metadata(vecs, starting_lengths)?;
        self.rollback_txids(vecs, starting_lengths);
        self.rollback_outputs_and_inputs(vecs, starting_lengths)?;

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

    fn rollback_block_metadata(&mut self, vecs: &Vecs, starting_lengths: &Lengths) -> Result<()> {
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

    fn rollback_txids(&mut self, vecs: &Vecs, starting_lengths: &Lengths) {
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

    fn rollback_outputs_and_inputs(
        &mut self,
        vecs: &Vecs,
        starting_lengths: &Lengths,
    ) -> Result<()> {
        let tx_index_to_first_txout_index_reader = vecs.transactions.first_txout_index.reader();
        let txout_index_to_output_type_reader = vecs.outputs.output_type.reader();
        let txout_index_to_type_index_reader = vecs.outputs.type_index.reader();

        let mut addr_index_tx_index_to_remove: FxHashSet<(OutputType, TypeIndex, TxIndex)> =
            FxHashSet::default();

        let rollback_start = starting_lengths.txout_index.to_usize();
        let rollback_end = vecs.outputs.output_type.len();

        let starting_tx_index = starting_lengths.tx_index;
        let first_txout_indexes = vecs.transactions.first_txout_index.collect_range_at(
            starting_tx_index.to_usize(),
            vecs.transactions.first_txout_index.len(),
        );

        if !valid_rollback_boundaries(&first_txout_indexes, rollback_start, rollback_end) {
            return Err(Error::Internal("Invalid rollback output boundaries"));
        }

        for (tx_index, txout_range) in txout_ranges(
            starting_tx_index,
            &first_txout_indexes,
            TxOutIndex::from(rollback_end),
        ) {
            for (vout, txout_index) in txout_range.enumerate() {
                let output_type = txout_index_to_output_type_reader.get_at(txout_index);
                if !output_type.is_addr() {
                    continue;
                }

                let addr_type = output_type;
                let addr_index = txout_index_to_type_index_reader.get_at(txout_index);

                addr_index_tx_index_to_remove.insert((addr_type, addr_index, tx_index));

                let outpoint = OutPoint::new(tx_index, Vout::from(vout));

                self.addr_type_to_addr_index_and_unspent_outpoint
                    .get_mut_unwrap(addr_type)
                    .remove(AddrIndexOutPoint::from((addr_index, outpoint)));
            }
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
                let txout_index = tx_index_to_first_txout_index_reader.get(output_tx_index) + vout;

                if txout_index < starting_lengths.txout_index {
                    let output_type = txout_index_to_output_type_reader.get(txout_index);
                    let type_index = txout_index_to_type_index_reader.get(txout_index);
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

        Ok(())
    }
}

impl IndexerStores for Stores {
    fn forced_import(parent: &Path, version: Version) -> Result<Self> {
        Ok(Self {
            inner: StoresInner::open(parent, version)?,
        })
    }

    fn next_height(&self) -> Result<Option<Height>> {
        self.inner.checkpoint_height()
    }

    fn begin_commit(&self, completed_height: Height) -> Result<PendingStoresCheckpoint> {
        self.inner.prepare_checkpoint(completed_height)
    }

    fn persist(
        &mut self,
        checkpoint: PendingStoresCheckpoint,
    ) -> Result<PersistedStoresCheckpoint> {
        self.inner.persist_checkpoint(checkpoint)
    }

    fn take_deferred_commit(&mut self, completed_height: Height) -> Result<DeferredStoresCommit> {
        self.inner.defer_commit(completed_height)
    }

    fn rollback_if_needed(&mut self, vecs: &Vecs, starting_lengths: &Lengths) -> Result<()> {
        self.inner.rollback(vecs, starting_lengths)
    }

    fn insert_block_height(&mut self, prefix: BlockHashPrefix, height: Height) {
        self.inner.blockhash_prefix_to_height.insert(prefix, height);
    }

    fn transaction_stores_mut(&mut self) -> TransactionStoresMut<'_> {
        TransactionStoresMut {
            addr_hashes: &mut self.inner.addr_type_to_addr_hash_to_addr_index,
            addr_tx_indexes: &mut self.inner.addr_type_to_addr_index_and_tx_index,
            addr_unspent_outpoints: &mut self.inner.addr_type_to_addr_index_and_unspent_outpoint,
            txid_prefixes: &mut self.inner.txid_prefix_to_tx_index,
        }
    }
}

fn valid_rollback_boundaries(
    first_txout_indexes: &[TxOutIndex],
    rollback_start: usize,
    rollback_end: usize,
) -> bool {
    if rollback_start > rollback_end {
        return false;
    }

    let Some(first) = first_txout_indexes.first() else {
        return rollback_start == rollback_end;
    };

    first.to_usize() == rollback_start
        && first_txout_indexes
            .windows(2)
            .all(|pair| pair[0] <= pair[1])
        && first_txout_indexes
            .last()
            .is_some_and(|last| last.to_usize() <= rollback_end)
}

fn txout_ranges(
    starting_tx_index: TxIndex,
    first_txout_indexes: &[TxOutIndex],
    rollback_end: TxOutIndex,
) -> impl Iterator<Item = (TxIndex, std::ops::Range<usize>)> + '_ {
    first_txout_indexes
        .iter()
        .copied()
        .enumerate()
        .map(move |(offset, first)| {
            let end = first_txout_indexes
                .get(offset + 1)
                .copied()
                .unwrap_or(rollback_end);
            (starting_tx_index + offset, first.to_usize()..end.to_usize())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_stores_initialize_zero_checkpoint() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let stores = Stores::forced_import(dir.path(), Version::ZERO)?;

        assert_eq!(stores.next_height()?, Some(Height::ZERO));
        Ok(())
    }

    #[test]
    fn missing_checkpoint_with_data_stays_invalid() -> Result<()> {
        let dir = tempfile::tempdir()?;

        {
            let mut stores = Stores::forced_import(dir.path(), Version::ZERO)?;
            let inner = &mut stores.inner;
            inner
                .blockhash_prefix_to_height
                .insert(BlockHashPrefix::from(1_u64), Height::ZERO);
            inner
                .blockhash_prefix_to_height
                .take_pending_ingest()
                .unwrap()
                .run()?;
            let pending_checkpoint = inner.checkpoint.begin(Height::ZERO)?;
            drop(pending_checkpoint);
        }

        let reopened = Stores::forced_import(dir.path(), Version::ZERO)?;
        assert_eq!(reopened.next_height()?, None);
        Ok(())
    }

    #[test]
    fn synchronous_commit_persists_data_and_checkpoint() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let prefix = BlockHashPrefix::from(1_u64);

        {
            let mut stores = Stores::forced_import(dir.path(), Version::ZERO)?;
            stores
                .inner
                .blockhash_prefix_to_height
                .insert(prefix, Height::ZERO);
            let checkpoint = stores.begin_commit(Height::new(42))?;
            stores.persist(checkpoint)?.publish()?;
        }

        let reopened = Stores::forced_import(dir.path(), Version::ZERO)?;
        assert_eq!(reopened.next_height()?, Some(Height::new(43)));
        assert_eq!(reopened.block_height(&prefix)?, Some(Height::ZERO));
        Ok(())
    }

    #[test]
    fn rollback_output_ranges_reconstruct_tx_indexes_and_vouts() {
        let first_txout_indexes = [100_usize, 103, 103, 105].map(TxOutIndex::from);
        let ranges: Vec<_> = txout_ranges(
            TxIndex::from(40_usize),
            &first_txout_indexes,
            TxOutIndex::from(108_usize),
        )
        .collect();

        assert_eq!(
            ranges,
            [
                (TxIndex::from(40_usize), 100..103),
                (TxIndex::from(41_usize), 103..103),
                (TxIndex::from(42_usize), 103..105),
                (TxIndex::from(43_usize), 105..108),
            ]
        );

        let reconstructed: Vec<_> = ranges
            .into_iter()
            .flat_map(|(tx_index, range)| {
                range
                    .enumerate()
                    .map(move |(vout, txout_index)| (txout_index, tx_index, Vout::from(vout)))
            })
            .collect();

        assert_eq!(
            reconstructed,
            [
                (100, TxIndex::from(40_usize), Vout::from(0_usize)),
                (101, TxIndex::from(40_usize), Vout::from(1_usize)),
                (102, TxIndex::from(40_usize), Vout::from(2_usize)),
                (103, TxIndex::from(42_usize), Vout::from(0_usize)),
                (104, TxIndex::from(42_usize), Vout::from(1_usize)),
                (105, TxIndex::from(43_usize), Vout::from(0_usize)),
                (106, TxIndex::from(43_usize), Vout::from(1_usize)),
                (107, TxIndex::from(43_usize), Vout::from(2_usize)),
            ]
        );
    }

    #[test]
    fn rollback_output_boundaries_are_validated() {
        assert!(valid_rollback_boundaries(
            &[TxOutIndex::from(100_usize), TxOutIndex::from(103_usize)],
            100,
            105,
        ));
        assert!(valid_rollback_boundaries(&[], 100, 100));

        assert!(!valid_rollback_boundaries(
            &[TxOutIndex::from(99_usize)],
            100,
            105,
        ));
        assert!(!valid_rollback_boundaries(
            &[TxOutIndex::from(100_usize), TxOutIndex::from(99_usize)],
            100,
            105,
        ));
        assert!(!valid_rollback_boundaries(
            &[TxOutIndex::from(100_usize), TxOutIndex::from(106_usize)],
            100,
            105,
        ));
        assert!(!valid_rollback_boundaries(&[], 100, 101));
    }
}
