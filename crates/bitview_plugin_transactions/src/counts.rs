use std::ops::Range;

use bitview_compute::{Inputs, write_locked};
use bitview_plugin_indexer::{BlockCounterVecs, Indexer};
use bitview_primitives::Count;
use bitview_traversable::Traversable;
use bitview_vecs::{
    IndexSources, LazyWindowStartVec, PerBlockCumulativeRolling, counter_group, shapes::Windows,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, TxIndex, Version};
use vecdb::{AnyVec, Database, ReadableVec, Rw, StorageMode, VecIndex};

/// Payments: a transaction's outputs other than OP_RETURN, less one for its change when it has
/// more than one. A heuristic.
#[derive(Traversable)]
pub(crate) struct Payments<M: StorageMode = Rw> {
    #[traversable(lead = "Payments, estimated from transaction outputs.")]
    pub(crate) count: PerBlockCumulativeRolling<Count, M>,
    /// Payments of transactions with any input spending SegWit.
    #[traversable(lead = "Payments made by transactions spending SegWit.")]
    pub(crate) segwit_spending: PerBlockCumulativeRolling<Count, M>,
    /// Payments of transactions signaling replacement under BIP-125.
    #[traversable(lead = "Payments made by transactions signaling replacement.")]
    pub(crate) rbf_signaling: PerBlockCumulativeRolling<Count, M>,
}

impl Payments {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            count: PerBlockCumulativeRolling::import(
                db,
                "payment_count",
                version,
                indexes,
                window_starts,
            )?,
            segwit_spending: PerBlockCumulativeRolling::import(
                db,
                "segwit_spending_payment_count",
                version,
                indexes,
                window_starts,
            )?,
            rbf_signaling: PerBlockCumulativeRolling::import(
                db,
                "rbf_signaling_payment_count",
                version,
                indexes,
                window_starts,
            )?,
        })
    }
}

#[derive(Traversable)]
pub(crate) struct Timelocks<M: StorageMode = Rw> {
    /// Transactions with an nLockTime that is a block height above 0.
    #[traversable(lead = "Transactions locked to a height.")]
    pub(crate) height: PerBlockCumulativeRolling<Count, M>,
    /// Transactions with an nLockTime that is a timestamp.
    #[traversable(lead = "Transactions locked to a time.")]
    pub(crate) time: PerBlockCumulativeRolling<Count, M>,
    /// Transactions with a nonzero nLockTime and every sequence final, so it isn't enforced.
    #[traversable(lead = "Transactions with an unenforced locktime.")]
    unenforced: PerBlockCumulativeRolling<Count, M>,
    /// Version-2 or later transactions with an input locked by a BIP-68 block count.
    #[traversable(lead = "Transactions with a relative block-count lock.")]
    relative_height: PerBlockCumulativeRolling<Count, M>,
    /// Version-2 or later transactions with an input locked by a BIP-68 time.
    #[traversable(lead = "Transactions with a relative time lock.")]
    relative_time: PerBlockCumulativeRolling<Count, M>,
}

impl Timelocks {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            height: PerBlockCumulativeRolling::import(
                db,
                "height_locktime_tx_count",
                version,
                indexes,
                window_starts,
            )?,
            time: PerBlockCumulativeRolling::import(
                db,
                "time_locktime_tx_count",
                version,
                indexes,
                window_starts,
            )?,
            unenforced: PerBlockCumulativeRolling::import(
                db,
                "unenforced_locktime_tx_count",
                version,
                indexes,
                window_starts,
            )?,
            relative_height: PerBlockCumulativeRolling::import(
                db,
                "relative_height_locktime_tx_count",
                version,
                indexes,
                window_starts,
            )?,
            relative_time: PerBlockCumulativeRolling::import(
                db,
                "relative_time_locktime_tx_count",
                version,
                indexes,
                window_starts,
            )?,
        })
    }

    /// Height and time locks from each transaction's stored locktime, in one walk; the three only
    /// its inputs tell, from the indexer's counts.
    pub(crate) fn compute(
        &mut self,
        max_from: Height,
        indexer: &Indexer,
        exit: &Exit,
    ) -> Result<()> {
        let txs = &indexer.vecs().transactions;
        let (first_tx, locktime) = (&txs.first_tx_index, &txs.raw_locktime);
        let Range { start, end } = Inputs::new(Version::ZERO, [first_tx as &dyn AnyVec, locktime])
            .prepare(
                [self.height.stored_mut(), self.time.stored_mut()],
                max_from.to_usize(),
                exit,
            )?;
        if start < end {
            let mut bounds = first_tx.collect_range_at(start, end);
            bounds.push(
                first_tx
                    .collect_one_at(end)
                    .unwrap_or(TxIndex::from(locktime.len())),
            );
            let mut locktimes =
                locktime.range_cursor_at(bounds[0].to_usize(), bounds[end - start].to_usize());
            for pair in bounds.windows(2) {
                let (mut height, mut time) = (0u64, 0u64);
                locktimes.for_each(pair[1].to_usize() - pair[0].to_usize(), |locktime| {
                    let is_time = locktime.is_block_time();
                    height += u64::from(!is_time && u32::from(locktime) > 0);
                    time += u64::from(is_time);
                });
                self.height.push_block(Count::from(height));
                self.time.push_block(Count::from(time));
            }
        }
        write_locked([self.height.stored_mut(), self.time.stored_mut()], exit)?;
        let counters = &txs.counters;
        self.unenforced
            .compute(max_from, &counters.unenforced_locktime_tx_count, exit)?;
        self.relative_height.compute(
            max_from,
            &counters.relative_height_locktime_tx_count,
            exit,
        )?;
        self.relative_time
            .compute(max_from, &counters.relative_time_locktime_tx_count, exit)
    }
}

counter_group! {
    BlockCounterVecs;

    pub(crate) struct Spending {
        /// Every input spends SegWit.
        #[traversable(lead = "Transactions spending only SegWit.")]
        only_segwit: Count => only_segwit_spending_tx_count,
        /// No input spends SegWit.
        #[traversable(lead = "Transactions spending only legacy outputs.")]
        only_legacy: Count => only_legacy_spending_tx_count,
        /// Some inputs spend SegWit and some do not.
        #[traversable(lead = "Transactions spending SegWit and legacy outputs together.")]
        segwit_and_legacy: Count => segwit_and_legacy_spending_tx_count,
    }
}
