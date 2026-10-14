mod block_loop;
mod write;
pub use block_loop::process_chunk;

use bitview_compute::undo_chunks;
use bitview_distribution::{readers::Columns, replay};
use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use vecdb::Database;

use crate::{Dependencies, Vecs};

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn database(&self) -> Option<&Database> {
        Some(&self.db)
    }

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
                let mut columns = Columns::new(indexer, &output_heights);
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
