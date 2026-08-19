use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use brk_types::{Dollars, PartsPerMillion32};
use vecdb::Exit;

use super::{super::moving_average, Vecs, macd, rsi_chain};
use bitview_compute::{RatioDollars, WindowsTo1m};

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    prices: &bitview_plugin_price::Vecs,
    blocks: &bitview_plugin_blocks::Vecs,
    moving_average: &moving_average::Vecs,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(indexer, prices, blocks, moving_average, exit)
}

impl Vecs {
    fn compute(
        &mut self,
        indexer: &Indexer,
        prices: &bitview_plugin_price::Vecs,
        blocks: &bitview_plugin_blocks::Vecs,
        moving_average: &moving_average::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        for (rsi_chain, &m) in self
            .rsi
            .as_mut_array()
            .into_iter()
            .zip(&WindowsTo1m::<()>::DAYS)
        {
            rsi_chain::compute(rsi_chain, indexer, blocks, 14 * m, 3 * m, exit)?;
        }

        for (macd_chain, &m) in self
            .macd
            .as_mut_array()
            .into_iter()
            .zip(&WindowsTo1m::<()>::DAYS)
        {
            macd::compute(
                macd_chain,
                indexer,
                blocks,
                prices,
                12 * m,
                26 * m,
                9 * m,
                exit,
            )?;
        }

        self.pi_cycle
            .ppm
            .compute_binary::<Dollars, Dollars, RatioDollars<PartsPerMillion32>>(
                starting_height,
                &moving_average.sma._111d.usd.height,
                &moving_average.sma._350d_x2.usd.height,
                exit,
            )?;

        Ok(())
    }
}
