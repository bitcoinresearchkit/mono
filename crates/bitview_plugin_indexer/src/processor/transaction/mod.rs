mod analysis;
mod computed;

use brk_error::Result;

use brk_error::Error;
use brk_store::Store;
use brk_types::{Height, StoredBool, TxIndex, TxVersion, Txid, TxidPrefix};
use rayon::prelude::*;
use tracing::error;
use vecdb::{AnyVec, WritableVec, likely, unlikely};

use crate::{
    TransactionCounts, TransactionFeaturesVecs, TxMetadataVecs,
    constants::DUPLICATE_TXIDS,
    stores::{IndexerStores as _, TransactionStoresMut},
};

pub use computed::ComputedTx;

use self::analysis::TransactionAnalysis;
use super::{
    BlockProcessor,
    txin::{self, InputSource},
    txout::{self, BlockAddresses, ProcessedOutput},
};

impl<'a> BlockProcessor<'a> {
    pub fn compute_txids(&self) -> Result<Vec<ComputedTx<'a>>> {
        let will_check_collisions = self.check_collisions;
        let base_tx_index = self.lengths.tx_index;

        let mut txs = self
            .block
            .txdata
            .par_iter()
            .enumerate()
            .map(|(index, tx)| {
                let (btc_txid, base_size, total_size) = self.block.compute_tx_id_and_sizes(index);
                let txid = Txid::from(btc_txid);
                let tx_index = base_tx_index + TxIndex::from(index);

                let insert_txid_prefix = if likely(!will_check_collisions) {
                    true
                } else {
                    let txid_prefix = TxidPrefix::from(&txid);
                    let prev_tx_index = self.stores.tx_index(&txid_prefix)?;

                    if let Some(prev_tx_index) = prev_tx_index {
                        self.validate_txid_collision(tx_index, prev_tx_index)?;
                        false
                    } else {
                        true
                    }
                };

                Ok(ComputedTx::new(
                    tx_index,
                    tx,
                    txid,
                    insert_txid_prefix,
                    base_size,
                    total_size,
                ))
            })
            .collect::<Result<Vec<_>>>()?;

        ComputedTx::set_block_offsets(&mut txs);
        Ok(txs)
    }

    pub fn validate_txid_collision(&self, tx_index: TxIndex, prev_tx_index: TxIndex) -> Result<()> {
        if tx_index == prev_tx_index {
            return Ok(());
        }

        let len = self.vecs.transactions.txid.len();
        let prev_txid = self
            .vecs
            .transactions
            .txid
            .get_append_only(prev_tx_index, &self.readers.txid)
            .ok_or(Error::Internal("Missing txid for tx_index"))
            .inspect_err(|_| {
                error!(?tx_index, len, "Missing txid for tx_index");
            })?;

        if unlikely(!DUPLICATE_TXIDS.contains(&prev_txid)) {
            error!(
                height = ?self.height,
                ?tx_index,
                ?prev_txid,
                ?prev_tx_index,
                "Unexpected TXID collision"
            );
            return Err(Error::Internal("Unexpected TXID collision"));
        }

        Ok(())
    }

    /// Analyzes transactions, then finalizes outputs/inputs in parallel with metadata storage.
    #[allow(clippy::too_many_arguments)]
    pub fn analyze_and_finalize_transactions(
        &mut self,
        txs: Vec<ComputedTx>,
        mut txouts: Vec<ProcessedOutput>,
        txins: &[InputSource],
        addresses: &mut BlockAddresses,
    ) {
        let transaction_analyses = self.analyze_transactions(&txs, txins, &txouts);
        let lengths = &mut *self.lengths;
        let base_tx_index = lengths.tx_index;
        let base_txin_index = lengths.txin_index;
        let transactions = &self.block.txdata;

        // Split transactions vecs: finalize needs first_txout_index/first_txin_index, metadata needs the rest
        let (first_txout_index, first_txin_index, mut tx_metadata) =
            self.vecs.transactions.split_for_finalize();

        let outputs = &mut self.vecs.outputs;
        let inputs = &mut self.vecs.inputs;
        let addrs = &mut self.vecs.addrs;
        let scripts = &mut self.vecs.scripts;
        let op_return = &mut self.vecs.op_return;
        let transaction_features = &mut self.vecs.transaction_features;
        let height = self.height;

        let TransactionStoresMut {
            addr_hashes,
            addr_tx_indexes,
            addr_unspent_outpoints,
            txid_prefixes,
        } = self.stores.transaction_stores_mut();

        rayon::join(
            || {
                txout::finalize_outputs(
                    transactions,
                    base_tx_index,
                    lengths,
                    first_txout_index,
                    outputs,
                    addrs,
                    scripts,
                    op_return,
                    addr_hashes,
                    addr_tx_indexes,
                    addr_unspent_outpoints,
                    &mut txouts,
                    addresses,
                );
                txin::finalize_inputs(
                    transactions,
                    base_tx_index,
                    base_txin_index,
                    first_txin_index,
                    inputs,
                    addr_tx_indexes,
                    addr_unspent_outpoints,
                    txins,
                    &txouts,
                );
            },
            || {
                store_tx_metadata(
                    height,
                    txs,
                    transaction_analyses,
                    txid_prefixes,
                    &mut tx_metadata,
                    transaction_features,
                )
            },
        );
    }
}

pub fn store_tx_metadata(
    height: Height,
    txs: Vec<ComputedTx>,
    transaction_analyses: Vec<TransactionAnalysis>,
    store: &mut Store<TxidPrefix, TxIndex>,
    md: &mut TxMetadataVecs<'_>,
    features: &mut TransactionFeaturesVecs,
) {
    debug_assert_eq!(txs.len(), transaction_analyses.len());
    let mut counts = TransactionCounts::default();
    for (ct, analysis) in txs.into_iter().zip(transaction_analyses) {
        if likely(ct.insert_txid_prefix) {
            store.insert(ct.txid_prefix(), ct.tx_index);
        }
        let tx_version = TxVersion::from(ct.tx.version);
        md.tx_version.debug_checked_push(ct.tx_index, tx_version);
        md.txid.debug_checked_push(ct.tx_index, ct.txid);
        md.raw_locktime
            .debug_checked_push(ct.tx_index, ct.tx.lock_time.into());
        md.weight
            .debug_checked_push(ct.tx_index, ct.weight().into());
        md.total_size
            .debug_checked_push(ct.tx_index, ct.total_size.into());
        md.total_sigop_cost
            .debug_checked_push(ct.tx_index, analysis.total_sigop_cost);
        md.is_explicitly_rbf
            .debug_checked_push(ct.tx_index, StoredBool::from(analysis.explicitly_rbf));
        counts.add_base(
            ct.tx.input.len(),
            ct.tx.output.len(),
            tx_version,
            analysis.explicitly_rbf,
        );
        features.push_and_count(analysis.features, &mut counts);
    }
    features.count.push(height, counts);
}
