use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::RatioSats;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, PartsPerMillion32, Sats, StoredBool, StoredU64, TxIndex};
use vecdb::{AnyStoredVec, AnyVec, BinaryTransform, ReadableVec, VecIndex, WritableVec};

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
        &features.has_inscription,
        &fees.fee.tx_index,
        exit,
    )
}

impl Vecs {
    pub(super) fn compute_fees(
        &mut self,
        starting_height: Height,
        first_tx: &impl ReadableVec<Height, TxIndex>,
        tx_counts: &impl ReadableVec<Height, StoredU64>,
        inscriptions: &impl ReadableVec<TxIndex, StoredBool>,
        fees: &impl ReadableVec<TxIndex, Sats>,
        exit: &Exit,
    ) -> Result<()> {
        let version =
            first_tx.version() + tx_counts.version() + inscriptions.version() + fees.version();
        self.fees.validate_and_truncate(version, starting_height)?;
        self.fee_share
            .ppm
            .height
            .validate_and_truncate(version, starting_height)?;

        let end_height = first_tx.len().min(tx_counts.len());
        let start_height = self
            .fees
            .cumulative
            .height
            .len()
            .min(self.fee_share.ppm.height.len())
            .min(end_height);
        self.fees.truncate_if_needed_at(start_height)?;
        self.fee_share
            .ppm
            .height
            .truncate_if_needed_at(start_height)?;
        if start_height == end_height {
            let _lock = exit.lock();
            self.fees.stored_mut().write()?;
            self.fee_share.ppm.height.write()?;
            return Ok(());
        }

        let end_tx = inscriptions.len().min(fees.len());
        let mut block_start = first_tx.collect_one_at(start_height).unwrap().to_usize();
        let mut inscriptions = inscriptions.cursor();
        let mut fees = fees.cursor();
        let mut counts = tx_counts.cursor();
        inscriptions.advance(block_start);
        fees.advance(block_start);
        counts.advance(start_height);

        for height in start_height..end_height {
            let block_end = block_start + u64::from(counts.next().unwrap()) as usize;
            // Only publish complete blocks when a dependency is still catching up.
            if block_end > end_tx {
                break;
            }
            let mut total = Sats::ZERO;
            let mut inscribed = Sats::ZERO;
            for _ in block_start..block_end {
                let fee = fees.next().unwrap();
                total += fee;
                if inscriptions.next().unwrap().is_true() {
                    inscribed += fee;
                }
            }
            self.fees.push_block(inscribed);
            self.fee_share
                .ppm
                .height
                .push(RatioSats::<PartsPerMillion32>::apply(inscribed, total));
            block_start = block_end;

            if (height + 1).is_multiple_of(WRITE_INTERVAL) {
                let _lock = exit.lock();
                self.fees.stored_mut().write()?;
                self.fee_share.ppm.height.write()?;
            }
        }
        let _lock = exit.lock();
        self.fees.stored_mut().write()?;
        self.fee_share.ppm.height.write()?;
        Ok(())
    }
}
