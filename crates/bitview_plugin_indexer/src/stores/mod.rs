use std::{
    fs,
    ops::{Range, RangeBounds},
    path::Path,
    time::Instant,
};

use bitview_primitives::{
    AddrHash, AddrIndexOutPoint, AddrIndexTxIndex, AddrTypeId, BlockHashPrefix, ByAddrType,
    Index40, TxOutIndex, TypeIndex,
};
use brk_error::{OptionData, Result};
use brk_store::{AnyStore, Kind, PendingIngest, Store, StoreView, open_database};
use brk_types::{Height, OutPoint, OutputType, TxIndex, TxidPrefix, Unit, Version, Vout};
use fjall::Database;
use rayon::prelude::*;
use rustc_hash::FxHashSet;
use tracing::debug;
use vecdb::{AnyVec, ReadableVec, VecIndex};

use super::Vecs;
use crate::{Lengths, constants::overwritten_tx_index};
use checkpoint::{
    DeferredStoresCommit, PendingStoresCheckpoint, PersistedStoresCheckpoint, StoresCheckpoint,
};

mod checkpoint;
mod transaction;
mod utxo;

pub(crate) use transaction::TransactionStoresMut;
pub use utxo::{UtxoKey, Utxos};

/// The eight address-unspent stores, pinned as ingested.
pub(crate) type AddrUnspentViews = ByAddrType<StoreView<AddrIndexOutPoint, Unit>>;

#[derive(Clone)]
pub struct Stores {
    db: Database,
    checkpoint: StoresCheckpoint,

