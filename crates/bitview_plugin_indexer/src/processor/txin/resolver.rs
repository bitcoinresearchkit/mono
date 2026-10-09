use bitview_primitives::TxOutIndex;
use brk_error::{Error, Result};
use brk_types::{OutPoint, SigOps, TxIndex, Txid, TxidPrefix, Vout};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use tracing::error;
use vecdb::unlikely;

use super::InputSource;
use crate::{
    processor::{BlockProcessor, transaction::ComputedTx},
    stores::UtxoKey,
};

#[derive(Default)]
pub struct InputResolver {
    block_parents: FxHashMap<TxidPrefix, TxIndex>,
    inputs: Vec<UnresolvedInput>,
    resolved: Vec<InputSource>,
}

impl InputResolver {
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

        self.resolved.resize(inputs.len(), InputSource::Coinbase);
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
                        let key = UtxoKey::new(prefix, vout);
                        let Some(utxo) = processor.stores.utxo(&key)? else {
                            error!("Spent output not in the UTXO store: prefix={prefix:?}, vout={vout:?}");
                            return Err(Error::Internal("Spent output not in the UTXO store"));
                        };
                        let (output_type, type_index) = (utxo.output_type(), utxo.type_index());

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
                            outpoint: OutPoint::new(utxo.tx_index(), vout),
                            txout_index: utxo.txout_index(),
                            output_type,
                            legacy_sigops,
                            type_index,
                        };
                        Ok(())
                    }
                }
            },
        )?;

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
        self.block_parents
            .extend(txs.iter().map(|tx| (tx.txid_prefix(), tx.tx_index)));

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
