use std::iter;

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
            let types = &indexer.vecs().outputs.output_type;
            let first_tx = fi_batch
                .first()
                .expect("block range is nonempty")
                .to_usize();
            let mut starts = indexer
                .vecs()
                .transactions
                .first_txout_index
                .range_cursor_at(first_tx, txid_len);
            let first_entry = starts.next().data()?.to_usize();
            let mut types_cursor = types.range_cursor_at(first_entry, types.len());
            walk_blocks(
                &fi_batch,
                txid_len,
                first_entry..types.len(),
                iter::from_fn(|| starts.next().map(|index| index.to_usize())),
                CoinbasePolicy::Include,
                |count, target| {
                    if let Some(per_tx) = target {
                        types_cursor.for_each(count, |kind| per_tx[kind as usize] += 1);
                    } else {
                        types_cursor.advance(count);
                    }
                },
                store,
            )
        },
    )
}
