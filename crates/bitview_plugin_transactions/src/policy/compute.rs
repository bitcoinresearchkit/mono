use bitview_compute::{WRITE_INTERVAL, write_locked};
use bitview_plugin_indexer::{FIRST_V29_POLICY_HEIGHT, Indexer};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Boolean, Count};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Sats;
use vecdb::{AnyVec, ReadableVec, WritableVec};

use super::Vecs;
use crate::{fees, resume::prepare_blocks};

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        fees: &fees::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let features = &indexer.vecs().transactions.features;
        let version = features.is_unconditionally_nonstandard.version()
            + features.dust_output.flag.version()
            + fees.fee.tx.version()
            + indexer.vecs().transactions.first_tx_index.version()
            + mappings.height.tx_index_count.version();
        let first_tx = &indexer.vecs().transactions.first_tx_index;
        let tx_counts = &mappings.height.tx_index_count;
        let end_tx = fees.fee.tx.len();
        let (flag, count) = self.nonstandard.stored_mut();
        let (heights, start_tx) = prepare_blocks(
            &mut [flag],
            &mut [count],
            version,
            indexer.compute_start(),
            end_tx,
            tx_counts.len(),
            mappings,
            first_tx,
            exit,
        )?;
        let mut unconditional = features
            .is_unconditionally_nonstandard
            .range_cursor_at(start_tx, end_tx);
        let mut has_dust = features.dust_output.flag.range_cursor_at(start_tx, end_tx);
        let mut fee = fees.fee.tx.cursor();
        let mut tx_count = tx_counts.cursor();
        tx_count.advance(heights.start);

        let mut block_start = start_tx;
        for height in heights {
            let block_end =
                (block_start + u64::from(tx_count.next().unwrap()) as usize).min(end_tx);
            let mut count = 0;

            for tx_index in block_start..block_end {
                let raw = unconditional.next().unwrap().is_true();
                let dust = has_dust.next().unwrap().is_true();
                let nonstandard =
                    raw || (dust && dust_is_nonstandard(height, || fee.get(tx_index).unwrap()));
                count += nonstandard as u64;
                self.nonstandard.flag.push(Boolean::from(nonstandard));
            }
            self.nonstandard.count.push_block(Count::from(count));

            if (height + 1).is_multiple_of(WRITE_INTERVAL) {
                self.write(exit)?;
            }

            block_start = block_end;
        }

        self.write(exit)
    }

    fn write(&mut self, exit: &Exit) -> Result<()> {
        let (flag, count) = self.nonstandard.stored_mut();
        write_locked([flag, count], exit)
    }
}

fn dust_is_nonstandard(height: usize, fee: impl FnOnce() -> Sats) -> bool {
    height < FIRST_V29_POLICY_HEIGHT as usize || fee() != Sats::ZERO
}
