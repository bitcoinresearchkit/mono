use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use brk_exit::Exit;
use brk_types::{Height, OutputType, Sats, TxOutIndex};
use vecdb::{AnyStoredVec, AnyVec, ReadableVec, VecIndex, WritableVec};

use super::Vecs;

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    prices: &bitview_plugin_price::Vecs,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(indexer, prices, exit)
}

impl Vecs {
    fn compute(
        &mut self,
        indexer: &Indexer,
        prices: &bitview_plugin_price::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.safe_lengths();
        let height_vec = &mut self.op_return.cumulative.sats.height;

        // Validate computed versions against dependencies
        let dep_version = indexer.vecs().outputs.first_txout_index.version()
            + indexer.vecs().outputs.output_type.version()
            + indexer.vecs().outputs.value.version();
        height_vec.validate_computed_version_or_reset(dep_version)?;

        // Get target height
        let target_len = indexer.vecs().outputs.first_txout_index.len();
        if target_len == 0 {
            self.op_return.compute_cents(
                starting_lengths.height,
                &prices.spot.cents.height,
                exit,
            )?;
            return Ok(());
        }
        let target_height = Height::from(target_len - 1);

        // Find starting height for this vec
        let current_len = height_vec.len();
        let starting_height = Height::from(current_len.min(starting_lengths.height.to_usize()));

        if starting_height <= target_height {
            // Pre-collect height-indexed data
            let first_txout_indexes: Vec<TxOutIndex> =
                indexer.vecs().outputs.first_txout_index.collect_range_at(
                    starting_height.to_usize(),
                    target_height.to_usize()
                        + 2.min(indexer.vecs().outputs.first_txout_index.len()),
                );

            let mut output_types_buf: Vec<OutputType> = Vec::new();
            let mut values_buf: Vec<Sats> = Vec::new();
            let mut cumulative = starting_height
                .decremented()
                .and_then(|height| height_vec.collect_one(height))
                .unwrap_or_default();

            height_vec.truncate_if_needed(starting_height)?;

            // Iterate blocks
            for h in starting_height.to_usize()..=target_height.to_usize() {
                let local_idx = h - starting_height.to_usize();

                // Get output range for this block
                let first_txout_index = first_txout_indexes[local_idx];
                let next_first_txout_index =
                    if let Some(&next) = first_txout_indexes.get(local_idx + 1) {
                        next
                    } else {
                        TxOutIndex::from(indexer.vecs().outputs.value.len())
                    };

                let out_start = first_txout_index.to_usize();
                let out_end = next_first_txout_index.to_usize();

                // Pre-collect both vecs into reusable buffers
                indexer.vecs().outputs.output_type.collect_range_into_at(
                    out_start,
                    out_end,
                    &mut output_types_buf,
                );
                indexer.vecs().outputs.value.collect_range_into_at(
                    out_start,
                    out_end,
                    &mut values_buf,
                );

                let mut op_return_value = Sats::ZERO;
                for (ot, val) in output_types_buf.iter().zip(values_buf.iter()) {
                    if *ot == OutputType::OpReturn {
                        op_return_value += *val;
                    }
                }

                cumulative += op_return_value;
                height_vec.push(cumulative);
            }

            height_vec.write()?;
        }

        self.op_return
            .compute_cents(starting_lengths.height, &prices.spot.cents.height, exit)?;

        Ok(())
    }
}
