mod block_loop;
mod write;
pub use block_loop::process_chunk;

use bitview_compute::undo_chunks;
use bitview_distribution::{readers::ColumnReaders, replay};
use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use brk_error::Result;

use crate::{Dependencies, Vecs};

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(&mut self, deps: Dependencies<'_>, context: UpdateContext<'_>) -> Result<()> {
        let version = deps.version();
        let Dependencies {
            indexer,
            mappings,
            price,
        } = deps;
        let exit = context.exit();
        replay::run(
            self,
            version,
            indexer,
            &price.spot.cents.height,
            exit,
            |vecs, utxos, heights, prices| {
                let output_heights = mappings.output_heights.load_full();
                let mut columns = ColumnReaders::new(indexer, &output_heights);
                // An ordinary update is one chunk, the protected tail, written with its undo.
                for (blocks, tail) in undo_chunks(heights) {
                    process_chunk(
                        vecs,
                        utxos,
                        indexer,
                        &mut columns,
                        blocks,
                        prices,
                        tail,
                        exit,
                    )?;
                }
                Ok(())
            },
        )
    }
}
