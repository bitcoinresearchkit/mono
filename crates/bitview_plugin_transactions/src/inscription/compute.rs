use bitview_compute::prepare_computed;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::RatioSats;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, PartsPerMillion32, Sats, StoredBool, StoredU64, TxIndex};
use vecdb::{AnyStoredVec, BinaryTransform, ReadableVec, VecIndex, WritableVec};

use super::Vecs;
use crate::FeesVecs;

const WRITE_INTERVAL: usize = 10_000;

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    mappings: &MappingsVecs,
    fees: &FeesVecs,
    exit: &Exit,
) -> Result<()> {
    let starting_height = indexer.safe_lengths().height;
    let features = &indexer.vecs().transaction_features;
    vecs.count.compute_cumulative_transformed(
        starting_height,
        &features.count.inscription,
        StoredU64::from,
        exit,
    )?;
    vecs.compute_fees(
        starting_height,
        &indexer.vecs().transactions.first_tx_index,
        &mappings.height.tx_index_count,
        &features.count.inscription,
        &features.has_inscription,
        &fees.fee.tx_index,
        &fees.total,
        exit,
    )
}

impl Vecs {
    #[allow(clippy::too_many_arguments)]
    fn compute_fees(
        &mut self,
        starting_height: Height,
        first_tx: &impl ReadableVec<Height, TxIndex>,
        tx_counts: &impl ReadableVec<Height, StoredU64>,
        inscription_counts: &impl ReadableVec<Height, StoredU64>,
        inscriptions: &impl ReadableVec<TxIndex, StoredBool>,
        fees: &impl ReadableVec<TxIndex, Sats>,
        total_fees: &impl ReadableVec<Height, Sats>,
        exit: &Exit,
    ) -> Result<()> {
        let version = first_tx.version()
            + tx_counts.version()
            + inscription_counts.version()
            + inscriptions.version()
            + fees.version()
            + total_fees.version();
        let end_height = first_tx
            .len()
            .min(tx_counts.len())
            .min(inscription_counts.len())
            .min(total_fees.len());
        let start_height = prepare_computed(
            [self.fees.stored_mut(), &mut self.fee_share.ppm.height],
            version,
            starting_height.to_usize().min(end_height),
            exit,
        )?;
        if start_height == end_height {
            return self.write_fees(exit);
        }

        let end_tx = inscriptions.len().min(fees.len());
        let mut block_start = first_tx.collect_one_at(start_height).unwrap().to_usize();
        let mut inscriptions = inscriptions.cursor();
        let mut fees = fees.cursor();
        let mut counts = tx_counts.cursor();
        let mut inscription_counts = inscription_counts.cursor();
        counts.advance(start_height);
        inscription_counts.advance(start_height);

        for height in start_height..end_height {
            let block_end = block_start + u64::from(counts.next().unwrap()) as usize;
            // Only publish complete blocks when a dependency is still catching up.
            if block_end > end_tx {
                break;
            }
            let total = total_fees.collect_one_at(height).unwrap();
            let mut inscribed = Sats::ZERO;
            // With no inscriptions, avoid reading transaction fees or flags.
            if u64::from(inscription_counts.next().unwrap()) > 0 {
                fees.advance(block_start - fees.position());
                inscriptions.advance(block_start - inscriptions.position());
                fees.for_each(block_end - block_start, |fee| {
                    if inscriptions.next().unwrap().is_true() {
                        inscribed += fee;
                    }
                });
            }
            self.fees.push_block(inscribed);
            self.fee_share
                .ppm
                .height
                .push(RatioSats::<PartsPerMillion32>::apply(inscribed, total));
            block_start = block_end;

            if (height + 1).is_multiple_of(WRITE_INTERVAL) {
                self.write_fees(exit)?;
            }
        }
        self.write_fees(exit)
    }

    fn write_fees(&mut self, exit: &Exit) -> Result<()> {
        let _lock = exit.lock();
        self.fees.write()?;
        self.fee_share.ppm.height.write()?;
        Ok(())
    }
}