    addr_type_to_addr_hash_to_addr_index: ByAddrType<Store<AddrHash, TypeIndex>>,
    addr_type_to_addr_index_and_tx_index: ByAddrType<Store<AddrIndexTxIndex, Unit>>,
    addr_type_to_addr_index_and_unspent_outpoint: ByAddrType<Store<AddrIndexOutPoint, Unit>>,
    blockhash_prefix_to_height: Store<BlockHashPrefix, Height>,
    txid_prefix_to_tx_index: Store<TxidPrefix, TxIndex>,
    utxos: Utxos,
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
        range: impl RangeBounds<AddrHash>,
    ) -> Result<impl DoubleEndedIterator<Item = Result<(AddrHash, TypeIndex)>> + '_> {
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
    ) -> Result<impl DoubleEndedIterator<Item = Result<TxIndex>> + '_> {
        let min = AddrIndexTxIndex::min_for_addr(addr_index);
        let cursor = AddrIndexTxIndex::from((addr_index, before));
        Ok(self
            .addr_type_to_addr_index_and_tx_index
            .get(addr_type)
            .data()?
            .range(min..cursor)
            .map(|record| record.map(|(key, _)| key.tx_index())))
    }

    /// Pins the address-unspent stores as ingested: call it with nothing pending.
    pub(crate) fn addr_unspent_views(&self) -> AddrUnspentViews {
        ByAddrType::from_fn(|id| {
            id.select(&self.addr_type_to_addr_index_and_unspent_outpoint)
                .view()
        })
    }

    #[inline]
    pub fn block_height(&self, prefix: &BlockHashPrefix) -> Result<Option<Height>> {
        self.blockhash_prefix_to_height.get(prefix)
    }

    #[inline]
    pub fn tx_index(&self, prefix: &TxidPrefix) -> Result<Option<TxIndex>> {
        self.txid_prefix_to_tx_index.get(prefix)
    }

    #[inline]
    pub fn utxos(&self) -> &Utxos {
        &self.utxos
    }

    pub(crate) fn import(parent: &Path, version: Version) -> Result<Self> {
        let pathbuf = parent.join("stores");
        let path = pathbuf.as_path();

        fs::create_dir_all(&pathbuf)?;
        let database = open_database(path)?;

        let db = &database;

        // Every store owns an independent keyspace, so all 28 recover in parallel.
        let (
            addr_hashes,
            addr_tx_indexes,
            addr_unspent_outpoints,
            blockhashes,
            txids,
            utxos,
            utxo_values,
        ) = parallel_import! {
            addr_hashes = ByAddrType::par_try_from_fn(|id: AddrTypeId| {
                Store::import(db, path, &format!("h2i{}", id as usize), version, Kind::Random)
            }),
            addr_tx_indexes = ByAddrType::par_try_from_fn(|id: AddrTypeId| {
                Store::import(db, path, &format!("a2t{}", id as usize), version, Kind::Vec)
            }),
            addr_unspent_outpoints = ByAddrType::par_try_from_fn(|id: AddrTypeId| {
                Store::import(db, path, &format!("a2u{}", id as usize), version, Kind::Vec)
            }),
            blockhashes = Store::import(db, path, "blockhash_prefix_to_height", version, Kind::Random),
            txids = Store::import(db, path, "txid_prefix_to_tx_index", version, Kind::Recent),
            utxos = Store::import(db, path, "utxos", version, Kind::Recent),
            utxo_values = Store::import(db, path, "utxo_values", version, Kind::Recent),
        };

        let mut stores = Self {
            db: database.clone(),
            checkpoint: StoresCheckpoint::new(path),
            addr_type_to_addr_hash_to_addr_index: addr_hashes,
            addr_type_to_addr_index_and_tx_index: addr_tx_indexes,
            addr_type_to_addr_index_and_unspent_outpoint: addr_unspent_outpoints,
            blockhash_prefix_to_height: blockhashes,
            txid_prefix_to_tx_index: txids,
            utxos: Utxos {
                entries: utxos,
                large_values: utxo_values,
            },
        };

        if stores.checkpoint.next_height()?.is_none() && stores.is_empty()? {
            stores.checkpoint.initialize_empty()?;
        }

        Ok(stores)
    }

    pub(crate) fn next_height(&self) -> Result<Option<Height>> {
        self.checkpoint.next_height()
    }

    /// Every store, once: commits and the emptiness check walk this one list.
    fn stores_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStore> {
        [
            &mut self.blockhash_prefix_to_height as &mut dyn AnyStore,
            &mut self.txid_prefix_to_tx_index,
            &mut self.utxos.entries,
            &mut self.utxos.large_values,
        ]
        .into_iter()
        .chain(
            self.addr_type_to_addr_hash_to_addr_index
                .values_mut()
                .map(|s| s as &mut dyn AnyStore),
        )
        .chain(
            self.addr_type_to_addr_index_and_tx_index
                .values_mut()
                .map(|s| s as &mut dyn AnyStore),
        )
        .chain(
            self.addr_type_to_addr_index_and_unspent_outpoint
                .values_mut()
                .map(|s| s as &mut dyn AnyStore),
        )
    }

    pub(crate) fn begin_commit(&self, completed_height: Height) -> Result<PendingStoresCheckpoint> {
        self.checkpoint.begin(completed_height)
    }

    pub(crate) fn persist(
        &mut self,
        checkpoint: PendingStoresCheckpoint,
    ) -> Result<PersistedStoresCheckpoint> {
        let i = Instant::now();
        let ingests = self.take_pending_ingests();
        let persisted =
            checkpoint.persist(|| ingests.into_par_iter().try_for_each(PendingIngest::run))?;
        debug!("Stores persisted in {:?}", i.elapsed());

        Ok(persisted)
    }

    fn take_pending_ingests(&mut self) -> Vec<PendingIngest> {
        self.stores_mut()
            .filter_map(|store| store.take_pending_ingest())
            .collect()
    }

    /// Takes every store's pending writes into a commit that begins the checkpoint only when it
    /// persists, so the stores keep their last checkpoint until then.
    pub(crate) fn take_deferred_commit(
        &mut self,
        completed_height: Height,
    ) -> DeferredStoresCommit {
        DeferredStoresCommit::new(
            self.db.clone(),
            self.take_pending_ingests(),
            self.checkpoint.clone(),
            completed_height,
        )
    }

    /// Stages the removal of the blocks from `starting_lengths` on, restoring the older outputs
    /// they spent.
    pub(crate) fn rollback_if_needed(
        &mut self,
        vecs: &Vecs,
        starting_lengths: &Lengths,
    ) -> Result<()> {
        debug_assert!(starting_lengths.height != Height::ZERO);
        debug_assert!(starting_lengths.tx_index != TxIndex::ZERO);
        debug_assert!(starting_lengths.txout_index != TxOutIndex::ZERO);

        self.rollback_block_metadata(vecs, starting_lengths);
        self.rollback_txids(vecs, starting_lengths);
        self.rollback_outputs_and_inputs(vecs, starting_lengths)?;

        Ok(())
    }

    fn is_empty(&mut self) -> Result<bool> {
        self.stores_mut()
            .try_fold(true, |empty, store| Ok(empty && store.is_empty()?))
    }

    fn rollback_block_metadata(&mut self, vecs: &Vecs, starting_lengths: &Lengths) {
        vecs.blocks.blockhash.for_each_range_at(
            starting_lengths.height.to_usize(),
            vecs.blocks.blockhash.len(),
            |blockhash| {
                self.blockhash_prefix_to_height
                    .remove(BlockHashPrefix::from(blockhash));
            },
        );

        for (addr_type, store) in self.addr_type_to_addr_hash_to_addr_index.iter_mut() {
            vecs.addrs
                .for_each_hash_from(addr_type, starting_lengths.height, |hash| {
                    store.remove(hash)
                });
        }
    }

    fn rollback_txids(&mut self, vecs: &Vecs, starting_lengths: &Lengths) {
        let start = starting_lengths.tx_index.to_usize();
        let end = vecs.transactions.txid.len();
        let mut tx_index = starting_lengths.tx_index;
        vecs.transactions
            .txid
            .for_each_range_at(start, end, |txid| {
                // A BIP30 duplicate's txid still maps to the original, which stays.
                if overwritten_tx_index(tx_index).is_none() {
                    self.txid_prefix_to_tx_index.remove(TxidPrefix::from(&txid));
                }
                tx_index = tx_index.incremented();
            });
    }

    fn rollback_outputs_and_inputs(
        &mut self,
        vecs: &Vecs,
        starting_lengths: &Lengths,
    ) -> Result<()> {
        let txout_index_to_output_type_reader = vecs.outputs.output_type.reader();
        let txout_index_to_type_index_reader = vecs.outputs.type_index.reader();
        let txout_index_to_value_reader = vecs.outputs.value.reader();

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

        debug_assert!(
            valid_rollback_boundaries(&first_txout_indexes, rollback_start, rollback_end),
            "Invalid rollback output boundaries"
        );

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
            // A BIP30 duplicate is its original's transaction again: the same keys, output types,
            // addresses and values. Undoing it brings back the original's outputs it overwrote;
            // when the original goes too, its removals cover the shared keys.
            let duplicate_of = overwritten_tx_index(tx_index);
            let restored = match duplicate_of {
                Some(original) if original < starting_tx_index => {
                    let first_txout_index = vecs
                        .transactions
                        .first_txout_index
                        .collect_one(original)
                        .data()?;
                    Some((original, first_txout_index.get()))
                }
                _ => None,
            };
            for (vout, txout_index) in txout_range.enumerate() {
                let output_type = txout_index_to_output_type_reader.get_at(txout_index);
                let type_index = txout_index_to_type_index_reader.get_at(txout_index);
                if !output_type.is_unspendable() {
                    let key = UtxoKey::new(txid_prefix, vout as u32);
                    let value = txout_index_to_value_reader.get_at(txout_index);
                    if let Some((original, first_txout_index)) = restored {
                        // Puts replace the duplicate's records with the original's (equal value).
                        self.utxos.insert(
                            key,
                            original,
                            first_txout_index + TxOutIndex::from(vout),
                            output_type,
                            type_index,
                            value,
                        );
                    } else if duplicate_of.is_none() {
                        self.utxos.remove(key, value);
                    }
                }
                if !output_type.is_addr() {
                    continue;
                }

                let addr_type = output_type;
                let addr_index = type_index;

                addr_index_tx_index_to_remove.insert((addr_type, addr_index, tx_index));

                let vout = Vout::from(vout);
                let outpoints = self
                    .addr_type_to_addr_index_and_unspent_outpoint
                    .get_mut_unwrap(addr_type);
                outpoints.remove(AddrIndexOutPoint::from((
                    addr_index,
                    OutPoint::new(tx_index, vout),
                )));
                if let Some((original, _)) = restored {
                    outpoints.insert(
                        AddrIndexOutPoint::from((addr_index, OutPoint::new(original, vout))),
                        Unit,
                    );
                }
            }
        }

        let start = starting_lengths.txin_index.to_usize();
        let end = vecs.inputs.outpoint.len();
        let mut outpoints = vecs.inputs.outpoint.cursor();
        let mut txout_indexes = vecs.inputs.txout_index.cursor();
        let mut output_types = vecs.inputs.output_type.cursor();
        let mut type_indexes = vecs.inputs.type_index.cursor();
        let mut spending_tx_indexes = vecs.inputs.tx_index.cursor();
        let mut values = vecs.inputs.value.cursor();

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
                    UtxoKey::new(TxidPrefix::from(&txid), u32::from(outpoint.vout())),
                    outpoint.tx_index(),
                    txout_indexes.get(index).data()?,
                    output_type,
                    type_index,
                    values.get(index).data()?,
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

    pub(crate) fn insert_block_height(&mut self, prefix: BlockHashPrefix, height: Height) {
        self.blockhash_prefix_to_height.insert(prefix, height);
    }

    pub(crate) fn transaction_stores_mut(&mut self) -> TransactionStoresMut<'_> {
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
