mod resolver;
mod source;

pub use resolver::InputResolver;
pub use source::InputSource;

use brk_error::Result;

use bitcoin::Transaction;
use brk_cohort::ByAddrType;
use brk_store::Store;
use brk_types::{
    AddrIndexOutPoint, AddrIndexTxIndex, OutPoint, OutputType, TxInIndex, TxIndex, TxOutIndex,
    TypeIndex, Unit, Vin,
};
use vecdb::{PcoVec, WritableVec, unlikely};

use super::{BlockProcessor, transaction::ComputedTx, txout::ProcessedOutput};
use crate::InputsVecs;

impl<'a> BlockProcessor<'a> {
    pub fn process_inputs<'b>(
        &self,
        txs: &[ComputedTx],
        resolver: &'b mut InputResolver,
    ) -> Result<&'b [InputSource]> {
        resolver.resolve(self, txs)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn finalize_inputs(
    transactions: &[Transaction],
    base_tx_index: TxIndex,
    base_txin_index: TxInIndex,
    first_txin_index: &mut PcoVec<TxIndex, TxInIndex>,
    inputs: &mut InputsVecs,
    addr_tx_index_stores: &mut ByAddrType<Store<AddrIndexTxIndex, Unit>>,
    addr_outpoint_stores: &mut ByAddrType<Store<AddrIndexOutPoint, Unit>>,
    txins: &[InputSource],
    txouts: &[ProcessedOutput],
) {
    let mut input_offset = 0;
    for (block_tx_index, tx) in transactions.iter().enumerate() {
        let tx_index = base_tx_index + TxIndex::from(block_tx_index);
        let next_input_offset = input_offset + tx.input.len();

        for (vin, input_source) in txins[input_offset..next_input_offset].iter().enumerate() {
            let block_txin_index = input_offset + vin;
            let txin_index = base_txin_index + TxInIndex::from(block_txin_index);
            let vin = Vin::from(vin);
            let (outpoint, txout_index, output_type, type_index) = match input_source {
                InputSource::PreviousBlock {
                    outpoint,
                    txout_index,
                    output_type,
                    legacy_sigops: _,
                    type_index,
                } => (*outpoint, *txout_index, *output_type, *type_index),
                InputSource::Coinbase => (
                    OutPoint::COINBASE,
                    TxOutIndex::COINBASE,
                    OutputType::Unknown,
                    TypeIndex::COINBASE,
                ),
                InputSource::SameBlock {
                    outpoint,
                    txout_offset,
                    txout_index,
                } => {
                    let output = &txouts[*txout_offset];
                    (
                        *outpoint,
                        *txout_index,
                        output.output_type,
                        output.resolved_type_index(),
                    )
                }
            };

            if vin.is_zero() {
                first_txin_index.debug_checked_push(tx_index, txin_index);
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

            if unlikely(!output_type.is_addr()) {
                continue;
            }
            let addr_type = output_type;
            let addr_index = type_index;

            addr_tx_index_stores
                .get_mut_unwrap(addr_type)
                .insert(AddrIndexTxIndex::from((addr_index, tx_index)), Unit);

            addr_outpoint_stores
                .get_mut_unwrap(addr_type)
                .remove(AddrIndexOutPoint::from((addr_index, outpoint)));
        }

        input_offset = next_input_offset;
    }
}
