//! Shared per-block-per-type cursor walker used by `outputs/by_type/` and
//! `inputs/by_type/`. The walker iterates blocks and aggregates the
//! per-tx output-type counts; pushing into a particular wrapper is left
//! to the caller.

use brk_error::Result;
use brk_types::{OutputType, TxIndex};
use vecdb::VecIndex;

/// Aggregated per-block counters produced by [`walk_blocks`].
pub(crate) struct BlockAggregate {
    pub entries_per_type: [u64; OutputType::COUNT],
    pub txs_per_type: [u64; OutputType::COUNT],
}

/// Whether to include the coinbase tx (first tx in each block) in the walk.
#[derive(Clone, Copy)]
pub(crate) enum CoinbasePolicy {
    Include,
    Skip,
}

/// Walk every block in `fi_batch`, calling `scan_tx` once per tx (which
/// fills a per-output-type count array for that tx),
/// aggregating into a [`BlockAggregate`] and handing it to `store`.
///
#[inline]
pub(crate) fn walk_blocks(
    fi_batch: &[TxIndex],
    txid_len: usize,
    coinbase: CoinbasePolicy,
    mut scan_tx: impl FnMut(usize, &mut [u32; OutputType::COUNT]) -> Result<()>,
    mut store: impl FnMut(BlockAggregate) -> Result<()>,
) -> Result<()> {
    for (j, first_tx) in fi_batch.iter().enumerate() {
        let fi = first_tx.to_usize();
        let next_fi = fi_batch
            .get(j + 1)
            .map(|v| v.to_usize())
            .unwrap_or(txid_len);

        let start_tx = match coinbase {
            CoinbasePolicy::Include => fi,
            CoinbasePolicy::Skip => fi + 1,
        };

        let mut entries_per_type = [0u64; OutputType::COUNT];
        let mut txs_per_type = [0u64; OutputType::COUNT];

        for tx_pos in start_tx..next_fi {
            let mut per_tx = [0u32; OutputType::COUNT];
            scan_tx(tx_pos, &mut per_tx)?;
            for (i, &n) in per_tx.iter().enumerate() {
                if n > 0 {
                    entries_per_type[i] += u64::from(n);
                    txs_per_type[i] += 1;
                }
            }
        }

        store(BlockAggregate {
            entries_per_type,
            txs_per_type,
        })?;
    }

    Ok(())
}
