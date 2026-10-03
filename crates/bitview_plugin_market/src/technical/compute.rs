use bitview_plugin_blocks::Vecs as BlocksVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_primitives::PartsPerMillion32;
use bitview_transforms::RatioDollars;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Dollars;
use rayon::{
    join,
    prelude::{IntoParallelIterator, ParallelIterator},
};

use super::{super::moving_average, Vecs};

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        prices: &PriceVecs,
        blocks: &BlocksVecs,
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
                        chain.compute(indexer, blocks, 14 * days, 3 * days, exit)
                    })
            },
            || {
                self.macd
                    .as_mut_array_with_days()
                    .into_par_iter()
                    .try_for_each(|(chain, days)| {
                        chain.compute(
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
