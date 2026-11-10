use bitview_plugin_outputs::published_spend;
use bitview_primitives::{Lengths, TxInIndex, TxOutIndex};
use brk_mempool::ReadOnlyState;
use brk_types::{BlockHash, Height, Timestamp, TxIndex, TxOutspend, TxStatus, Txid, Vin, Vout};
use rustc_hash::FxHashMap;
use serde_json::to_vec;
use vecdb::{ReadableVec, VecIndex};

use crate::{Error, OptionData, Query, ReadPin, RepresentationId, Result};

impl Query {
    fn outspend(&self, txid: &Txid, vout: Vout) -> Result<TxOutspend> {
        let (mempool, pin) = self.mempool_then_pin()?;
        let safe = pin.lengths();
        let outputs = match self.resolve_tx_outputs(txid, &pin) {
            Ok(outputs) => outputs,
            Err(error) if Self::pool_answers(&error, &pin) => {
                let error = match self.pending_view(mempool.as_deref(), &pin)? {
                    Some((mempool, view)) => {
                        match mempool.outspend_if_present(txid, vout, &view)? {
                            Some(outspend) => return Ok(outspend),
                            None => Self::unanswered(error, &view, txid),
                        }
                    }
                    None => error,
                };
                // An output no known transaction holds is unspent, as Esplora answers.
                return match error {
                    Error::UnknownTxid => Ok(TxOutspend::UNSPENT),
                    error => Err(error),
                };
            }
            Err(error) => return Err(error),
        };
        let Some(txout) = outputs.get(vout) else {
            return Ok(TxOutspend::UNSPENT);
        };
        let mut outspends = [Some(vec![TxOutspend::UNSPENT])];
        self.resolve_outspends(&[Some(txout)], &mut outspends, safe)?;
        let [confirmed] = outspends;
        let confirmed = confirmed.ok_or(Error::UnknownTxid)?.pop().data()?;
        if confirmed.spent {
            return Ok(confirmed);
        }
        // Confirmed spends answer first; the pool, as the pin sees it, adds a pending spend of the
        // rest.
        Ok(match self.pending_view(mempool.as_deref(), &pin)? {
            Some((mempool, view)) => mempool.outspend(txid, vout, &view)?,
            None => TxOutspend::UNSPENT,
        })
    }

    /// Resolve and serialize one outspend exactly once, identifying confirmed
    /// results by their spending block and live results by their content.
    pub fn outspend_json(&self, txid: &Txid, vout: Vout) -> Result<(Vec<u8>, RepresentationId)> {
        let outspend = self.outspend(txid, vout)?;
        let bytes = to_vec(&outspend).unwrap();
        let identity = outspend_identity(&outspend, &bytes);
        Ok((bytes, identity))
    }

    fn outspends(&self, txid: &Txid) -> Result<Vec<TxOutspend>> {
        let (mempool, pin) = self.mempool_then_pin()?;
        self.outspends_at(std::slice::from_ref(txid), mempool.as_deref(), &pin)?
            .pop()
            .flatten()
            .ok_or(Error::UnknownTxid)
    }

    /// Single and batch reads share resolution and the exact same chain/pending overlay rules.
    /// Callers supply unique IDs. A missing transaction is `None`; other failures propagate.
    fn outspends_at(
        &self,
        txids: &[Txid],
        mempool: Option<&ReadOnlyState>,
        pin: &ReadPin,
    ) -> Result<Vec<Option<Vec<TxOutspend>>>> {
        let safe = pin.lengths();
        let mut firsts = Vec::with_capacity(txids.len());
        let mut outspends = Vec::with_capacity(txids.len());
        for txid in txids {
            let outputs = match self.resolve_tx_outputs(txid, pin) {
                Ok(outputs) => outputs,
                Err(error) => {
                    let pending = if Self::pool_answers(&error, pin) {
                        match self.pending_view(mempool, pin)? {
                            Some((mempool, view)) => {
                                match mempool.outspends_if_present(txid, &view)? {
                                    Some(outspends) => Ok(outspends),
                                    None => Err(Self::unanswered(error, &view, txid)),
                                }
                            }
                            None => Err(error),
                        }
                    } else {
                        Err(error)
                    };
                    outspends.push(match pending {
                        Ok(outspends) => Some(outspends),
                        Err(Error::UnknownTxid) => None,
                        Err(error) => return Err(error),
                    });
                    firsts.push(None);
                    continue;
                }
            };
            firsts.push(Some(outputs.first));
            outspends.push(Some(vec![TxOutspend::UNSPENT; outputs.len]));
        }
        self.resolve_outspends(&firsts, &mut outspends, safe)?;
        for ((txid, first), outspends) in txids.iter().zip(firsts).zip(&mut outspends) {
            // Pending parents are already resolved. Only what the chain leaves unspent asks
            // the pool; confirmed spends always win.
            if first.is_some()
                && let Some(outspends) = outspends
                && outspends.iter().any(|outspend| !outspend.spent)
                && let Some((mempool, view)) = self.pending_view(mempool, pin)?
            {
                mempool.merge_outspends(txid, outspends, &view)?;
            }
        }
        Ok(outspends)
    }

