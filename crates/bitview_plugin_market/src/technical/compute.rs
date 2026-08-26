use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use brk_types::{Dollars, PartsPerMillion32};
use rayon::{
    join,
    prelude::{IntoParallelIterator, ParallelIterator},
};
use vecdb::Exit;

use super::{super::moving_average, Vecs, macd, rsi_chain};
use bitview_compute::RatioDollars;

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
        let (rsi, macd) = join(
            || {
                self.rsi
                    .as_mut_array_with_days()
                    .into_par_iter()
                    .try_for_each(|(chain, days)| {
                        rsi_chain::compute(chain, indexer, blocks, 14 * days, 3 * days, exit)
                    })
            },
            || {
                self.macd
                    .as_mut_array_with_days()
                    .into_par_iter()
                    .try_for_each(|(chain, days)| {
                        macd::compute(
                            chain,
                            indexer,
                            blocks,
                            prices,
                            12 * days,
                            26 * days,
                            9 * days,
                            exit,
                        )
                    })
            },
        );
        rsi?;
        macd?;

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
