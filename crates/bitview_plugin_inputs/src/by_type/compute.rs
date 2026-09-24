use bitview_compute::{CoinbasePolicy, walk_blocks};
use bitview_plugin_indexer::Indexer;
use bitview_vecs::compute_type_counts;
use brk_error::{OptionData, Result};
use brk_exit::Exit;
use vecdb::{AnyVec, ReadableVec, VecIndex};

use super::Vecs;

impl Vecs {
    pub fn compute(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        let dep_version = indexer.vecs().inputs.output_type.version()
            + indexer.vecs().transactions.first_tx_index.version()
            + indexer.vecs().transactions.first_txin_index.version()
            + indexer.vecs().transactions.txid.version();
        let first_tx_index = &indexer.vecs().transactions.first_tx_index;
        let end = first_tx_index.len();
        compute_type_counts(
            self.input_count_stored
                .iter_typed_mut()
                .zip(self.tx_count_stored.iter_mut())
                .map(|((kind, entries), txs)| (kind, entries, txs)),
            indexer.safe_lengths().height,
            end,
            dep_version,
            exit,
            |skip, store| {
                let fi_batch = first_tx_index.collect_range_at(skip, end);
                let txid_len = indexer.vecs().transactions.txid.len();
                let total_txin_len = indexer.vecs().inputs.output_type.len();

                let mut fi_in_cursor = indexer.vecs().transactions.first_txin_index.cursor();
                let first_tx = fi_batch
                    .first()
                    .expect("block range is nonempty")
                    .to_usize()
                    + 1;
                let first_txin = if first_tx < txid_len {
                    fi_in_cursor.get(first_tx).data()?.to_usize()
                } else {
                    total_txin_len
                };
                let mut itype_cursor = indexer
                    .vecs()
                    .inputs
                    .output_type
                    .range_cursor_at(first_txin, total_txin_len);
                walk_blocks(
                    &fi_batch,
                    txid_len,
                    CoinbasePolicy::Skip,
                    |tx_pos, per_tx| {
                        let fi_in = fi_in_cursor.get(tx_pos).data()?.to_usize();
                        let next_fi_in = if tx_pos + 1 < txid_len {
                            fi_in_cursor.get(tx_pos + 1).data()?.to_usize()
                        } else {
                            total_txin_len
                        };

                        itype_cursor.advance(fi_in - itype_cursor.position());
                        itype_cursor.for_each(next_fi_in - fi_in, |otype| {
                            per_tx[otype as usize] += 1;
                        });
                        Ok(())
                    },
                    store,
                )
            },
        )
    }
}
