use std::{fs, ops::Range, path::Path, time::Instant};

use bitview_cohort::{AddrTypeId, ByAddrType};
use bitview_primitives::{
    AddrHash, AddrIndexOutPoint, AddrIndexTxIndex, BlockHashPrefix, Index40, TxOutIndex, TypeIndex,
};
use brk_error::{Error, OptionData, Result};
use brk_store::{AnyStore, Kind, PendingIngest, Store, open_database};
use brk_types::{Height, OutPoint, OutputType, TxIndex, TxidPrefix, Unit, Version, Vout};
use fjall::Database;
use rayon::{join, prelude::*};
use rustc_hash::FxHashSet;
use tracing::debug;
use vecdb::{AnyVec, ReadableVec, VecIndex};

use super::Vecs;
use crate::{Lengths, constants::DUPLICATE_TXID_PREFIXES};
use checkpoint::{
    DeferredStoresCommit, PendingStoresCheckpoint, PersistedStoresCheckpoint, StoresCheckpoint,
};

pub mod checkpoint;
pub mod transaction;
mod utxo;

pub use transaction::TransactionStoresMut;
pub use utxo::{Utxo, UtxoKey};

#[derive(Clone)]
pub struct Stores {
    db: Database,
    checkpoint: StoresCheckpoint,

    addr_type_to_addr_hash_to_addr_index: ByAddrType<Store<AddrHash, TypeIndex>>,
    addr_type_to_addr_index_and_tx_index: ByAddrType<Store<AddrIndexTxIndex, Unit>>,
    addr_type_to_addr_index_and_unspent_outpoint: ByAddrType<Store<AddrIndexOutPoint, Unit>>,
    blockhash_prefix_to_height: Store<BlockHashPrefix, Height>,
    txid_prefix_to_tx_index: Store<TxidPrefix, TxIndex>,
    /// Every output not spent yet but OP_RETURN ones: what resolving an input needs, in one lookup.
    utxos: Store<UtxoKey, Utxo>,
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

    /// The unspent output under `key`, as last ingested: it can be above the published safe lengths, and the key
    /// holds only an 8-byte txid prefix, so readers outside the indexer check both. Spent outputs miss.
    #[inline]
    pub fn utxo(&self, key: &UtxoKey) -> Result<Option<Utxo>> {
        self.utxos.get(key)
    }

    pub fn import(parent: &Path, version: Version) -> Result<Self> {
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
        let create_utxo_store =
            || Store::import(database_ref, path, "utxos", version, Kind::Recent);
        let create_prefix_stores = || {
            join(
                || join(create_blockhash_prefix_store, create_txid_prefix_store),
                create_utxo_store,
            )
        };

        // Every store owns an independent keyspace, so all 27 can recover in parallel.
        let (
            ((addr_hashes, addr_tx_indexes), addr_unspent_outpoints),
            ((blockhashes, txids), utxos),
        ) = join(create_address_stores, create_prefix_stores);

        let stores = Self {
            db: database.clone(),
            checkpoint: StoresCheckpoint::new(path),
            addr_type_to_addr_hash_to_addr_index: addr_hashes?,
            addr_type_to_addr_index_and_tx_index: addr_tx_indexes?,
            addr_type_to_addr_index_and_unspent_outpoint: addr_unspent_outpoints?,
            blockhash_prefix_to_height: blockhashes?,
            txid_prefix_to_tx_index: txids?,
            utxos: utxos?,
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
            &mut self.utxos,
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
        take!(self.utxos);

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
            && self.utxos.is_empty()?
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
        let first_txout_indexes = vecs
            .transactions
            .first_txout_index
            .collect_range_at(
                starting_tx_index.to_usize(),
                vecs.transactions.first_txout_index.len(),
            )
            .into_iter()
            .map(Index40::get)
            .collect::<Vec<_>>();

        if !valid_rollback_boundaries(&first_txout_indexes, rollback_start, rollback_end) {
            return Err(Error::Internal("Invalid rollback output boundaries"));
        }

        let txids = vecs.transactions.txid.reader();
        let removed_txids = vecs
            .transactions
            .txid
            .collect_range_at(starting_tx_index.to_usize(), vecs.transactions.txid.len());
        debug_assert_eq!(removed_txids.len(), first_txout_indexes.len());

        for ((tx_index, txout_range), txid) in txout_ranges(
            starting_tx_index,
            &first_txout_indexes,
            TxOutIndex::from(rollback_end),
        )
        .zip(&removed_txids)
        {
            let txid_prefix = TxidPrefix::from(txid);
            for (vout, txout_index) in txout_range.enumerate() {
                let output_type = txout_index_to_output_type_reader.get_at(txout_index);
                if !output_type.is_unspendable() {
                    self.utxos
                        .remove(UtxoKey::new(txid_prefix, Vout::from(vout)));
                }
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
        let mut txout_indexes = vecs.inputs.txout_index.cursor();
        let mut output_types = vecs.inputs.output_type.cursor();
        let mut type_indexes = vecs.inputs.type_index.cursor();
        let mut spending_tx_indexes = vecs.inputs.tx_index.cursor();

        for index in start..end {
            let outpoint = outpoints.get(index).data()?;
            if outpoint.is_coinbase() {
                continue;
            }
            let output_type = output_types.get(index).data()?;
            let type_index = type_indexes.get(index).data()?;
            // Outputs created by removed transactions went with them; older ones are unspent again.
            if outpoint.tx_index() < starting_tx_index {
                let txid = txids.get(outpoint.tx_index());
                self.utxos.insert(
                    UtxoKey::new(TxidPrefix::from(&txid), outpoint.vout()),
                    Utxo::new(
                        outpoint.tx_index(),
                        txout_indexes.get(index).data()?,
                        output_type,
                        type_index,
                    ),
                );
            }
            if output_type.is_addr() {
                let addr_type = output_type;
                let addr_index = type_index;
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
            utxos: &mut self.utxos,
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
