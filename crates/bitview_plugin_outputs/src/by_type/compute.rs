use bitview_compute::{CoinbasePolicy, walk_blocks};
use bitview_plugin_indexer::Indexer;
use bitview_vecs::compute_type_counts;
use brk_error::{OptionData, Result};
use brk_exit::Exit;
use vecdb::{AnyVec, ReadableVec, VecIndex};

use super::Vecs;

pub fn compute(vecs: &mut Vecs, indexer: &Indexer, exit: &Exit) -> Result<()> {
    let dep_version = indexer.vecs().outputs.output_type.version()
        + indexer.vecs().transactions.first_tx_index.version()
        + indexer.vecs().transactions.first_txout_index.version()
        + indexer.vecs().transactions.txid.version();
    let first_tx_index = &indexer.vecs().transactions.first_tx_index;
    let end = first_tx_index.len();
    compute_type_counts(
        vecs.output_count_stored
            .iter_typed_mut()
            .zip(vecs.tx_count_stored.iter_mut())
            .map(|((kind, entries), txs)| (kind, entries, txs)),
        indexer.safe_lengths().height,
        end,
        dep_version,
        exit,
        |skip, store| {
            let fi_batch = first_tx_index.collect_range_at(skip, end);
            let txid_len = indexer.vecs().transactions.txid.len();
            let total_txout_len = indexer.vecs().outputs.output_type.len();
            let first_txout_index = &indexer.vecs().transactions.first_txout_index;
            let first_tx = fi_batch
                .first()
                .expect("block range is nonempty")
                .to_usize();
            let mut first_txout_cursor = first_txout_index.range_cursor_at(first_tx, txid_len);
            let first_txout = first_txout_cursor.next().data()?.to_usize();
            let mut output_type_cursor = indexer
                .vecs()
                .outputs
                .output_type
                .range_cursor_at(first_txout, total_txout_len);
            walk_blocks(
                &fi_batch,
                txid_len,
                CoinbasePolicy::Include,
                |tx_pos, per_tx| {
                    let next_first_txout = if tx_pos + 1 < txid_len {
                        first_txout_cursor.next().data()?.to_usize()
                    } else {
                        total_txout_len
                    };

                    let output_count = next_first_txout - output_type_cursor.position();
                    output_type_cursor.for_each(output_count, |otype| {
                        per_tx[otype as usize] += 1;
                    });
                    Ok(())
                },
                store,
            )
        },
    )
}
