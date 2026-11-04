use bitview_plugin_indexer::Indexer;
use bitview_primitives::Count;
use bitview_vecs::PerBlockCumulativeRolling;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Sats;

use super::{Vecs, with_flagged_features};

/// The features, as the fee pass values them: those with a flag (in `with_flagged_features`
/// order), then SegWit, a single input and a single output.
pub(crate) const FEATURES: usize = 24;

/// The features' counts, each from the indexer's per-block count.
pub(crate) struct Counts<'a>([&'a mut PerBlockCumulativeRolling<Count>; FEATURES]);

impl Vecs {
    /// The counts and the values apart, so both compute at once: the values are the fee pass's.
    pub(crate) fn split_mut(
        &mut self,
    ) -> (Counts<'_>, [&mut PerBlockCumulativeRolling<Sats>; FEATURES]) {
        macro_rules! split {
            (
                scripts: [$($script:ident),*],
                signatures: [$($signature:ident),*],
                others: [$($other:ident),*]
            ) => {
                (
                    Counts([
                        $(&mut self.scripts.$script.count.count,)*
                        $(&mut self.signatures.$signature.count.count,)*
                        $(&mut self.$other.count.count,)*
                        &mut self.segwit.count,
                        &mut self.one_input.count,
                        &mut self.one_output.count,
                    ]),
                    [
                        $(&mut self.scripts.$script.count.value.value,)*
                        $(&mut self.signatures.$signature.count.value.value,)*
                        $(&mut self.$other.count.value.value,)*
                        &mut self.segwit.value.value,
                        &mut self.one_input.value.value,
                        &mut self.one_output.value.value,
                    ],
                )
            };
        }
        with_flagged_features!(split)
    }
}

impl Counts<'_> {
    pub(crate) fn compute(self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        let start = indexer.compute_start().height;
        let vecs = indexer.vecs();
        let source = &vecs.transactions.features;
        macro_rules! sources {
            (
                scripts: [$($script:ident),*],
                signatures: [$($signature:ident),*],
                others: [$($other:ident),*]
            ) => {
                [
                    $(&*source.$script.block,)*
                    $(&*source.$signature.block,)*
                    $(&*source.$other.block,)*
                    &*vecs.blocks.segwit_tx_count,
                    &*source.one_input.block,
                    &*source.one_output.block,
                ]
            };
        }
        for (count, source) in self.0.into_iter().zip(with_flagged_features!(sources)) {
            count.compute_cumulative_transformed(start, source, Count::from, exit)?;
        }
        Ok(())
    }
}
