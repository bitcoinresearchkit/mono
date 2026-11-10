use std::ops::Range;

use bitview_compute::{Inputs, write_locked};
use bitview_plugin_indexer::Indexer;
use bitview_primitives::{BlockVersion, Count};
use bitview_traversable::Traversable;
use bitview_vecs::{CumulativeRolling, IndexSources, LazyWindowStartVec, shapes::Windows};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use vecdb::{AnyVec, Database, ReadableVec, Rw, StorageMode, VecIndex};

/// Blocks with bit 4 set in a BIP-9 version, the bit BIP-110 signals with: the top three version
/// bits are 001 and bit 4 is 1.
pub(crate) fn compute_version_bit_4(
    total: &mut CumulativeRolling<Count>,
    indexer: &Indexer,
    from: Height,
    exit: &Exit,
) -> Result<()> {
    total.compute_cumulative_transformed(
        from,
        &indexer.vecs().blocks.version,
        |version: BlockVersion| Count::from(version.is_signalling_soft_fork(4)),
        exit,
    )
}

#[derive(Traversable)]
pub(crate) struct CoinbaseLocktime<M: StorageMode = Rw> {
    /// Blocks whose coinbase has a nonzero nLockTime.
    #[traversable(lead = "Blocks with a locktime set on the coinbase.")]
    nonzero: CumulativeRolling<Count, M>,
    /// Blocks whose coinbase nLockTime is the height minus 1 and whose sequence isn't final:
    /// BIP-54's rule.
    #[traversable(lead = "Blocks following BIP-54's coinbase locktime rule.")]
    bip54: CumulativeRolling<Count, M>,
}

impl CoinbaseLocktime {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            nonzero: CumulativeRolling::import(
                db,
                "nonzero_coinbase_locktime_block_count",
                version,
                indexes,
                window_starts,
            )?,
            bip54: CumulativeRolling::import(
                db,
                "bip54_coinbase_locktime_block_count",
                version,
                indexes,
                window_starts,
            )?,
        })
    }

    pub(crate) fn compute(&mut self, indexer: &Indexer, from: Height, exit: &Exit) -> Result<()> {
        let blocks = &indexer.vecs().blocks;
        let (locktime, sequence) = (&blocks.coinbase_locktime, &blocks.coinbase_sequence);
        let Range { start, end } =
            Inputs::new(Version::ZERO, [locktime as &dyn AnyVec, sequence]).prepare(
                [self.nonzero.stored_mut(), self.bip54.stored_mut()],
                from.to_usize(),
                exit,
            )?;
        if start < end {
            let locktimes = locktime.collect_range_at(start, end);
            let sequences = sequence.collect_range_at(start, end);
            for (offset, (locktime, sequence)) in locktimes.iter().zip(&sequences).enumerate() {
                let (locktime, height) = (u32::from(*locktime), start + offset);
                let bip54 = height > 0 && locktime as usize == height - 1 && !sequence.is_final();
                self.nonzero.push_block(Count::from(locktime != 0));
                self.bip54.push_block(Count::from(bip54));
            }
        }
        write_locked([self.nonzero.stored_mut(), self.bip54.stored_mut()], exit)
    }
}
