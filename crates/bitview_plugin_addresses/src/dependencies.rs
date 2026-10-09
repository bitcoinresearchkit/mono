use std::sync::mpsc::Receiver;

use bitview_cohort::ByAddrType;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_primitives::TxInIndex;
use brk_types::{Height, Sats, Version};
use vecdb::{AnyVec, PcoVec, ReadableBoxedVec};

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub mappings: &'a MappingsVecs,
    pub input_values: &'a PcoVec<TxInIndex, Sats>,
    /// Per-type supply, sent once utxos has computed it: only the shares and average
    /// balances derived after the block loop read it, so the loop runs alongside UTXOs.
    pub type_supply: Receiver<ByAddrType<ReadableBoxedVec<Height, Sats>>>,
    pub price: &'a PriceVecs,
}

impl Dependencies<'_> {
    pub(crate) fn version(&self) -> Version {
        let v = self.indexer.vecs();
        Version::ONE
            + Version::combine_all([
                self.price.spot.cents.height.version(),
                self.input_values.version(),
                v.transactions.first_tx_index.version(),
                v.outputs.first_txout_index.version(),
                v.inputs.first_txin_index.version(),
                self.mappings.tx_index.output_count.version(),
                self.mappings.tx_index.input_count.version(),
                v.outputs.value.version(),
                v.outputs.output_type.version(),
                v.outputs.type_index.version(),
                v.inputs.txout_index.version(),
                v.inputs.output_type.version(),
                v.inputs.type_index.version(),
                v.addrs.p2pk65.first_index.version(),
                v.addrs.p2pk33.first_index.version(),
                v.addrs.p2pkh.first_index.version(),
                v.addrs.p2sh.first_index.version(),
                v.addrs.p2wpkh.first_index.version(),
                v.addrs.p2wsh.first_index.version(),
                v.addrs.p2tr.first_index.version(),
                v.addrs.p2a.first_index.version(),
            ])
    }
}
