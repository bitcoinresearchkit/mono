use std::{fs, ops::Range, path::Path, time::Instant};

use bitview_cohort::{AddrTypeId, ByAddrType};
use brk_error::{Error, OptionData, Result};
use brk_store::{AnyStore, Kind, PendingIngest, Store, open_database};
use brk_types::{
    AddrHash, AddrIndexOutPoint, AddrIndexTxIndex, BlockHashPrefix, Height, OutPoint, OutputType,
    TxIndex, TxOutIndex, TxidPrefix, TypeIndex, Unit, Version, Vout,
};
use checkpoint::{
    DeferredStoresCommit, PendingStoresCheckpoint, PersistedStoresCheckpoint, StoresCheckpoint,
};
use fjall::Database;
use rayon::{join, prelude::*};
use rustc_hash::FxHashSet;
use tracing::debug;
use vecdb::{AnyVec, ReadableVec, VecIndex};

use super::Vecs;
use crate::{Lengths, constants::DUPLICATE_TXID_PREFIXES};

pub mod checkpoint;
pub mod transaction;

pub use transaction::TransactionStoresMut;

#[derive(Clone)]
pub struct Stores {
    db: Database,
    checkpoint: StoresCheckpoint,

    addr_type_to_addr_hash_to_addr_index: ByAddrType<Store<AddrHash, TypeIndex>>,
    addr_type_to_addr_index_and_tx_index: ByAddrType<Store<AddrIndexTxIndex, Unit>>,
    addr_type_to_addr_index_and_unspent_outpoint: ByAddrType<Store<AddrIndexOutPoint, Unit>>,
    blockhash_prefix_to_height: Store<BlockHashPrefix, Height>,
    txid_prefix_to_tx_index: Store<TxidPrefix, TxIndex>,
}

impl Stores {
    #[inline]
    pub fn addr_index(&self, addr_type: OutputType, hash: &AddrHash) -> Result<Option<TypeIndex>> {
        self.addr_type_to_addr_hash_to_addr_index
            .get(addr_type)
            .data()?
            .get(hash)
    }

