use brk_error::Result;
use brk_indexer::Indexer;
use vecdb::Exit;

use super::Vecs;
use crate::{
    distribution::{self, UTXOStates},
    frameworks, indexes, market, price,
};

impl Vecs {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        indexes: &indexes::Vecs,
        prices: &price::Vecs,
        distribution: &distribution::Vecs,
        utxo_states: &UTXOStates,
        frameworks: &frameworks::Vecs,
        moving_average: &market::MovingAverageVecs,
        exit: &Exit,
    ) -> Result<()> {
        self.db.sync_bg_tasks()?;

        self.bedrock.compute(
            indexer,
            indexes,
            distribution,
            utxo_states,
            frameworks,
            exit,
        )?;
        self.capital_sentiment.compute(
            indexer,
            indexes,
            prices,
            distribution,
            moving_average,
            exit,
        )?;
        self.rarity_meter.compute(
            indexer,
            distribution,
            &frameworks.cointime,
            &frameworks.coinflow,
            prices,
            exit,
        )?;

        let models_exit = exit.clone();
        self.db.run_bg(move |db| {
            let _lock = models_exit.lock();
            db.compact_deferred_default()
        });
        Ok(())
    }
}
