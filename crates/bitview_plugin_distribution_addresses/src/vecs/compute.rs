use bitview_cohort::AddrTypeId;
use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use brk_types::{Height, Lengths};
use vecdb::{AnyVec, ReadableVec};

use crate::{
    Dependencies,
    compute::{ComputeContext, Workspace, process_chunk},
    live::LiveState,
    state::AddrStates,
};

use super::Vecs;

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn compute(&mut self, deps: Dependencies<'_>, context: UpdateContext<'_>) -> Result<()> {
        let live = self.live.take();
        self.db.sync_bg_tasks()?;
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
            .min(deps.price.spot.cents.height.len())
            .min(
                deps.type_supply
                    .iter()
                    .map(|(_, v)| v.len())
                    .min()
                    .unwrap_or(0),
            );
        let mut start = if changed {
            0
        } else {
            usize::from(deps.indexer.safe_lengths().height)
                .min(end)
                .min(usize::from(self.balances.min_resume_len()))
                .min(usize::from(self.addr_state.min_stamped_len()))
                .min(self.addrs.min_resume_len())
                .min(caps_end)
        };
        let Dependencies {
            indexer,
            mappings,
            input_values,
            price,
            type_supply,
        } = deps;
        let hash = start
            .checked_sub(1)
            .and_then(|h| indexer.vecs().blocks.blockhash.collect_one(Height::from(h)));
        let live =
            live.filter(|s| !changed && s.end == start && s.hash == hash && s.version == version);
        let reuse = live.is_some();
        let LiveState {
            mut addrs,
            mut prices,
            ..
        } = live.unwrap_or_else(|| LiveState {
            end: 0,
            hash: None,
            version,
            addrs: AddrStates::new(),
            prices: Vec::new(),
        });
        if !reuse && start > 0 {
            let current = usize::from(self.addr_state.max_stamped_len()).max(caps_end);
            if start < current {
                let _lock = exit.lock();
                start = self.rollback_state(start)?;
            }
            if start > 0
                && (addrs
                    .import(&self.balances, &self.addrs.funded, Height::from(start))
                    .is_none()
                    || self.restore_caps(&mut addrs).is_none())
            {
                start = 0;
            }
        }
        if start == 0 {
            let _lock = exit.lock();
            self.addr_state.reset()?;
            self.caps.reset()?;
            self.addrs.reset_height()?;
            addrs = AddrStates::new();
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
                &mut addrs,
                indexer,
                mappings,
                &mut workspace,
                &ctx,
                next == end,
                exit,
            )?;
            from = next;
        }
        let starting_lengths = Lengths {
            height: Height::from(start),
            ..Default::default()
        };
        // Derive address metrics from completed per-type sources.
        let type_supply_sats = AddrTypeId::series(|id, _| id.select(&type_supply));
        self.addrs
            .reused
            .compute_rest(&starting_lengths, &type_supply_sats, exit)?;
        self.addrs
            .respent
            .compute_rest(&starting_lengths, &type_supply_sats, exit)?;
        self.addrs
            .exposed
            .compute_rest(&starting_lengths, &type_supply_sats, exit)?;

        let type_funded_addr_counts =
            AddrTypeId::series(|id, _| &id.select(&self.addrs.funded.counts.by_addr_type).height);
        self.addrs.avg_balance.compute(
            &type_supply_sats,
            &type_funded_addr_counts,
            starting_lengths.height,
            exit,
        )?;

        self.addrs.total.compute(
            starting_lengths.height,
            &self.addrs.funded.counts,
            &self.addrs.empty,
            exit,
        )?;

        context.compact_database(&self.db);
        self.live = Some(LiveState {
            end,
            hash: end
                .checked_sub(1)
                .and_then(|h| indexer.vecs().blocks.blockhash.collect_one(Height::from(h))),
            version,
            addrs,
            prices,
        });
        Ok(())
    }
}
