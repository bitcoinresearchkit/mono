use std::iter;

use bitview_plugin_indexer::Indexer;
use bitview_vecs::{CoinbasePolicy, compute_type_counts};
use brk_error::Result;
use brk_exit::Exit;
use vecdb::{AnyVec, VecIndex};

use super::Vecs;

impl Vecs {
    pub(crate) fn compute(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        let txs = &indexer.vecs().transactions;
        let types = &indexer.vecs().inputs.output_type;
        let txid_len = txs.txid.len();
        let entries_len = types.len();
        compute_type_counts(
            self.count_stored
                .iter_typed_mut()
                .zip(self.tx_count_stored.iter_mut())
                .map(|((kind, entries), txs)| (kind, entries, txs)),
            &txs.first_tx_index,
            txid_len,
            entries_len,
            indexer.safe_lengths().height,
            types.version()
                + txs.first_tx_index.version()
                + txs.first_txin_index.version()
                + txs.txid.version(),
            CoinbasePolicy::Skip,
            |first_tx| {
                let mut starts = txs.first_txin_index.range_cursor_at(first_tx, txid_len);
                iter::from_fn(move || starts.next().map(|index| index.to_usize()))
            },
            |first_entry| {
                let mut types = types.range_cursor_at(first_entry, entries_len);
                move |count, target: Option<&mut [u32; _]>| match target {
                    Some(per_tx) => types.for_each(count, |kind| per_tx[kind as usize] += 1),
                    None => types.advance(count),
                }
            },
            exit,
        )
    }
}
