use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Sats, StoredBool, StoredU64};
use vecdb::{AnyStoredVec, AnyVec, ReadableVec, VecIndex, WritableVec};

use super::Vecs;
use crate::fees;

const FIRST_EPHEMERAL_DUST_HEIGHT: usize = 905_000;
const WRITE_INTERVAL: usize = 10_000;

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    mappings: &MappingsVecs,
    fees: &fees::Vecs,
    exit: &Exit,
) -> Result<()> {
    let features = &indexer.vecs().transaction_features;
    let version = features.is_unconditionally_nonstandard.version()
        + features.has_dust_output.version()
        + fees.fee.tx_index.version()
        + indexer.vecs().transactions.first_tx_index.version()
        + mappings.height.tx_index_count.version();
    {
        let _lock = exit.lock();
        vecs.is_nonstandard
            .validate_computed_version_or_reset(version)?;
        vecs.count
            .nonstandard
            .validate_computed_version_or_reset(version)?;
    }
    let starting_lengths = indexer.safe_lengths();
    let target_tx = fees.fee.tx_index.len();
    let target_height = mappings.height.tx_index_count.len();
    let tx_len = vecs
        .is_nonstandard
        .len()
        .min(starting_lengths.tx_index.to_usize());
    let count_len = vecs
        .count
        .nonstandard
        .cumulative
        .height
        .len()
        .min(starting_lengths.height.to_usize());
    let next_height = mappings
        .tx_heights
        .resume_height(tx_len, target_tx, target_height);
    let start_height = count_len.min(next_height);
    if start_height >= target_height {
        return Ok(());
    }

    let first_tx = &indexer.vecs().transactions.first_tx_index;
    let start_tx = first_tx.collect_one_at(start_height).unwrap().to_usize();
    {
        let _lock = exit.lock();
        vecs.is_nonstandard.truncate_if_needed_at(start_tx)?;
        vecs.count.nonstandard.truncate_if_needed_at(start_height)?;
    }
    let mut unconditional = features
        .is_unconditionally_nonstandard
        .range_cursor_at(start_tx, target_tx);
    let mut has_dust = features
        .has_dust_output
        .range_cursor_at(start_tx, target_tx);
    let mut fee = fees.fee.tx_index.cursor();
    let mut tx_count = mappings.height.tx_index_count.cursor();
    tx_count.advance(start_height);

    let mut block_start = start_tx;
    for height in start_height..target_height {
        let block_end = (block_start + u64::from(tx_count.next().unwrap()) as usize).min(target_tx);
        let mut count = 0;

        for tx_index in block_start..block_end {
            let raw = unconditional.next().unwrap().is_true();
            let dust = has_dust.next().unwrap().is_true();
            let nonstandard =
                raw || (dust && dust_is_nonstandard(height, || fee.get(tx_index).unwrap()));
            count += nonstandard as u64;
            vecs.is_nonstandard.push(StoredBool::from(nonstandard));
        }
        vecs.count.nonstandard.push_block(StoredU64::from(count));

        if (height + 1).is_multiple_of(WRITE_INTERVAL) {
            let _lock = exit.lock();
            vecs.is_nonstandard.write()?;
            vecs.count.nonstandard.write()?;
        }

        block_start = block_end;
    }

    let _lock = exit.lock();
    vecs.is_nonstandard.write()?;
    vecs.count.nonstandard.write()?;
    Ok(())
}

fn dust_is_nonstandard(height: usize, fee: impl FnOnce() -> Sats) -> bool {
    height < FIRST_EPHEMERAL_DUST_HEIGHT || fee() != Sats::ZERO
}

#[cfg(test)]
mod tests {
    use brk_types::Sats;

    use super::{FIRST_EPHEMERAL_DUST_HEIGHT, dust_is_nonstandard};

    #[test]
    fn zero_fee_ephemeral_dust_starts_at_activation() {
        assert!(dust_is_nonstandard(
            FIRST_EPHEMERAL_DUST_HEIGHT - 1,
            || panic!("fees are irrelevant before activation"),
        ));
        assert!(!dust_is_nonstandard(FIRST_EPHEMERAL_DUST_HEIGHT, || {
            Sats::ZERO
        }));
        assert!(dust_is_nonstandard(FIRST_EPHEMERAL_DUST_HEIGHT, || {
            Sats::new(1)
        }));
    }
}
