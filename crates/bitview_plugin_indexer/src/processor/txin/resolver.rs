use brk_error::Result;

use std::collections::hash_map::Entry;

use brk_error::Error;
use brk_types::{
    OutPoint, OutputType, SigOps, TxIndex, TxOutIndex, Txid, TxidPrefix, TypeIndex, Vout,
};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use tracing::error;
use vecdb::unlikely;

use super::InputSource;
use crate::processor::{BlockProcessor, transaction::ComputedTx};

const PARALLEL_PARENT_READ_THRESHOLD: usize = 1_000;

#[derive(Default)]
pub struct InputResolver {
    parent_locations: FxHashMap<TxidPrefix, ParentLocation>,
    previous_parent_prefixes: Vec<TxidPrefix>,
    inputs: Vec<UnresolvedInput>,
    reads: ReadBatch,
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
        self.reads.resolve(
            processor,
            &self.previous_parent_prefixes,
            &self.inputs,
            processor.lengths.tx_index,
        )?;

        let tracks_executed_legacy_sigops = processor.tracks_executed_legacy_sigops();
        let reads = &self.reads;
        let inputs = &self.inputs;

        self.resolved.clear();
        self.resolved.resize(inputs.len(), InputSource::Coinbase);
        self.resolved.par_iter_mut().enumerate().try_for_each(
            |(input_index, resolved)| -> Result<()> {
                match inputs[input_index] {
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
                    UnresolvedInput::PreviousBlock { parent_index, vout } => {
                        let parent = reads.parent(parent_index);
                        let outpoint = OutPoint::new(parent.tx_index, vout);
                        let txout_index = parent.first_txout_index + vout;
                        let (output_type, type_index) = reads.output(input_index);

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

        Ok(&self.resolved)
    }

    pub fn prepare(
        &mut self,
        txs: &[ComputedTx<'_>],
        block_first_tx_index: TxIndex,
        block_first_txout_index: TxOutIndex,
    ) {
        self.parent_locations.clear();
        self.previous_parent_prefixes.clear();
        self.inputs.clear();

        self.parent_locations.reserve(txs.len());
        self.parent_locations.extend(
            txs.iter()
                .map(|tx| (tx.txid_prefix(), ParentLocation::SameBlock(tx.tx_index))),
        );

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

                let parent_index = match self.parent_locations.entry(txid_prefix) {
                    Entry::Occupied(entry) => match *entry.get() {
                        ParentLocation::SameBlock(tx_index) => {
                            let block_tx_index =
                                usize::from(tx_index) - usize::from(block_first_tx_index);
                            let tx = &txs[block_tx_index];
                            let txout_offset = tx.txout_offset(vout);
                            self.inputs.push(UnresolvedInput::SameBlock {
                                outpoint: OutPoint::new(tx_index, vout),
                                txout_offset,
                                txout_index: block_first_txout_index
                                    + TxOutIndex::from(txout_offset),
                            });
                            continue;
                        }
                        ParentLocation::Previous(parent_index) => parent_index.to_usize(),
                    },
                    Entry::Vacant(entry) => {
                        let parent_index = self.previous_parent_prefixes.len();
                        entry.insert(ParentLocation::Previous(PreviousParentIndex::new(
                            parent_index,
                        )));
                        self.previous_parent_prefixes.push(txid_prefix);
                        parent_index
                    }
                };

                self.inputs
                    .push(UnresolvedInput::PreviousBlock { parent_index, vout });
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum ParentLocation {
    SameBlock(TxIndex),
    Previous(PreviousParentIndex),
}

#[derive(Clone, Copy)]
pub struct PreviousParentIndex(u32);

impl PreviousParentIndex {
    pub fn new(index: usize) -> Self {
        debug_assert!(u32::try_from(index).is_ok());
        Self(index as u32)
    }

    #[inline]
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

const _: () = assert!(size_of::<ParentLocation>() == 8);

#[derive(Clone, Copy)]
pub struct ParentRead {
    tx_index: TxIndex,
    first_txout_index: TxOutIndex,
}

#[derive(Clone, Copy)]
pub struct OutputRead {
    input_index: usize,
    txout_index: TxOutIndex,
}

#[derive(Default)]
pub struct ReadBatch {
    parents: Vec<ParentRead>,
    outputs: Vec<OutputRead>,
    output_types: Vec<OutputType>,
    type_indices: Vec<TypeIndex>,
}

impl ReadBatch {
    pub fn resolve(
        &mut self,
        processor: &BlockProcessor<'_>,
        previous_parent_prefixes: &[TxidPrefix],
        inputs: &[UnresolvedInput],
        current_tx_index: TxIndex,
    ) -> Result<()> {
        self.resolve_parents(processor, previous_parent_prefixes, current_tx_index)?;
        self.prepare_outputs(inputs);
        self.read_outputs(processor)
    }

    pub fn resolve_parents(
        &mut self,
        processor: &BlockProcessor<'_>,
        previous_parent_prefixes: &[TxidPrefix],
        current_tx_index: TxIndex,
    ) -> Result<()> {
        let parallel_raw_reads = previous_parent_prefixes.len() >= PARALLEL_PARENT_READ_THRESHOLD;

        self.parents.clear();
        self.parents.resize(
            previous_parent_prefixes.len(),
            ParentRead {
                tx_index: TxIndex::default(),
                first_txout_index: TxOutIndex::default(),
            },
        );

        self.parents
            .par_iter_mut()
            .zip(previous_parent_prefixes.par_iter())
            .try_for_each(|read| {
                let (read, txid_prefix) = read;
                let store_result = processor.stores.tx_index(txid_prefix)?;

                let tx_index = match store_result {
                    Some(tx_index) if tx_index < current_tx_index => tx_index,
                    _ => {
                        error!(
                            "UnknownTxid: prefix={:?}, store_result={:?}, current_tx_index={:?}",
                            txid_prefix, store_result, current_tx_index
                        );
                        return Err(Error::UnknownTxid);
                    }
                };

                read.tx_index = tx_index;
                if parallel_raw_reads {
                    read.first_txout_index = processor
                        .vecs
                        .transactions
                        .first_txout_index
                        .get_append_only(tx_index, &processor.readers.tx_index_to_first_txout_index)
                        .ok_or(Error::Internal("Missing txout_index"))?;
                }
                Ok(())
            })?;

        if !parallel_raw_reads {
            for read in &mut self.parents {
                read.first_txout_index = processor
                    .vecs
                    .transactions
                    .first_txout_index
                    .get_append_only(
                        read.tx_index,
                        &processor.readers.tx_index_to_first_txout_index,
                    )
                    .ok_or(Error::Internal("Missing txout_index"))?;
            }
        }

        Ok(())
    }

    pub fn prepare_outputs(&mut self, inputs: &[UnresolvedInput]) {
        self.outputs.clear();
        self.outputs.reserve(inputs.len());

        for (input_index, input) in inputs.iter().enumerate() {
            if let UnresolvedInput::PreviousBlock { parent_index, vout } = *input {
                let parent = self.parent(parent_index);
                self.outputs.push(OutputRead {
                    input_index,
                    txout_index: parent.first_txout_index + vout,
                });
            }
        }

        self.output_types.clear();
        self.output_types.resize(inputs.len(), OutputType::Unknown);
        self.type_indices.clear();
        self.type_indices.resize(inputs.len(), TypeIndex::default());
    }

    pub fn read_outputs(&mut self, processor: &BlockProcessor<'_>) -> Result<()> {
        let outputs = &self.outputs;
        if outputs.is_empty() {
            return Ok(());
        }

        let output_types = &mut self.output_types;
        let type_indices = &mut self.type_indices;

        for read in outputs {
            output_types[read.input_index] = processor
                .vecs
                .outputs
                .output_type
                .get_append_only(
                    read.txout_index,
                    &processor.readers.txout_index_to_output_type,
                )
                .ok_or(Error::Internal("Missing output_type"))?;
            type_indices[read.input_index] = processor
                .vecs
                .outputs
                .type_index
                .get_append_only(
                    read.txout_index,
                    &processor.readers.txout_index_to_type_index,
                )
                .ok_or(Error::Internal("Missing type_index"))?;
        }
        Ok(())
    }

    pub fn parent(&self, original_index: usize) -> ParentRead {
        self.parents[original_index]
    }

    pub fn output(&self, input_index: usize) -> (OutputType, TypeIndex) {
        (
            self.output_types[input_index],
            self.type_indices[input_index],
        )
    }
}

#[derive(Clone, Copy)]
pub enum UnresolvedInput {
    Coinbase,
    PreviousBlock {
        parent_index: usize,
        vout: Vout,
    },
    SameBlock {
        outpoint: OutPoint,
        txout_offset: usize,
        txout_index: TxOutIndex,
    },
}
