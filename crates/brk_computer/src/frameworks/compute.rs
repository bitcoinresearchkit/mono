use brk_error::Result;
use brk_indexer::Indexer;
use vecdb::Exit;

use super::Vecs;
use crate::{blocks, distribution, indexes, price, supply};

impl Vecs {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        indexes: &indexes::Vecs,
        prices: &price::Vecs,
        blocks: &blocks::Vecs,
        supply: &supply::Vecs,
        distribution: &distribution::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        self.db.sync_bg_tasks()?;

        self.cointime
            .compute(indexer, prices, blocks, supply, distribution, exit)?;
        self.coinflow
            .compute(indexer, indexes, prices, distribution, exit)?;

        let frameworks_exit = exit.clone();
        self.db.run_bg(move |db| {
            let _lock = frameworks_exit.lock();
            db.compact_deferred_default()
        });
        Ok(())
    }
}
