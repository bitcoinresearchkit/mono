use bitview_plugin_indexer::Indexer;
use bitview_primitives::Count;
use brk_error::Result;
use brk_exit::Exit;

use super::{Vecs, with_flagged_features};

impl Vecs {
    pub(crate) fn compute(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        let start = indexer.compute_start().height;
        let source = &indexer.vecs().transactions.features;
        self.segwit.compute_cumulative_transformed(
            start,
            &*indexer.vecs().blocks.segwit_tx_count,
            Count::from,
            exit,
        )?;
        macro_rules! compute {
            ($($count:expr, $feature:ident);*) => {$(
                $count.compute_cumulative_transformed(
                    start,
                    &*source.$feature.block,
                    Count::from,
                    exit,
                )?;
            )*};
            (
                scripts: [$($script:ident),*],
                signatures: [$($signature:ident),*],
                others: [$($other:ident),*]
            ) => {
                compute!(
                    $(self.scripts.$script.count, $script);*;
                    $(self.signatures.$signature.count, $signature);*;
                    $(self.$other.count, $other);*
                )
            };
        }
        with_flagged_features!(compute);
        for (count, source) in [
            (&mut self.one_input, &source.one_input.block),
            (&mut self.one_output, &source.one_output.block),
        ] {
            count.compute_cumulative_transformed(start, &**source, Count::from, exit)?;
        }
        Ok(())
    }
}
