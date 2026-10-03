use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use brk_types::Height;
use vecdb::{AnyVec, Database, ReadableVec};

use crate::{
    Dependencies,
    compute::{ComputeContext, Workspace, process_chunk},
    live::LiveState,
    state::UTXOStates,
};

use super::Vecs;

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn database(&self) -> Option<&Database> {
        Some(&self.db)
    }

    fn compute_state(&mut self, deps: Dependencies<'_>, context: UpdateContext<'_>) -> Result<()> {
        let live = self.live.take();
        let version = deps.version();
        let exit = context.exit();
        let changed = {
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
        let mut start = if changed {
            0
        } else {
            usize::from(deps.indexer.safe_lengths().height)
                .min(end)
                .min(usize::from(self.cohorts.min_resume_len()))
                .min(caps_end)
        };
        let Dependencies {
            indexer,
            mappings,
            input_values,
            price,
        } = deps;
        let hash = start
            .checked_sub(1)
            .and_then(|h| indexer.vecs().blocks.blockhash.collect_one(Height::from(h)));
        let live =
            live.filter(|s| !changed && s.end == start && s.hash == hash && s.version == version);
        let reuse = live.is_some();
        let LiveState {
            mut utxos,
            mut prices,
            ..
        } = live.unwrap_or_else(|| LiveState {
            end: 0,
            hash: None,
            version,
            utxos: UTXOStates::new(),
            prices: Vec::new(),
        });
        if !reuse && start > 0 {
            let current = caps_end;
            if start < current {
                let _lock = exit.lock();
                start = self.rollback_state(start)?;
            }
            if start > 0
                && (utxos.import(&self.cohorts, Height::from(start)).is_none()
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
        let mut workspace = Workspace::new(indexer, input_values, &output_heights);
        let mut from = start;
        while from < end {
            let next = (from + 10_000).min(end);
            let ctx = ComputeContext {
                starting_height: Height::from(from),
                last_height: Height::from(next - 1),
                height_to_price: &prices,
            };
            process_chunk(
                self,
                &mut utxos,
                indexer,
                &mut workspace,
                &ctx,
                next == end,
                exit,
            )?;
            from = next;
        }
        self.live = Some(LiveState {
            end,
            hash: end
                .checked_sub(1)
                .and_then(|h| indexer.vecs().blocks.blockhash.collect_one(Height::from(h))),
            version,
            utxos,
            prices,
        });
        Ok(())
    }
}
