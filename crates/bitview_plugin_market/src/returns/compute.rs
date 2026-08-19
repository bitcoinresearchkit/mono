use brk_error::Result;

use brk_indexer::Indexer;
use vecdb::Exit;

use super::Vecs;

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    blocks: &bitview_plugin_blocks::Vecs,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(indexer, blocks, exit)
}

impl Vecs {
    fn compute(
        &mut self,
        indexer: &Indexer,
        blocks: &bitview_plugin_blocks::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.safe_lengths();

        let _24h_price_return_ratio = &self.periods._24h.ratio.height;

        for sd in self.sd_24h.as_mut_array() {
            sd.compute_all(
                &blocks.lookback,
                &starting_lengths,
                exit,
                _24h_price_return_ratio,
            )?;
        }

        Ok(())
    }
}
