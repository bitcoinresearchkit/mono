use bitview_plugin_indexer::Indexer;
use bitview_primitives::Count;
use bitview_vecs::PerBlockCumulativeRolling;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Height;

use crate::flagged::TxValue;

use super::Vecs;

/// The versions' counts, each from the indexer's per-block count: 1, 2, 3, then the others.
pub(crate) struct Counts<'a>([&'a mut PerBlockCumulativeRolling<Count>; 4]);

impl Vecs {
    /// The counts and the values apart, so both compute at once: the values are the fee pass's.
    pub(crate) fn split_mut(&mut self) -> (Counts<'_>, [&mut TxValue; 4]) {
        let Self { v1, v2, v3, other } = self;
        (
            Counts([
                &mut v1.count,
                &mut v2.count,
                &mut v3.count,
                &mut other.count,
            ]),
            [
                &mut v1.value,
                &mut v2.value,
                &mut v3.value,
                &mut other.value,
            ],
        )
    }
}

impl Counts<'_> {
    pub(crate) fn compute(self, indexer: &Indexer, start: Height, exit: &Exit) -> Result<()> {
        let source = &indexer.vecs().transactions.versions;
        let sources = [
            &*source.v1.block,
            &*source.v2.block,
            &*source.v3.block,
            &*source.other.block,
        ];
        for (count, source) in self.0.into_iter().zip(sources) {
            count.compute_cumulative_transformed(start, source, Count::from, exit)?;
        }
        Ok(())
    }
}
