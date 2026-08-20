mod address;
mod op_return;
mod processed;

pub use address::BlockAddresses;
pub use processed::ProcessedOutput;
pub use processed::ProcessedOutputData;

use brk_error::Result;

use bitcoin::{Script, Transaction, constants::WITNESS_SCALE_FACTOR};
use bitview_cohort::ByAddrType;
use brk_store::Store;
use brk_types::{
    AddrBytes, AddrHash, AddrIndexOutPoint, AddrIndexTxIndex, OutPoint, OutputType, Sats, SigOps,
    TxIndex, TxOutIndex, TypeIndex, Unit, Vout,
};
use rayon::prelude::*;
use vecdb::{BytesVec, WritableVec, likely};

use super::BlockProcessor;
use crate::{AddrsVecs, Lengths, OpReturnVecs, OutputsVecs, ScriptsVecs};

impl<'a> BlockProcessor<'a> {
    pub fn process_outputs(&self, addresses: &mut BlockAddresses) -> Result<Vec<ProcessedOutput>> {
        let total_outputs: usize = self.block.txdata.iter().map(|tx| tx.output.len()).sum();
        let mut items = Vec::with_capacity(total_outputs);
        for tx in &self.block.txdata {
            items.extend(&tx.output);
        }

        let outputs = items
            .into_par_iter()
            .map(|txout| {
                let script = &txout.script_pubkey;
                let output_type = OutputType::from(script);
                let legacy_sigops = executed_legacy_sigops_for_output(output_type, script);
                let data = if output_type.is_addr() {
                    ProcessedOutputData::Address(
                        AddrHash::from_script(script, output_type).unwrap(),
                    )
                } else if likely(output_type == OutputType::OpReturn) {
                    ProcessedOutputData::OpReturn(op_return::analyze(script))
                } else {
                    ProcessedOutputData::None
                };

                ProcessedOutput {
                    output_type,
                    legacy_sigops,
                    data,
                }
            })
            .collect::<Vec<_>>();

        addresses.resolve(self, &outputs)?;

        Ok(outputs)
    }
}

pub fn executed_legacy_sigops_for_output(
    output_type: OutputType,
    script_pubkey: &Script,
) -> SigOps {
    SigOps::from(
        match output_type {
            OutputType::P2PKH | OutputType::P2PK33 | OutputType::P2PK65 => 1,
            OutputType::P2MS | OutputType::Unknown => script_pubkey.count_sigops(),
            _ => 0,
        }
        .saturating_mul(WITNESS_SCALE_FACTOR),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn finalize_outputs(
    transactions: &[Transaction],
    base_tx_index: TxIndex,
    lengths: &mut Lengths,
    first_txout_index: &mut BytesVec<TxIndex, TxOutIndex>,
    outputs: &mut OutputsVecs,
    addrs: &mut AddrsVecs,
    scripts: &mut ScriptsVecs,
    op_return_vecs: &mut OpReturnVecs,
    addr_hash_stores: &mut ByAddrType<Store<AddrHash, TypeIndex>>,
    addr_tx_index_stores: &mut ByAddrType<Store<AddrIndexTxIndex, Unit>>,
    addr_outpoint_stores: &mut ByAddrType<Store<AddrIndexOutPoint, Unit>>,
    txouts: &mut [ProcessedOutput],
    addresses: &mut BlockAddresses,
) {
    let base_txout_index = lengths.txout_index;
    let mut output_offset = 0;
    for (block_tx_index, tx) in transactions.iter().enumerate() {
        let tx_index = base_tx_index + TxIndex::from(block_tx_index);
        let next_output_offset = output_offset + tx.output.len();

        for (vout, (txout, processed)) in tx
            .output
            .iter()
            .zip(&mut txouts[output_offset..next_output_offset])
            .enumerate()
        {
            let block_txout_index = output_offset + vout;
            let txout_index = base_txout_index + TxOutIndex::from(block_txout_index);
            let vout = Vout::from(vout);
            let output_type = processed.output_type;
            let legacy_sigops = processed.legacy_sigops;
            let data = processed.data;
            let sats = Sats::from(txout.value);

            if vout.is_zero() {
                first_txout_index.debug_checked_push(tx_index, txout_index);
            }

            let type_index = match data {
                ProcessedOutputData::Address(addr_hash) => {
                    let addr_type = output_type;
                    let type_index = addresses.index_mut(addr_type, &addr_hash);

                    if let Some(ti) = *type_index {
                        ti
                    } else {
                        let ti = lengths.increment_addr_index(addr_type);

                        *type_index = Some(ti);
                        addr_hash_stores
                            .get_mut_unwrap(addr_type)
                            .insert(addr_hash, ti);
                        let addr_bytes =
                            AddrBytes::try_from((&txout.script_pubkey, addr_type)).unwrap();
                        addrs.push_bytes_if_needed(ti, addr_bytes);

                        ti
                    }
                }
                ProcessedOutputData::OpReturn(op_return) => {
                    let op_return_index = lengths.op_return_index;

                    op_return_vecs
                        .to_tx_index
                        .debug_checked_push(lengths.op_return_index, tx_index);
                    op_return_vecs
                        .kind
                        .debug_checked_push(op_return_index, op_return.kind);
                    op_return_vecs
                        .post_op_return_bytes
                        .debug_checked_push(op_return_index, op_return.post_op_return_bytes);
                    lengths.op_return_index.copy_then_increment()
                }
                ProcessedOutputData::None => match output_type {
                    OutputType::P2MS => {
                        let index = lengths.p2ms_output_index;
                        scripts.p2ms.to_tx_index.debug_checked_push(index, tx_index);
                        scripts
                            .p2ms
                            .legacy_sigops
                            .debug_checked_push(index, legacy_sigops);
                        lengths.p2ms_output_index.copy_then_increment()
                    }
                    OutputType::Empty => {
                        scripts
                            .empty
                            .to_tx_index
                            .debug_checked_push(lengths.empty_output_index, tx_index);
                        lengths.empty_output_index.copy_then_increment()
                    }
                    OutputType::Unknown => {
                        let index = lengths.unknown_output_index;
                        scripts
                            .unknown
                            .to_tx_index
                            .debug_checked_push(index, tx_index);
                        scripts
                            .unknown
                            .legacy_sigops
                            .debug_checked_push(index, legacy_sigops);
                        lengths.unknown_output_index.copy_then_increment()
                    }
                    _ => unreachable!(),
                },
                ProcessedOutputData::Resolved(_) => unreachable!(),
            };

            outputs.value.debug_checked_push(txout_index, sats);
            outputs
                .output_type
                .debug_checked_push(txout_index, output_type);
            outputs
                .type_index
                .debug_checked_push(txout_index, type_index);
            processed.data = ProcessedOutputData::Resolved(type_index);

            if likely(output_type.is_addr()) {
                let addr_type = output_type;
                let addr_index = type_index;

                addr_tx_index_stores
                    .get_mut_unwrap(addr_type)
                    .insert(AddrIndexTxIndex::from((addr_index, tx_index)), Unit);

                addr_outpoint_stores.get_mut_unwrap(addr_type).insert(
                    AddrIndexOutPoint::from((addr_index, OutPoint::new(tx_index, vout))),
                    Unit,
                );
            }
        }

        output_offset = next_output_offset;
    }
}
