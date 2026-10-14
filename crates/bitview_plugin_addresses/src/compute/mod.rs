mod block_loop;
mod readers;
mod write;
pub use block_loop::process_chunk;
pub use readers::{AddrReaders, Workspace};

use bitview_distribution::replay;
use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use brk_types::Height;
use vecdb::Database;

use crate::{Dependencies, Vecs};

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn database(&self) -> Option<&Database> {
        Some(&self.db)
    }

    fn compute_state(&mut self, deps: Dependencies<'_>, context: UpdateContext<'_>) -> Result<()> {
        self.discard_unpublished_state();
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
            |vecs, addrs, heights, prices| {
                let output_heights = mappings.output_heights.read();
                let mut workspace = Workspace::new(indexer, &output_heights);
                let mut from = heights.start;
                while from < heights.end {
                    let to = (from + 10_000).min(heights.end);
                    from = process_chunk(
                        vecs,
                        addrs,
                        indexer,
                        mappings,
                        &mut workspace,
                        from..to,
                        prices,
                        to == heights.end,
                        exit,
                    )?;
                }
                // Derive the rest from the members' completed series.
                let from = Height::from(heights.start);
                vecs.all.compute(from, exit)?;
                for member in vecs.types.iter_mut() {
                    member.compute(from, exit)?;
                }
                Ok(())
            },
        )
    }
}
