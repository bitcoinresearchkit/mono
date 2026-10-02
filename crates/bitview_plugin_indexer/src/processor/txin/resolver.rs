use brk_error::{Error, Result};
use brk_types::{OutPoint, SigOps, TxIndex, TxOutIndex, Txid, TxidPrefix, Vout};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use tracing::error;
use vecdb::unlikely;

use super::{InputSource, parent_cache::ParentCache, parent_read::ParentRead};
use crate::processor::{BlockProcessor, transaction::ComputedTx};

#[derive(Default)]
pub struct InputResolver {
    block_parents: FxHashMap<TxidPrefix, TxIndex>,
    inputs: Vec<UnresolvedInput>,
    cache: ParentCache,
    resolved: Vec<InputSource>,
}

impl InputResolver {
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn resolve(
        &mut self,
        processor: &BlockProcessor<'_>,
        txs: &[ComputedTx<'_>],
    ) -> Result<&[InputSource]> {
        self.prepare(
            txs,
            processor.lengths.tx_index,
            processor.lengths.txout_index,
        );

        let tracks_executed_legacy_sigops = processor.tracks_executed_legacy_sigops();
        let inputs = &self.inputs;
        let cache = &self.cache;

        self.resolved.resize(inputs.len(), InputSource::Coinbase);
        // Resolve each input in one pass so parent and output reads can overlap.
        self.resolved.par_iter_mut().zip(inputs).try_for_each(
            |(resolved, input)| -> Result<()> {
                match *input {
                    UnresolvedInput::Coinbase => {
                        *resolved = InputSource::Coinbase;
                        Ok(())
                    }
                    UnresolvedInput::SameBlock {
                        outpoint,
                        txout_offset,
                        txout_index,
                    } => {
                        *resolved = InputSource::SameBlock {
                            outpoint,
                            txout_offset,
                            txout_index,
                        };
                        Ok(())
                    }
                    UnresolvedInput::PreviousBlock { prefix, vout } => {
                        let parent = Self::read_parent(cache, processor, prefix)?;
                        let outpoint = OutPoint::new(parent.tx_index, vout);
                        let txout_index = parent.first_txout_index + vout;
                        let output_type = processor
                            .vecs
                            .outputs
                            .output_type
                            .get_append_only(
                                txout_index,
                                &processor.readers.txout_index_to_output_type,
                            )
                            .ok_or(Error::Internal("Missing output_type"))?;
                        let type_index = processor
                            .vecs
                            .outputs
                            .type_index
                            .get_append_only(
                                txout_index,
                                &processor.readers.txout_index_to_type_index,
                            )
                            .ok_or(Error::Internal("Missing type_index"))?;

                        let legacy_sigops = if tracks_executed_legacy_sigops {
                            processor
                                .vecs
                                .scripts
                                .legacy_sigops(output_type, type_index, &processor.readers.scripts)
                                .ok_or(Error::Internal("Missing legacy_sigops"))?
                        } else {
                            SigOps::ZERO
                        };

                        *resolved = InputSource::PreviousBlock {
                            outpoint,
                            txout_index,
                            output_type,
                            legacy_sigops,
                            type_index,
                        };
                        Ok(())
                    }
                }
            },
        )?;

        // Recent parents already have a cheap lookup in the store's pending map.
        let stored_tx_count = processor.readers.tx_index_to_first_txout_index.stored_len();
        for (input, resolved) in self.inputs.iter().zip(&self.resolved) {
            if let (
                UnresolvedInput::PreviousBlock { prefix, vout },
                InputSource::PreviousBlock {
                    outpoint,
                    txout_index,
                    ..
                },
            ) = (input, resolved)
                && usize::from(outpoint.tx_index()) < stored_tx_count
            {
                self.cache.insert(
                    *prefix,
                    ParentRead {
                        tx_index: outpoint.tx_index(),
                        first_txout_index: TxOutIndex::new(
                            u64::from(*txout_index) - u64::from(*vout),
                        ),
                    },
                );
            }
        }
        Ok(&self.resolved)
    }

    fn prepare(
        &mut self,
        txs: &[ComputedTx<'_>],
        block_first_tx_index: TxIndex,
        block_first_txout_index: TxOutIndex,
    ) {
        self.block_parents.clear();
        self.inputs.clear();

        self.block_parents.reserve(txs.len());
        self.block_parents.extend(txs.iter().map(|tx| {
            let prefix = tx.txid_prefix();
            // A newly indexed transaction may replace a historical prefix.
            self.cache.invalidate(prefix);
            (prefix, tx.tx_index)
        }));

        let total_inputs = txs.iter().map(|tx| tx.tx.input.len()).sum();
        self.inputs.reserve(total_inputs);

        for tx in txs {
            for txin in &tx.tx.input {
                let previous_output = &txin.previous_output;
                if unlikely(previous_output.is_null()) {
                    self.inputs.push(UnresolvedInput::Coinbase);
                    continue;
                }

                let txid = *<&Txid>::from(&previous_output.txid);
                let txid_prefix = TxidPrefix::from(&txid);
                let vout = Vout::from(previous_output.vout);

                if let Some(tx_index) = self.block_parents.get(&txid_prefix).copied() {
                    let block_tx_index = usize::from(tx_index) - usize::from(block_first_tx_index);
                    let tx = &txs[block_tx_index];
                    let txout_offset = tx.txout_offset(vout);
                    self.inputs.push(UnresolvedInput::SameBlock {
                        outpoint: OutPoint::new(tx_index, vout),
                        txout_offset,
                        txout_index: block_first_txout_index + TxOutIndex::from(txout_offset),
                    });
                } else {
                    self.inputs.push(UnresolvedInput::PreviousBlock {
                        prefix: txid_prefix,
                        vout,
                    });
                }
            }
        }
    }

    fn read_parent(
        cache: &ParentCache,
        processor: &BlockProcessor<'_>,
        prefix: TxidPrefix,
    ) -> Result<ParentRead> {
        let current_tx_index = processor.lengths.tx_index;
        if let Some(cached) = cache.get(prefix)
            && cached.tx_index < current_tx_index
        {
            return Ok(cached);
        }
        let store_result = processor.stores.tx_index(&prefix)?;
        let tx_index = match store_result {
            Some(tx_index) if tx_index < current_tx_index => tx_index,
            _ => {
                error!(
                    "UnknownTxid: prefix={:?}, store_result={:?}, current_tx_index={:?}",
                    prefix, store_result, current_tx_index
                );
                return Err(Error::UnknownTxid);
            }
        };
        let first_txout_index = processor
            .vecs
            .transactions
            .first_txout_index
            .get_append_only(tx_index, &processor.readers.tx_index_to_first_txout_index)
            .ok_or(Error::Internal("Missing txout_index"))?;
        Ok(ParentRead {
            tx_index,
            first_txout_index,
        })
    }
}

#[derive(Clone, Copy)]
enum UnresolvedInput {
    Coinbase,
    PreviousBlock {
        prefix: TxidPrefix,
        vout: Vout,
    },
    SameBlock {
        outpoint: OutPoint,
        txout_offset: usize,
        txout_index: TxOutIndex,
    },
}

#[cfg(test)]
#[path = "resolver_tests.rs"]
mod tests;
