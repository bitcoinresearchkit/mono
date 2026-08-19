use brk_error::Result;

use bitview_plugin::ComputePlugin;
use brk_indexer::Indexer;
use vecdb::Exit;

use super::Vecs;
impl Vecs {
    fn compute_inner(
        &mut self,
        indexer: &Indexer,
        inputs: &bitview_plugin_inputs::Vecs,
        blocks: &bitview_plugin_blocks::Vecs,
        prices: &bitview_plugin_price::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        self.db.sync_bg_tasks()?;

        let starting_lengths = indexer.safe_lengths();

        super::count::compute(&mut self.count, indexer, blocks, exit)?;
        super::value::compute(&mut self.value, indexer, prices, exit)?;
        super::by_type::compute(&mut self.by_type, indexer, exit)?;
        super::unspent::compute(
            &mut self.unspent,
            &self.count,
            &inputs.count,
            &self.by_type,
            &starting_lengths,
            exit,
        )?;
        let lock = super::spent::compute(&mut self.spent, indexer, exit)?;
        self.db.run_bg(move |db| {
            let _lock = lock;
            db.compact_deferred_default()
        });
        Ok(())
    }
}

impl ComputePlugin for Vecs {
    type Dependencies<'a> = crate::Dependencies<'a>;
    type Output = ();

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        exit: &Exit,
    ) -> Result<Self::Output> {
        self.compute_inner(
            dependencies.indexer,
            dependencies.inputs,
            dependencies.blocks,
            dependencies.price,
            exit,
        )
    }
}
