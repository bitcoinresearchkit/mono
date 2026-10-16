use bitview_plugin_indexer::Indexer;
use bitview_primitives::Count;
use brk_error::Result;
use brk_exit::Exit;

use super::{Vecs, with_windowed_features};

impl Vecs {
    pub(crate) fn compute(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        let start = indexer.compute_start().height;
        let source = &indexer.vecs().transactions.features;
        let this = self;
        this.segwit.compute_cumulative_transformed(
            start,
            &*indexer.vecs().blocks.segwit_tx_count,
            Count::from,
            exit,
        )?;
        macro_rules! compute {
            ($($feature:ident),*) => {$(
                this.$feature.count.compute_cumulative_transformed(
                    start,
                    &*source.$feature.block,
                    Count::from,
                    exit,
                )?;
            )*};
        }
        with_windowed_features!(compute);
        Ok(())
    }
}