    pub fn addr_hash_range(
        &self,
        addr_type: OutputType,
        range: Range<AddrHash>,
    ) -> Result<impl DoubleEndedIterator<Item = (AddrHash, TypeIndex)> + '_> {
        Ok(self
            .addr_type_to_addr_hash_to_addr_index
            .get(addr_type)
            .data()?
            .range(range))
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
            .addr_type_to_addr_index_and_unspent_outpoint
            .get(addr_type)
            .data()?
            .prefix(addr_index)
            .map(|(key, _)| (key.tx_index(), key.vout())))
    }

    #[inline]
    pub fn block_height(&self, prefix: &BlockHashPrefix) -> Result<Option<Height>> {
        self.blockhash_prefix_to_height.get(prefix)
    }

    #[inline]
    pub fn tx_index(&self, prefix: &TxidPrefix) -> Result<Option<TxIndex>> {
        self.txid_prefix_to_tx_index.get(prefix)
    }

    pub fn forced_import(parent: &Path, version: Version) -> Result<Self> {
        let pathbuf = parent.join("stores");
        let path = pathbuf.as_path();

        fs::create_dir_all(&pathbuf)?;
        let database = open_database(path)?;

        let database_ref = &database;

        let create_addr_hash_to_addr_index_store = |id: AddrTypeId| {
            Store::import(
                database_ref,
                path,
                &format!("h2i{}", id as usize),
                version,
                Kind::Random,
            )
        };

        let create_addr_index_to_tx_index_store = |id: AddrTypeId| {
            Store::import(
                database_ref,
                path,
                &format!("a2t{}", id as usize),
                version,
                Kind::Vec,
            )
        };

        let create_addr_index_to_unspent_outpoint_store = |id: AddrTypeId| {
            Store::import(
                database_ref,
                path,
                &format!("a2u{}", id as usize),
                version,
                Kind::Vec,
            )
        };

        let create_blockhash_prefix_store = || {
            Store::import(
                database_ref,
                path,
                "blockhash_prefix_to_height",
                version,
                Kind::Random,
            )
        };
        let create_txid_prefix_store = || {
            Store::import(
                database_ref,
                path,
                "txid_prefix_to_tx_index",
                version,
                Kind::Recent,
            )
        };

        let create_address_stores = || {
            join(
                || {
                    join(
                        || ByAddrType::par_try_from_fn(create_addr_hash_to_addr_index_store),
                        || ByAddrType::par_try_from_fn(create_addr_index_to_tx_index_store),
                    )
                },
                || ByAddrType::par_try_from_fn(create_addr_index_to_unspent_outpoint_store),
            )
        };
        let create_prefix_stores = || join(create_blockhash_prefix_store, create_txid_prefix_store);

        // Every store owns an independent keyspace, so all 26 can recover in parallel.
        let (((addr_hashes, addr_tx_indexes), addr_unspent_outpoints), (blockhashes, txids)) =
            join(create_address_stores, create_prefix_stores);

        let stores = Self {
            db: database.clone(),
            checkpoint: StoresCheckpoint::new(path),
            addr_type_to_addr_hash_to_addr_index: addr_hashes?,
            addr_type_to_addr_index_and_tx_index: addr_tx_indexes?,
            addr_type_to_addr_index_and_unspent_outpoint: addr_unspent_outpoints?,
            blockhash_prefix_to_height: blockhashes?,
            txid_prefix_to_tx_index: txids?,
        };

        if stores.checkpoint.next_height()?.is_none() && stores.is_empty()? {
            stores.checkpoint.initialize_empty()?;
        }

        Ok(stores)
    }

    pub fn next_height(&self) -> Result<Option<Height>> {
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

    pub fn begin_commit(&self, completed_height: Height) -> Result<PendingStoresCheckpoint> {
        self.checkpoint.begin(completed_height)
    }

    pub fn persist(
        &mut self,
        checkpoint: PendingStoresCheckpoint,
    ) -> Result<PersistedStoresCheckpoint> {
        let i = Instant::now();
        let persisted = checkpoint.persist(|| {
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

    pub fn take_deferred_commit(
        &mut self,
        completed_height: Height,
    ) -> Result<DeferredStoresCommit> {
        let checkpoint = self.checkpoint.begin(completed_height)?;
        let ingests = self.take_pending_ingests();
        Ok(DeferredStoresCommit::new(
            self.db.clone(),
            ingests,
            checkpoint,
        ))
    }

    /// Stages reverse-key entries below the lowered bound for persistence.
    pub fn rollback_if_needed(&mut self, vecs: &Vecs, starting_lengths: &Lengths) -> Result<()> {
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
    }

    fn rollback_outputs_and_inputs(
        &mut self,
        vecs: &Vecs,
        starting_lengths: &Lengths,
    ) -> Result<()> {
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
        let mut outpoints = vecs.inputs.outpoint.cursor();
        let mut output_types = vecs.inputs.output_type.cursor();
        let mut type_indexes = vecs.inputs.type_index.cursor();
        let mut spending_tx_indexes = vecs.inputs.tx_index.cursor();

        for index in start..end {
            let outpoint = outpoints.get(index).data()?;
            if outpoint.is_coinbase() {
                continue;
            }
            let output_type = output_types.get(index).data()?;
            if output_type.is_addr() {
                let addr_type = output_type;
                let addr_index = type_indexes.get(index).data()?;
                let spending_tx_index = spending_tx_indexes.get(index).data()?;

                addr_index_tx_index_to_remove.insert((addr_type, addr_index, spending_tx_index));

                // Remove every discarded spend from address history, but only
                // restore outputs created before the removed transactions.
                if outpoint.tx_index() < starting_tx_index {
                    self.addr_type_to_addr_index_and_unspent_outpoint
                        .get_mut_unwrap(addr_type)
                        .insert(AddrIndexOutPoint::from((addr_index, outpoint)), Unit);
                }
            }
        }

        for (addr_type, addr_index, tx_index) in addr_index_tx_index_to_remove {
            self.addr_type_to_addr_index_and_tx_index
                .get_mut_unwrap(addr_type)
                .remove(AddrIndexTxIndex::from((addr_index, tx_index)));
        }

        Ok(())
    }

    pub fn insert_block_height(&mut self, prefix: BlockHashPrefix, height: Height) {
        self.blockhash_prefix_to_height.insert(prefix, height);
    }

    pub fn transaction_stores_mut(&mut self) -> TransactionStoresMut<'_> {
        TransactionStoresMut {
            addr_hashes: &mut self.addr_type_to_addr_hash_to_addr_index,
            addr_tx_indexes: &mut self.addr_type_to_addr_index_and_tx_index,
            addr_unspent_outpoints: &mut self.addr_type_to_addr_index_and_unspent_outpoint,
            txid_prefixes: &mut self.txid_prefix_to_tx_index,
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
) -> impl Iterator<Item = (TxIndex, Range<usize>)> + '_ {
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
    use brk_types::{StoreValue, TxInIndex};
    use tempfile::tempdir;
    use vecdb::{AnyStoredVec, WritableVec};

    use super::*;
    use crate::test_cache::init_cache;

    #[test]
    fn rollback_restores_only_surviving_outputs_and_preserves_older_address_history() -> Result<()>
    {
        init_cache();
        let dir = tempdir()?;
        let mut vecs = Vecs::forced_import(dir.path(), Version::ZERO)?;
        let mut stores = Stores::forced_import(dir.path(), Version::ZERO)?;
        let ty = OutputType::P2PKH;
        let addr = TypeIndex::new(7);
        let removed_addr = TypeIndex::new(8);
        let intermediate_addr = TypeIndex::new(9);
        let outpoint = |tx, vout| OutPoint::new(TxIndex::new(tx), Vout::from(vout));

        for first in [0_usize, 3, 5] {
            vecs.transactions
                .first_txout_index
                .push(TxOutIndex::from(first));
        }
        vecs.transactions.first_txout_index.write()?;
        for (ty, index) in [
            (ty, addr),
            (ty, addr),
            (OutputType::OpReturn, TypeIndex::new(0)),
            (ty, intermediate_addr),
            (ty, removed_addr),
            (ty, removed_addr),
        ] {
            vecs.outputs.output_type.push(ty);
            vecs.outputs.type_index.push(index);
        }
        vecs.outputs.output_type.write()?;
        vecs.outputs.type_index.write()?;

        for (point, ty, index, tx) in [
            (
                OutPoint::COINBASE,
                OutputType::Unknown,
                TypeIndex::COINBASE,
                0,
            ),
            (
                OutPoint::COINBASE,
                OutputType::Unknown,
                TypeIndex::COINBASE,
                1,
            ),
            (outpoint(0, 0_u32), ty, addr, 1),
            (outpoint(0, 1_u32), ty, addr, 2),
            (
                outpoint(0, 2_u32),
                OutputType::OpReturn,
                TypeIndex::new(0),
                2,
            ),
            (outpoint(1, 0_u32), ty, intermediate_addr, 2),
        ] {
            vecs.inputs.outpoint.push(point);
            vecs.inputs.output_type.push(ty);
            vecs.inputs.type_index.push(index);
            vecs.inputs.tx_index.push(TxIndex::new(tx));
        }
        for (index, tx) in [
            (addr, 0),
            (addr, 1),
            (addr, 2),
            (removed_addr, 1),
            (removed_addr, 2),
            (intermediate_addr, 1),
            (intermediate_addr, 2),
        ] {
            stores
                .addr_type_to_addr_index_and_tx_index
                .get_mut_unwrap(ty)
                .insert(AddrIndexTxIndex::from((index, TxIndex::new(tx))), Unit);
        }
        for point in [outpoint(1, 1_u32), outpoint(2, 0_u32)] {
            stores
                .addr_type_to_addr_index_and_unspent_outpoint
                .get_mut_unwrap(ty)
                .insert(AddrIndexOutPoint::from((removed_addr, point)), Unit);
        }
        let checkpoint = stores.begin_commit(Height::new(2))?;
        stores.persist(checkpoint)?.publish()?;

        stores.rollback_outputs_and_inputs(
            &vecs,
            &Lengths {
                tx_index: TxIndex::new(1),
                txout_index: TxOutIndex::new(3),
                txin_index: TxInIndex::new(1),
                ..Lengths::default()
            },
        )?;
        let checkpoint = stores.begin_commit(Height::ZERO)?;
        stores.persist(checkpoint)?.publish()?;
        drop(stores);

        let stores = Stores::forced_import(dir.path(), Version::ZERO)?;
        assert_eq!(
            stores.addr_unspent_outpoints(ty, addr)?.collect::<Vec<_>>(),
            [
                (TxIndex::ZERO, Vout::ZERO),
                (TxIndex::ZERO, Vout::from(1_u32))
            ]
        );
        assert_eq!(stores.addr_unspent_outpoints(ty, removed_addr)?.count(), 0);
        assert_eq!(
            stores
                .addr_unspent_outpoints(ty, intermediate_addr)?
                .count(),
            0
        );
        assert_eq!(
            stores
                .addr_tx_indexes_before(ty, intermediate_addr, TxIndex::new(3))?
                .count(),
            0
        );
        assert_eq!(
            stores
                .addr_tx_indexes_before(ty, addr, TxIndex::new(3))?
                .collect::<Vec<_>>(),
            [TxIndex::ZERO]
        );
        assert_eq!(
            stores
                .addr_tx_indexes_before(ty, removed_addr, TxIndex::new(3))?
                .count(),
            0
        );
        Ok(())
    }

    #[test]
    fn commit_paths_persist_every_store_before_publishing_the_checkpoint() -> Result<()> {
        for deferred in [false, true] {
            let dir = tempdir()?;
            let block = BlockHashPrefix::from(1_u64);
            let prefix = TxidPrefix::from_store_bytes(2_u64.to_be_bytes());
            let hash = AddrHash::new(3);
            let address = TypeIndex::new(9);
            let tx = TxIndex::new(7);
            let point = OutPoint::new(tx, Vout::ZERO);
            let mut stores = Stores::forced_import(dir.path(), Version::ZERO)?;
            assert_eq!(stores.next_height()?, Some(Height::ZERO));
            stores
                .blockhash_prefix_to_height
                .insert(block, Height::new(42));
            stores.txid_prefix_to_tx_index.insert(prefix, tx);
            for ty in OutputType::ADDR_TYPES {
                stores
                    .addr_type_to_addr_hash_to_addr_index
                    .get_mut_unwrap(ty)
                    .insert(hash, address);
                stores
                    .addr_type_to_addr_index_and_tx_index
                    .get_mut_unwrap(ty)
                    .insert(AddrIndexTxIndex::from((address, tx)), Unit);
                stores
                    .addr_type_to_addr_index_and_unspent_outpoint
                    .get_mut_unwrap(ty)
                    .insert(AddrIndexOutPoint::from((address, point)), Unit);
            }
            let persisted = if deferred {
                let commit = stores.take_deferred_commit(Height::new(42))?;
                drop(stores);
                commit.persist()?
            } else {
                let checkpoint = stores.begin_commit(Height::new(42))?;
                let persisted = stores.persist(checkpoint)?;
                drop(stores);
                persisted
            };
            assert_eq!(
                StoresCheckpoint::new(&dir.path().join("stores")).next_height()?,
                None
            );
            persisted.publish()?;

            let stores = Stores::forced_import(dir.path(), Version::ZERO)?;
            assert_eq!(stores.next_height()?, Some(Height::new(43)));
            assert_eq!(stores.block_height(&block)?, Some(Height::new(42)));
            assert_eq!(stores.tx_index(&prefix)?, Some(tx));
            for ty in OutputType::ADDR_TYPES {
                assert_eq!(stores.addr_index(ty, &hash)?, Some(address));
                assert_eq!(
                    stores
                        .addr_tx_indexes_before(ty, address, tx + 1)?
                        .collect::<Vec<_>>(),
                    [tx]
                );
                assert_eq!(
                    stores
                        .addr_unspent_outpoints(ty, address)?
                        .collect::<Vec<_>>(),
                    [(tx, Vout::ZERO)]
                );
            }
        }
        Ok(())
    }
}
