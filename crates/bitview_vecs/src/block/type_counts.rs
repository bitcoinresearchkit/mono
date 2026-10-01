use bitview_compute::{BlockAggregate, prepare_computed};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, OutputType, StoredU64, Version};
use vecdb::{AnyStoredVec, ReadableVec, WritableVec};

use crate::CachedSeries;

const WRITE_INTERVAL: usize = 10_000;

type TypeCountTarget<'a> = (
    OutputType,
    &'a mut CachedSeries<Height, StoredU64>,
    &'a mut CachedSeries<Height, StoredU64>,
);

/// Compute cumulative entry and transaction counts for the selected output
/// types. Inputs skip coinbase transactions; outputs include them.
pub fn compute_type_counts<'a>(
    targets: impl IntoIterator<Item = TypeCountTarget<'a>>,
    max_from: Height,
    end: usize,
    version: Version,
    exit: &Exit,
    scan: impl FnOnce(usize, &mut dyn FnMut(BlockAggregate) -> Result<()>) -> Result<()>,
) -> Result<()> {
    let mut targets: Vec<_> = targets.into_iter().collect();
    let skip = prepare_computed(
        targets
            .iter_mut()
            .flat_map(|(_, entries, txs)| {
                [
                    &mut **entries as &mut dyn AnyStoredVec,
                    &mut **txs as &mut dyn AnyStoredVec,
                ]
            })
            .collect::<Vec<_>>(),
        version,
        usize::from(max_from).min(end),
        exit,
    )?;
    if skip >= end || targets.is_empty() {
        return Ok(());
    }
    let mut entry_totals = [StoredU64::ZERO; OutputType::COUNT];
    let mut tx_totals = [StoredU64::ZERO; OutputType::COUNT];
    for (kind, entries, txs) in &mut targets {
        entry_totals[*kind as usize] = entries.collect_last().unwrap_or_default();
        tx_totals[*kind as usize] = txs.collect_last().unwrap_or_default();
    }

    let mut height = skip;
    let write = |targets: &mut [TypeCountTarget<'_>]| -> Result<()> {
        let _lock = exit.lock();
        for (_, entries, txs) in targets {
            entries.write()?;
            txs.write()?;
        }
        Ok(())
    };
    scan(skip, &mut |aggregate| {
        for (kind, entries, txs) in &mut targets {
            let kind = *kind as usize;
            entry_totals[kind] += StoredU64::from(aggregate.entries_per_type[kind]);
            tx_totals[kind] += StoredU64::from(aggregate.txs_per_type[kind]);
            entries.push(entry_totals[kind]);
            txs.push(tx_totals[kind]);
        }
        height += 1;
        if height.is_multiple_of(WRITE_INTERVAL) {
            write(&mut targets)?;
        }
        Ok(())
    })?;
    write(&mut targets)
}
