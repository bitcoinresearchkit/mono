use bitview_cohort::OutputTypeId;
use bitview_compute::{CoinbasePolicy, walk_blocks};
use bitview_plugin_indexer::Indexer;
use brk_error::{OptionData, Result};
use brk_types::{StoredU16, StoredU64};
use vecdb::{AnyVec, ColumnId, Exit, ReadableVec, VecIndex};

use super::Vecs;

const WRITE_INTERVAL: usize = 10_000;

pub fn compute(vecs: &mut Vecs, indexer: &Indexer, exit: &Exit) -> Result<()> {
    vecs.compute(indexer, exit)
}

impl Vecs {
    fn compute(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        let starting_lengths = indexer.safe_lengths();

        let dep_version = indexer.vecs().outputs.output_type.version()
            + indexer.vecs().transactions.first_tx_index.version()
            + indexer.vecs().transactions.first_txout_index.version()
            + indexer.vecs().transactions.txid.version();

        self.output_count
            .validate_and_truncate(dep_version, starting_lengths.height)?;
        self.output_count.invalidate();
        self.tx_count
            .validate_and_truncate(dep_version, starting_lengths.height)?;

        let skip = self
            .output_count
            .height
            .len()
            .min(self.tx_count.cumulative.len());

        let first_tx_index = &indexer.vecs().transactions.first_tx_index;
        let end = first_tx_index.len();
        if skip < end {
            self.output_count.truncate_if_needed_at(skip)?;
            self.tx_count.truncate_if_needed_at(skip)?;

            let fi_batch = first_tx_index.collect_range_at(skip, end);
            let txid_len = indexer.vecs().transactions.txid.len();
            let total_txout_len = indexer.vecs().outputs.output_type.len();
            let fo_reader = indexer.vecs().transactions.first_txout_index.reader();
            let first_tx = fi_batch
                .first()
                .expect("block range is nonempty")
                .to_usize();
            let first_txout = fo_reader.try_get_at(first_tx).data()?.to_usize();
            let mut otype_cursor = indexer
                .vecs()
                .outputs
                .output_type
                .range_cursor_at(first_txout, total_txout_len);
            let mut height = skip;

            walk_blocks(
                &fi_batch,
                txid_len,
                CoinbasePolicy::Include,
                |tx_pos, per_tx| {
                    let next_fo = if tx_pos + 1 < txid_len {
                        fo_reader.try_get_at(tx_pos + 1).data()?.to_usize()
                    } else {
                        total_txout_len
                    };

                    let output_count = next_fo - otype_cursor.position();
                    otype_cursor.for_each(output_count, |otype| {
                        per_tx[otype as usize] += 1;
                    });
                    Ok(())
                },
                |agg| {
                    self.output_count.push(OutputTypeId::from_fn(|column| {
                        let value = agg.entries_per_type[column.output_type() as usize];
                        debug_assert!(u16::try_from(value).is_ok());
                        StoredU16::new(value as u16)
                    }));
                    self.tx_count.push_block(OutputTypeId::from_fn(|column| {
                        StoredU64::from(agg.txs_per_type[column.output_type() as usize])
                    }));

                    height += 1;
                    if height.is_multiple_of(WRITE_INTERVAL) {
                        let _lock = exit.lock();
                        self.output_count.write()?;
                        self.tx_count.write()?;
                    }
                    Ok(())
                },
            )?;

            {
                let _lock = exit.lock();
                self.output_count.write()?;
                self.tx_count.write()?;
            }
            self.spendable_output_count.invalidate();
        }

        Ok(())
    }
}
