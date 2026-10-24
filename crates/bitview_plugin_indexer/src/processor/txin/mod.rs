pub mod resolver;
pub mod source;

pub use resolver::InputResolver;
pub use source::InputSource;

use bitview_primitives::{
    AddrIndexOutPoint, AddrIndexTxIndex, ByAddrType, TxInIndex, TxOutIndex, TypeIndex,
};
use brk_store::Store;
use brk_types::{OutPoint, OutputType, Sats, TxIndex, Txid, TxidPrefix, Unit};
use vecdb::{PcoVec, WritableVec, unlikely};

use super::{transaction::ComputedTx, txout::ProcessedOutput};
use crate::{
    InputsVecs,
    stores::{UtxoKey, Utxos},
};

#[allow(clippy::too_many_arguments)]
pub fn finalize_inputs(
    txs: &[ComputedTx],
    base_txin_index: TxInIndex,
    first_txin_index: &mut PcoVec<TxIndex, TxInIndex>,
    inputs: &mut InputsVecs,
    addr_tx_index_stores: &mut ByAddrType<Store<AddrIndexTxIndex, Unit>>,
    addr_outpoint_stores: &mut ByAddrType<Store<AddrIndexOutPoint, Unit>>,
    utxos: &mut Utxos,
    txins: &[InputSource],
    txouts: &[ProcessedOutput],
) {
    for ct in txs {
        let tx_index = ct.tx_index;
        let first_txin = base_txin_index + TxInIndex::from(ct.input_offset);
        // Every transaction has an input.
        first_txin_index.debug_checked_push(tx_index, first_txin);

        for (vin, (input_source, txin)) in ct.inputs(txins).iter().zip(&ct.tx.input).enumerate() {
            let txin_index = first_txin + TxInIndex::from(vin);
            let (outpoint, txout_index, output_type, type_index, value) = match input_source {
                InputSource::PreviousBlock {
                    outpoint,
                    txout_index,
                    output_type,
                    legacy_sigop_cost: _,
                    type_index,
                    value,
                } => (*outpoint, *txout_index, *output_type, *type_index, *value),
                InputSource::Coinbase => (
                    OutPoint::COINBASE,
                    TxOutIndex::COINBASE,
                    OutputType::UnknownScript,
                    TypeIndex::COINBASE,
                    Sats::MAX,
                ),
                InputSource::SameBlock {
                    outpoint,
                    txout_offset,
                    txout_index,
                    value,
                } => {
                    let output = &txouts[*txout_offset];
                    (
                        *outpoint,
                        *txout_index,
                        output.output_type,
                        output.resolved_type_index(),
                        *value,
                    )
                }
            };

            if !matches!(input_source, InputSource::Coinbase) {
                let previous_output = &txin.previous_output;
                let txid = <&Txid>::from(&previous_output.txid);
                utxos.remove(
                    UtxoKey::new(TxidPrefix::from(txid), previous_output.vout),
                    value,
                );
            }

            inputs.tx_index.debug_checked_push(txin_index, tx_index);
            inputs.outpoint.debug_checked_push(txin_index, outpoint);
            inputs
                .txout_index
                .debug_checked_push(txin_index, txout_index);
            inputs
                .output_type
                .debug_checked_push(txin_index, output_type);
            inputs.type_index.debug_checked_push(txin_index, type_index);
            inputs.value.debug_checked_push(txin_index, value);

            if unlikely(!output_type.is_addr()) {
                continue;
            }
            addr_tx_index_stores
                .get_mut_unwrap(output_type)
                .insert(AddrIndexTxIndex::from((type_index, tx_index)), Unit);

            addr_outpoint_stores
                .get_mut_unwrap(output_type)
                .remove(AddrIndexOutPoint::from((type_index, outpoint)));
        }
    }
}
