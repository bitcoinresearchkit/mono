use bitview_compute::WRITE_INTERVAL;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::TxInIndex;
use brk_error::{Error, Result};
use brk_exit::Exit;
use brk_types::Height;
use rustc_hash::FxHashMap;
use statedb::{Amount, Spends};
use tracing::info;
use vecdb::{AnyVec, Bytes, ReadableVec, VecIndex};

use crate::STORAGE;

/// Groups each block's spends by the height of the output they spend, streaming the indexer's
/// input values and spent outputs in order; creation heights come from the in-memory
/// output-height map.
pub(crate) fn compute(
    spends: &mut Spends,
    indexer: &Indexer,
    mappings: &Mappings,
    from: Height,
    end: Height,
    exit: &Exit,
) -> Result<()> {
    let vecs = indexer.vecs();
    let first = &vecs.inputs.first_txin_index;
    let txout_indexes = &*vecs.inputs.txout_index;
    if from > end || usize::from(end) > first.len() {
        return Err(Error::NotFound("invalid input computation range".into()));
    }
    {
        let _lock = exit.lock();
        prepare(spends, indexer, from)?;
    }
    let start = spends.end();
    let end = usize::from(end);
    if start == end {
        return Ok(());
    }
    let mut boundaries = first.collect_range_at(start, end);
    debug_assert_eq!(boundaries.len(), end - start, "complete input boundaries");
    boundaries.push(
        first
            .collect_one_at(end)
            .unwrap_or(TxInIndex::from(txout_indexes.len())),
    );
    let (begin, finish) = (boundaries[0].to_usize(), boundaries[end - start].to_usize());
    let output_heights = mappings.output_heights.load_full();
    let mut values = vecs.inputs.value.range_cursor_at(begin, finish);
    let mut origins = txout_indexes.range_cursor_at(begin, finish);
    let mut hashes = vecs.blocks.blockhash.bounded_cursor_at(start, end);
    let mut spent: FxHashMap<u32, Amount> = FxHashMap::default();
    for (offset, pair) in boundaries.windows(2).enumerate() {
        // The first input belongs to the coinbase and has no previous output.
        values.advance(1);
        origins.advance(1);
        spent.clear();
        for _ in pair[0].to_usize() + 1..pair[1].to_usize() {
            let value = values.next().unwrap();
            let origin = output_heights
                .get_shared(origins.next().unwrap())
                .ok_or_else(|| Error::NotFound("spent output creation height".into()))?;
            let amount = spent.entry(u32::from(origin)).or_default();
            amount.sats += u64::from(value);
            amount.count += 1;
        }
        let h = start + offset;
        let hash = hashes.next().unwrap();
        let _lock = exit.lock();
        debug_assert_eq!(h, spends.end());
        spends.push(hash.to_bytes(), spent.iter().map(|(&h, &a)| (h, a)))?;
        if (h + 1).is_multiple_of(WRITE_INTERVAL) || h + 1 == end {
            spends.commit()?;
            if h + 1 < end {
                info!(
                    "Computing input origins: {:.0}%",
                    (offset + 1) as f64 / (end - start) as f64 * 100.0
                );
            }
        }
    }
    Ok(())
}

fn prepare(spends: &mut Spends, indexer: &Indexer, from: Height) -> Result<()> {
    let vecs = indexer.vecs();
    // The producer's own version and its inputs': a bump of either rebuilds the journal.
    let version = STORAGE.schema_version()
        + vecs.inputs.value.version()
        + vecs.inputs.txout_index.version()
        + vecs.inputs.first_txin_index.version()
        + vecs.outputs.first_txout_index.version();
    spends.validate_version(u32::from(version).into())?;
    if usize::from(from) < spends.end() {
        spends.truncate(usize::from(from))?;
    }
    let len = spends.end();
    if len > 0
        && Some(spends.hash(len - 1)?)
            != vecs
                .blocks
                .blockhash
                .collect_one(Height::from(len - 1))
                .map(|hash| hash.to_bytes())
    {
        return Err(Error::NotFound("movement chain prefix mismatch".into()));
    }
    Ok(())
}
