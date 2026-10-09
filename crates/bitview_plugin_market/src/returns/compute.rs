use bitview_plugin_blocks::Vecs as BlocksVecs;
use bitview_plugin_indexer::Indexer;
use brk_error::Result;
use brk_exit::Exit;

use super::Vecs;

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        blocks: &BlocksVecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.safe_lengths();

        let _24h_price_return = &self.periods._24h.percent.height;

        self.daily
            .compute(&blocks.lookback, &starting_lengths, exit, _24h_price_return)
    }
}