    /// Outspends in request order from one chain pin and mempool publication. Repeated IDs
    /// resolve once; an unknown ID has an empty inner array, as mempool/electrs answers.
    pub fn batch_outspends_json(&self, txids: &[Txid]) -> Result<(Vec<u8>, RepresentationId)> {
        let (mempool, pin) = self.mempool_then_pin()?;
        let mut unique = FxHashMap::default();
        let mut parents = Vec::new();
        for txid in txids {
            if let std::collections::hash_map::Entry::Vacant(entry) = unique.entry(*txid) {
                entry.insert(parents.len());
                parents.push(*txid);
            }
        }
        let outspends = self.outspends_at(&parents, mempool.as_deref(), &pin)?;
        drop(pin);
        let ordered: Vec<_> = txids
            .iter()
            .map(|txid| outspends[unique[txid]].as_deref().unwrap_or_default())
            .collect();
        let bytes = to_vec(&ordered).unwrap();
        let identity = RepresentationId::content(&bytes);
        Ok((bytes, identity))
    }

    /// Resolve and serialize all outspends exactly once. Fully confirmed
    /// arrays are identified by their newest spending block; arrays with any
    /// live element are identified by their exact content.
    pub fn outspends_json(&self, txid: &Txid) -> Result<(Vec<u8>, RepresentationId)> {
        let outspends = self.outspends(txid)?;
        let bytes = to_vec(&outspends).unwrap();
        let identity = outspends_identity(&outspends, &bytes);
        Ok((bytes, identity))
    }

    /// Resolve spend status for ranges of outputs, as of the published `bound`: the
    /// spend index changes in place, so spends from unpublished inputs read as unspent.
    /// Resolve compressed inputs in one sorted batch, retaining parent and output order.
    /// Each confirmed destination starts unspent and holds the response directly. An unpublished
    /// spending height clears only its parent, preserving the single-read `UnknownTxid` result.
    fn resolve_outspends(
        &self,
        firsts: &[Option<TxOutIndex>],
        outspends: &mut [Option<Vec<TxOutspend>>],
        bound: Lengths,
    ) -> Result<()> {
        debug_assert_eq!(firsts.len(), outspends.len());
        let indexer = self.indexer();
        let txin_index_reader = self.plugins().outputs.spent.spending_txin_index.reader();
        let mut requested = Vec::new();
        let mut ordered = true;
        for (parent, (&first_txout, spends)) in firsts.iter().zip(&*outspends).enumerate() {
            let (Some(first_txout), Some(spends)) = (first_txout, spends) else {
                continue;
            };
            for index in 0..spends.len() {
                let Some(txin_index) = published_spend(
                    txin_index_reader
                        .try_get(first_txout + Vout::from(index))
                        .data()?,
                    bound,
                ) else {
                    continue;
                };
                ordered &= requested
                    .last()
                    .is_none_or(|&(previous, _)| previous <= txin_index);
                requested.push((txin_index, (parent, index)));
            }
        }
        if requested.is_empty() {
            return Ok(());
        }
        let txid_reader = indexer.vecs().transactions.txid.reader();
        let tx_heights = self.plugins().mappings.tx_heights.load();
        let mut tx_heights = tx_heights.cursor();
        let mut cached_status: Option<(Height, BlockHash, Timestamp)> = None;
        let mut cached_tx: Option<(TxIndex, Txid, Height)> = None;
        visit_spending_positions(
            &indexer.vecs().inputs.tx_index,
            &indexer.vecs().transactions.first_txin_index,
            &mut requested,
            bound.tx_index,
            ordered,
            |(parent, index), spending_tx_index, vin| {
                let (spending_txid, spending_height) = if let Some((tx, txid, height)) = cached_tx
                    && tx == spending_tx_index
                {
                    (txid, height)
                } else {
                    let spending_txid = txid_reader.try_get(spending_tx_index).data()?;
                    let spending_height: Height = tx_heights.get(spending_tx_index).data()?;
                    if spending_height >= bound.height {
                        outspends[parent] = None;
                        return Ok(());
                    }
                    cached_tx = Some((spending_tx_index, spending_txid, spending_height));
                    (spending_txid, spending_height)
                };
                let (block_hash, block_time) = if let Some((height, hash, time)) = cached_status
                    && height == spending_height
                {
                    (hash, time)
                } else {
                    let (hash, time) = self.block_hash_and_time(spending_height)?;
                    cached_status = Some((spending_height, hash, time));
                    (hash, time)
                };

                if let Some(spends) = &mut outspends[parent] {
                    spends[index] = TxOutspend {
                        spent: true,
                        txid: Some(spending_txid),
                        vin: Some(vin),
                        status: Some(TxStatus::confirmed(spending_height, block_hash, block_time)),
                    };
                }
                Ok(())
            },
        )?;

        Ok(())
    }
}

