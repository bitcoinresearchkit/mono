mod block_loop;
mod write;
pub use block_loop::process_chunk;

use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_distribution_common::{
    readers::Columns,
    replay::{LiveState, tip_hash},
};
use brk_error::Result;
use brk_types::Height;
use vecdb::{AnyVec, Database, ReadableVec};

use crate::{Dependencies, Vecs, state::UTXOStates};

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn database(&self) -> Option<&Database> {
        Some(&self.db)
    }

    fn compute_state(&mut self, deps: Dependencies<'_>, context: UpdateContext<'_>) -> Result<()> {
        let live = self.live.take();
        let version = deps.version();
        let exit = context.exit();
        let resume = {
            let _lock = exit.lock();
            self.validate_state(version)?
        };
        let caps_end = self.caps.end();
        let end = deps
            .indexer
            .vecs()
            .blocks
            .blockhash
            .len()
            .min(deps.price.spot.cents.height.len());
        let mut start = resume.map_or(0, |len| {
            usize::from(deps.indexer.safe_lengths().height)
                .min(end)
                .min(len)
                .min(caps_end)
        });
        let Dependencies {
            indexer,
            mappings,
            input_values,
            price,
        } = deps;
        let hash = tip_hash(indexer, start);
        let live = live.filter(|s| {
            resume.is_some() && s.end == start && s.hash == hash && s.version == version
        });
        let reuse = live.is_some();
        let (mut utxos, mut prices) =
            live.map_or_else(|| (UTXOStates::new(), Vec::new()), |s| (s.state, s.prices));
        if !reuse && start > 0 {
            let current = caps_end;
            if start < current {
                let _lock = exit.lock();
                start = self.rollback_state(start)?;
            }
            if start > 0
                && (utxos.restore(&self.cohorts, Height::from(start)).is_none()
                    || self.restore_caps(&mut utxos).is_none())
            {
                start = 0;
            }
        }
        if start == 0 {
            let _lock = exit.lock();
            self.caps.reset()?;
            utxos = UTXOStates::new();
        }
        prices.truncate(start);
        prices.extend(price.spot.cents.height.collect_range_at(prices.len(), end));
        let output_heights = mappings.output_heights.read();
        let mut columns = Columns::new(indexer, input_values, &output_heights);
        for from in (start..end).step_by(10_000) {
            let next = (from + 10_000).min(end);
            process_chunk(
                self,
                &mut utxos,
                indexer,
                &mut columns,
                from..next,
                &prices,
                next == end,
                exit,
            )?;
        }
        self.live = Some(LiveState {
            end,
            hash: tip_hash(indexer, end),
            version,
            state: utxos,
            prices,
        });
        Ok(())
    }
}
