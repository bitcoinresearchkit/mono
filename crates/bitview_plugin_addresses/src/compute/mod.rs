mod block_loop;
mod readers;
mod write;
pub use block_loop::process_chunk;
pub use readers::{AddrReaders, Workspace};

use bitview_distribution::replay::{LiveState, tip_hash};
use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use brk_types::Height;
use vecdb::{AnyVec, Database, ReadableVec};

use crate::{Dependencies, Vecs, state::AddrStates};

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
                .min(usize::from(self.state.min_stamped_len()))
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
        let (mut addrs, mut prices) =
            live.map_or_else(|| (AddrStates::new(), Vec::new()), |s| (s.state, s.prices));
        if !reuse && start > 0 {
            let current = usize::from(self.state.max_stamped_len()).max(caps_end);
            if start < current {
                let _lock = exit.lock();
                start = self.rollback_state(start)?;
            }
            if start > 0
                && (addrs.restore(&self.balances, Height::from(start)).is_none()
                    || self.restore_caps(&mut addrs).is_none())
            {
                start = 0;
            }
        }
        if start == 0 {
            let _lock = exit.lock();
            self.state.reset()?;
            self.caps.reset()?;
            self.reset_metrics()?;
            addrs = AddrStates::new();
        }
        prices.truncate(start);
        prices.extend(price.spot.cents.height.collect_range_at(prices.len(), end));
        let output_heights = mappings.output_heights.read();
        let mut workspace = Workspace::new(indexer, input_values, &output_heights);
        let mut from = start;
        while from < end {
            let to = (from + 10_000).min(end);
            from = process_chunk(
                self,
                &mut addrs,
                indexer,
                mappings,
                &mut workspace,
                from..to,
                &prices,
                to == end,
                exit,
            )?;
        }
        // Derive the rest from the members' completed series.
        let from = Height::from(start);
        self.all.compute(from, exit)?;
        for member in self.types.iter_mut() {
            member.compute(from, exit)?;
        }

        self.live = Some(LiveState {
            end,
            hash: tip_hash(indexer, end),
            version,
            state: addrs,
            prices,
        });
        Ok(())
    }
}