fn visit_spending_positions<O: Copy>(
    input_txs: &impl ReadableVec<TxInIndex, TxIndex>,
    first_inputs: &impl ReadableVec<TxIndex, TxInIndex>,
    requested: &mut [(TxInIndex, O)],
    tx_bound: TxIndex,
    ordered: bool,
    mut visit: impl FnMut(O, TxIndex, Vin) -> Result<()>,
) -> Result<()> {
    if requested.is_empty() {
        return Ok(());
    }
    if ordered {
        // Already ordered reads do not revisit pages. Keep the low-allocation
        // cursor path for small transactions and sequential spending inputs.
        let mut inputs = input_txs.cursor();
        let mut firsts = first_inputs.cursor();
        for &(input, output) in requested.iter() {
            let tx = inputs.get(input.to_usize()).data()?;
            if tx < tx_bound {
                let first = firsts.get(tx.to_usize()).data()?;
                visit(output, tx, checked_vin(input, first)?)?;
            }
        }
        return Ok(());
    }
    requested.sort_unstable_by_key(|&(input, _)| input);
    let mut inputs: Vec<_> = requested
        .iter()
        .map(|&(input, _)| input.to_usize())
        .collect();
    let txs = input_txs.read_sorted_at(&inputs);
    if txs.len() != requested.len() {
        return Err(Error::Internal("Missing spending transaction index"));
    }
    // Input-to-transaction mapping is monotonic. Keep repeated transactions:
    // native sorted reads already share each decoded page across duplicates.
    let valid = txs.partition_point(|&tx| tx < tx_bound);
    inputs.truncate(valid);
    for (index, tx) in inputs.iter_mut().zip(&txs) {
        *index = tx.to_usize();
    }
    let firsts = first_inputs.read_sorted_at(&inputs);
    if firsts.len() != valid {
        return Err(Error::Internal("Missing spending first input index"));
    }
    for ((&(input, output), &tx), first) in requested[..valid].iter().zip(&txs).zip(firsts) {
        visit(output, tx, checked_vin(input, first)?)?;
    }
    Ok(())
}

fn checked_vin(input: TxInIndex, first: TxInIndex) -> Result<Vin> {
    usize::from(input)
        .checked_sub(usize::from(first))
        .filter(|vin| *vin <= usize::from(u16::MAX))
        .map(Vin::from)
        .ok_or(Error::Internal("Invalid spending input position"))
}

fn outspend_identity(outspend: &TxOutspend, bytes: &[u8]) -> RepresentationId {
    confirmed_spending_block(outspend).map_or_else(
        || RepresentationId::content(bytes),
        |(hash, _)| RepresentationId::Block(hash),
    )
}

/// Unspent outputs have no status, and unconfirmed statuses no block.
fn confirmed_spending_block(outspend: &TxOutspend) -> Option<(BlockHash, Height)> {
    let status = outspend.status.as_ref()?;
    status.block_hash.zip(status.block_height)
}

