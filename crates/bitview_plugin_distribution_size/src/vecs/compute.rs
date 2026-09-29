use bitview_cohort::AddrTypeId;
use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use brk_types::{Height, Lengths};
use vecdb::{AnyVec, ReadableVec, WritableVec};

use crate::{
    Dependencies,
    compute::{ComputeContext, process_chunk},
    live::LiveState,
    state::{AddrStates, UTXOStates},
};

use super::Vecs;

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn compute(&mut self, deps: Dependencies<'_>, context: UpdateContext<'_>) -> Result<()> {
        let live = self.live.take();
        self.db.sync_bg_tasks()?;
        let version = deps.version();
        let changed = self.validate_state(version)?;
        let caps_end = self.cap_checkpoint_len();
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
                .min(usize::from(self.addr_state.min_stamped_len()))
                .min(self.addrs.min_resume_len())
                .min(caps_end)
        };
        let Dependencies {
            indexer,
            mappings,
            input_values,
            price,
        } = deps;
        let exit = context.exit();
        let hash = start
            .checked_sub(1)
            .and_then(|h| indexer.vecs().blocks.blockhash.collect_one(Height::from(h)));
        let live =
            live.filter(|s| !changed && s.end == start && s.hash == hash && s.version == version);
        let reuse = live.is_some();
        let LiveState {
            mut utxos,
            mut addrs,
            mut prices,
            ..
        } = live.unwrap_or_else(|| LiveState {
            end: 0,
            hash: None,
            version,
            utxos: UTXOStates::new(),
            addrs: AddrStates::new(),
            prices: Vec::new(),
        });
        if !reuse && start > 0 {
            let current = usize::from(self.addr_state.max_stamped_len()).max(caps_end);
            if start < current {
                start = self.rollback_state(start).unwrap_or(0);
            }
            if start > 0
                && (utxos.import(&self.cohorts, Height::from(start)).is_none()
                    || addrs
                        .import(&self.cohorts, &self.addrs.funded, Height::from(start))
                        .is_none()
                    || self.restore_caps(&mut utxos, &mut addrs).is_none())
            {
                start = 0;
            }
        }
        if start == 0 {
            self.addr_state.reset()?;
            self.caps.reset()?;
            self.addrs.reset_height()?;
            utxos = UTXOStates::new();
            addrs = AddrStates::new();
        }
        prices.truncate(start);
        prices.extend(price.spot.cents.height.collect_range_at(prices.len(), end));
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
                &mut addrs,
                indexer,
                mappings,
                input_values,
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
        // 6b. Compute address metrics derived from stored per-type sources.
        let type_supply = &self.cohorts.supply.total.cohorts.utxo.type_;
        let type_outputs = &self.cohorts.outputs.unspent_count.cohorts.utxo.type_;
        let type_supply_sats =
            AddrTypeId::series(|id, _| &type_supply.get(id.output_type()).sats.height);
        let type_utxo_counts =
            AddrTypeId::series(|id, _| &type_outputs.get(id.output_type()).height);
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
        self.addrs.avg_amount.compute(
            &type_supply_sats,
            &type_utxo_counts,
            &type_funded_addr_counts,
            starting_lengths.height,
            exit,
        )?;

        // 6c. Compute total_addr_count = addr_count + empty_addr_count
        self.addrs.total.compute(
            starting_lengths.height,
            &self.addrs.funded.counts,
            &self.addrs.empty,
            exit,
        )?;

        context.compact_database(&self.db);
        self.db.sync_bg_tasks()?;
        self.live = Some(LiveState {
            end,
            hash: end
                .checked_sub(1)
                .and_then(|h| indexer.vecs().blocks.blockhash.collect_one(Height::from(h))),
            version,
            utxos,
            addrs,
            prices,
        });
        Ok(())
    }
}
