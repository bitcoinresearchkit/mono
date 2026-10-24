pub mod address;
pub mod op_return;
pub mod processed;

pub use address::BlockAddresses;
pub use processed::{ProcessedOutput, ProcessedOutputData, is_single_sig};

use bitcoin::constants::WITNESS_SCALE_FACTOR;
use bitview_primitives::{
    AddrHash, AddrIndexOutPoint, AddrIndexTxIndex, ByAddrType, Index40, TxOutIndex, TypeIndex,
};
use brk_error::Result;
use brk_store::Store;
use brk_types::{
    AddressBytes, OutPoint, OutputType, Sats, SigOpCount, SigOps32, TxIndex, Unit, Vout,
};
use vecdb::{BytesVec, WritableVec, likely};

use super::{BlockProcessor, transaction::ComputedTx};
use crate::{
    AddrsVecs, Lengths, OpReturnVecs, OutputsVecs, ScriptsVecs,
    constants::overwritten_tx_index,
    stores::{UtxoKey, Utxos},
};

/// Consensus's script size limit: Core treats a larger locking script as unspendable.
const MAX_SCRIPT_SIZE: usize = 10_000;

impl<'a> BlockProcessor<'a> {
    /// Classifies the block's outputs into `outputs`, cleared first.
    pub fn process_outputs(
        &self,
        txs: &[ComputedTx],
        addresses: &mut BlockAddresses,
        outputs: &mut Vec<ProcessedOutput>,
    ) -> Result<()> {
        outputs.clear();
        // Start address lookups without a temporary pointer list or another parallel barrier.
        outputs.extend(txs.iter().flat_map(|ct| &ct.tx.output).map(|txout| {
            let script = &txout.script_pubkey;
            let output_type = OutputType::from(script);
            // Core leaves larger scripts out of its UTXO set as unspendable; none exist,
            // which keeps the UTXO set's totals equal to Core's.
            debug_assert!(
                output_type == OutputType::OpReturn || script.len() <= MAX_SCRIPT_SIZE,
                "unspendable {}-byte script outside Core's UTXO set",
                script.len()
            );
            // With the accurate sigop count a spend executes; the legacy count, for the
            // transaction's total, stays in the data (see `script_legacy_sigops`). A P2MS script
            // can hold extra signature opcodes before its CHECKMULTISIG and an unknown one
            // anything, so both are counted.
            let (data, executed) = if output_type.is_addr() {
                let hash = AddrHash::from_script(script, output_type).unwrap();
                let single_sig = usize::from(is_single_sig(output_type));
                (ProcessedOutputData::Address(hash), single_sig)
            } else if likely(output_type == OutputType::OpReturn) {
                (ProcessedOutputData::OpReturn(op_return::analyze(script)), 0)
            } else if output_type == OutputType::EmptyScript {
                (ProcessedOutputData::None, 0)
            } else {
                let count = SigOpCount::of(script);
                let legacy_sigops = count.legacy as u32;
                (
                    ProcessedOutputData::Script { legacy_sigops },
                    count.accurate,
                )
            };

            ProcessedOutput {
                output_type,
                legacy_sigop_cost: SigOps32::from(executed * WITNESS_SCALE_FACTOR),
                data,
            }
        }));

        addresses.resolve(self, txs, outputs)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn finalize_outputs(
    txs: &[ComputedTx],
    lengths: &mut Lengths,
    first_txout_index: &mut BytesVec<TxIndex, Index40<TxOutIndex>>,
    outputs: &mut OutputsVecs,
    addrs: &mut AddrsVecs,
    scripts: &mut ScriptsVecs,
    op_return_vecs: &mut OpReturnVecs,
    addr_hash_stores: &mut ByAddrType<Store<AddrHash, TypeIndex>>,
    addr_tx_index_stores: &mut ByAddrType<Store<AddrIndexTxIndex, Unit>>,
    addr_outpoint_stores: &mut ByAddrType<Store<AddrIndexOutPoint, Unit>>,
    utxos: &mut Utxos,
    txouts: &mut [ProcessedOutput],
    addresses: &mut BlockAddresses,
) {
    let base_txout_index = lengths.txout_index;
    for ct in txs {
        let tx_index = ct.tx_index;
        let txid_prefix = ct.txid_prefix();
        let overwritten = overwritten_tx_index(tx_index);
        let first_txout = base_txout_index + TxOutIndex::from(ct.output_offset);
        // Every transaction has an output.
        first_txout_index.debug_checked_push(tx_index, Index40::new(first_txout));

        for (vout, (txout, processed)) in
            ct.tx.output.iter().zip(ct.outputs_mut(txouts)).enumerate()
        {
            let txout_index = first_txout + TxOutIndex::from(vout);
            let vout_index = vout as u32;
            let vout = Vout::from(vout);
            let output_type = processed.output_type;
            let legacy_sigop_cost = processed.legacy_sigop_cost;
            let data = processed.data;
            let sats = Sats::from(txout.value);

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
                            AddressBytes::try_from((&txout.script_pubkey, addr_type)).unwrap();
                        addrs.push_bytes(ti, addr_bytes);

                        ti
                    }
                }
                ProcessedOutputData::OpReturn(op_return) => {
                    let op_return_index = lengths.op_return_index;

                    op_return_vecs
                        .to_tx_index
                        .debug_checked_push(op_return_index, tx_index);
                    op_return_vecs
                        .kind
                        .debug_checked_push(op_return_index, op_return.kind);
                    op_return_vecs
                        .post_op_return_bytes
                        .debug_checked_push(op_return_index, op_return.post_op_return_bytes);
                    lengths.op_return_index.copy_then_increment()
                }
                ProcessedOutputData::None | ProcessedOutputData::Script { .. } => match output_type
                {
                    OutputType::P2MS => {
                        let index = lengths.p2ms_output_index;
                        scripts.p2ms.to_tx_index.debug_checked_push(index, tx_index);
                        scripts
                            .p2ms
                            .legacy_sigop_cost
                            .debug_checked_push(index, legacy_sigop_cost);
                        lengths.p2ms_output_index.copy_then_increment()
                    }
                    OutputType::EmptyScript => {
                        scripts
                            .empty_script
                            .to_tx_index
                            .debug_checked_push(lengths.empty_script_output_index, tx_index);
                        lengths.empty_script_output_index.copy_then_increment()
                    }
                    OutputType::UnknownScript => {
                        let index = lengths.unknown_script_output_index;
                        scripts
                            .unknown_script
                            .to_tx_index
                            .debug_checked_push(index, tx_index);
                        scripts
                            .unknown_script
                            .legacy_sigop_cost
                            .debug_checked_push(index, legacy_sigop_cost);
                        lengths.unknown_script_output_index.copy_then_increment()
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

            // OP_RETURN outputs can never be spent. A BIP30 duplicate coinbase (two, in 2010)
            // replaces its original's entry, as in Bitcoin Core.
            if !output_type.is_unspendable() {
                utxos.insert(
                    UtxoKey::new(txid_prefix, vout_index),
                    tx_index,
                    txout_index,
                    output_type,
                    type_index,
                    sats,
                );
            }

            if likely(output_type.is_addr()) {
                let addr_type = output_type;
                let addr_index = type_index;

                addr_tx_index_stores
                    .get_mut_unwrap(addr_type)
                    .insert(AddrIndexTxIndex::from((addr_index, tx_index)), Unit);

                let outpoints = addr_outpoint_stores.get_mut_unwrap(addr_type);
                outpoints.insert(
                    AddrIndexOutPoint::from((addr_index, OutPoint::new(tx_index, vout))),
                    Unit,
                );
                // The overwritten output left the set unspent: drop it from its address.
                if let Some(overwritten) = overwritten {
                    outpoints.remove(AddrIndexOutPoint::from((
                        addr_index,
                        OutPoint::new(overwritten, vout),
                    )));
                }
            }
        }
    }
}