fn outspends_identity(outspends: &[TxOutspend], bytes: &[u8]) -> RepresentationId {
    let content = || RepresentationId::content(bytes);
    let mut newest = None;

    for outspend in outspends {
        let Some((hash, height)) = confirmed_spending_block(outspend) else {
            return content();
        };

        // Confirmed spends read under one pin: one block per height.
        if newest.is_none_or(|(_, newest_height)| height > newest_height) {
            newest = Some((hash, height));
        }
    }

    newest.map_or_else(content, |(hash, _)| RepresentationId::Block(hash))
}

#[cfg(test)]
mod tests {
    use std::{
        marker::PhantomData,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use vecdb::{AnyVec, VecValue, Version};

    use super::*;

    struct TestVec<I, T> {
        values: Vec<T>,
        reads: AtomicUsize,
        index: PhantomData<I>,
    }

    impl<I, T> TestVec<I, T> {
        fn new(values: Vec<T>) -> Self {
            Self {
                values,
                reads: AtomicUsize::new(0),
                index: PhantomData,
            }
        }
    }

    impl<I: VecIndex, T: VecValue> AnyVec for TestVec<I, T> {
        fn version(&self) -> Version {
            Version::ZERO
        }
        fn name(&self) -> &str {
            "outspend-test"
        }
        fn len(&self) -> usize {
            self.values.len()
        }
        fn is_mutable(&self) -> bool {
            false
        }

        vecdb::any_vec_type_methods!(I, T);
    }

    impl<I: VecIndex, T: VecValue> ReadableVec<I, T> for TestVec<I, T> {
        fn cursor_chunk_size(&self) -> usize {
            2
        }

        fn read_into_at(&self, from: usize, to: usize, out: &mut Vec<T>) {
            self.reads.fetch_add(1, Ordering::Relaxed);
            out.extend_from_slice(&self.values[from..to.min(self.len())]);
        }

        fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
            self.values[from..to.min(self.len())]
                .iter()
                .cloned()
                .for_each(f);
        }

        fn try_fold_range_at<B, E, F: FnMut(B, T) -> std::result::Result<B, E>>(
            &self,
            from: usize,
            to: usize,
            init: B,
            f: F,
        ) -> std::result::Result<B, E> {
            self.values[from..to.min(self.len())]
                .iter()
                .cloned()
                .try_fold(init, f)
        }
    }

    #[test]
    fn grouped_spending_positions_scatter_and_share_pages() {
        for ordered in [false, true] {
            let input_txs =
                TestVec::<TxInIndex, _>::new([0, 0, 1, 1, 1, 2, 3, 3].map(TxIndex::new).to_vec());
            let first_inputs =
                TestVec::<TxIndex, _>::new([0, 2, 5, 6].map(TxInIndex::new).to_vec());
            let mut requested = [
                (TxInIndex::new(4), (0, 1)),
                (TxInIndex::new(6), (1, 0)),
                (TxInIndex::new(1), (1, 2)),
                (TxInIndex::new(3), (0, 0)),
                (TxInIndex::new(4), (1, 1)),
                (TxInIndex::new(5), (0, 2)),
            ];
            if ordered {
                requested.sort_unstable_by_key(|&(input, _)| input);
            }
            let mut positions = [[None; 3]; 2];
            visit_spending_positions(
                &input_txs,
                &first_inputs,
                &mut requested,
                TxIndex::new(3),
                ordered,
                |(parent, output), tx, vin| {
                    positions[parent][output] = Some((tx, vin));
                    Ok(())
                },
            )
            .unwrap();

            let position = |tx, vin| Some((TxIndex::new(tx), Vin::from(vin)));
            assert_eq!(
                positions,
                [
                    [position(1, 1_usize), position(1, 2), position(2, 0)],
                    [None, position(1, 2), position(0, 1)],
                ]
            );
            // The duplicate input and shared spending transaction span parents. Each source
            // chunk is still read once, and the transaction at the exclusive bound is ignored.
            assert_eq!(input_txs.reads.load(Ordering::Relaxed), 4);
            assert_eq!(first_inputs.reads.load(Ordering::Relaxed), 2);
        }
    }
}
