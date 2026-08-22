use brk_error::Result;

use bitview_cohort::{AddrTypeId, ByAddrType};
use bitview_plugin_indexer::Lengths;
use bitview_traversable::Traversable;
use brk_types::{PartsPerMillion32, StoredF32, StoredU32, StoredU64, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, AnyVec, Database, Exit, Rw, StorageMode, WritableVec};

use bitview_compute::{
    CachedBlockCountReader, CachedWindowStartVec, ColumnarPerBlockCumulativeRolling,
    CountPerBlockRollingAverage, LazyColumnPerBlockCumulativeRolling,
    LazyPerBlockCumulativeRolling, LazyPercentCumulativeRolling, PerBlockRollingAverage, RatioU64,
    Windows, WithAddrTypes,
};

use super::state::AddrTypeToAddrEventCount;

/// Per-block reused-address event metrics. Holds three families of
/// signals: output-level (use), input-level (spend), and address-level
/// (active in block).
///
/// `output_to_reused_addr_count`: every output landing on an address that had
/// already received at least one prior output anywhere in its lifetime,
/// i.e. an output-level reuse event. Outputs are not deduplicated per
/// address within a block: an address receiving N outputs in one block
/// that had `before` lifetime outputs contributes
/// `max(0, N - max(0, 1 - before))` events. Only the very first output
/// an address ever sees is excluded. Every subsequent output counts,
/// matching the standard "% of outputs to previously-used addresses"
/// reuse ratio reported by external sources. `output_to_reused_addr_share`
/// uses `bitview_plugin_outputs::ByTypeVecs::output_count` (all 12 output types) as
/// denominator. `spendable_output_to_reused_addr_share` uses the
/// op_return-excluded 11-type aggregate (`spendable_output_count`).
///
/// `input_from_reused_addr_count`: every input spending from an address
/// whose lifetime `funded_txo_count > 1` at the time of the spend (i.e.
/// the address is in the same reused set tracked by
/// `reused_addr_count`). Every input is checked independently. If a
/// single address has multiple inputs in one block each one counts.
/// This is a *stable-predicate* signal about the sending address, not
/// an output-level repeat event: the first spend from a reused address
/// counts just as much as the tenth. Denominator
/// (`input_from_reused_addr_share`): `bitview_plugin_inputs::ByTypeVecs::input_count` (11
/// spendable types, where `p2ms`, `unknown`, `empty` count as true
/// negatives).
///
/// `active_reused_addr_count` / `active_reused_addr_share`: block-level
/// *address* signals (single aggregate, not per-type).
/// `active_reused_addr_count` is the count of distinct addresses
/// involved in this block (sent ∪ received) that satisfy `is_reused()`
/// after the block's events, populated inline in `process_received`
/// (each receiver, post-receive) and in `process_sent` (each
/// first-encounter sender, deduped against `received_addrs` so
/// addresses that did both aren't double-counted).
/// `active_reused_addr_share` is the per-block ratio
/// `reused / active * 100` as a percentage in `[0, 100]` (or `0.0` for
/// empty blocks). The denominator (distinct active addrs per block)
/// lives at `addrs.activity.active.all`,
/// derived from `sending + receiving - bidirectional`. Both fields
/// expose lazy 24h/1w/1m/1y rolling *averages* of the per-block values.
/// Sums and cumulatives of distinct-address counts would be misleading
/// because the same address can appear in multiple blocks, so the
/// cumulative count remains an internal source for the lazy views.
#[derive(Traversable)]
pub struct AddrEventsVecs<M: StorageMode = Rw> {
    /// Outputs classified by an address-event rule. Reuse counts
    /// every output after an address's first lifetime receive; respending counts
    /// outputs to addresses with more than one prior lifetime spend. Multiple
    /// qualifying outputs to one address are counted separately.
    pub output_to_reused_addr_count: ColumnarPerBlockCumulativeRolling<
        StoredU64,
        AddrTypeId,
        WithAddrTypes<
            LazyColumnPerBlockCumulativeRolling<StoredU64, AddrTypeId>,
            LazyPerBlockCumulativeRolling<StoredU64>,
        >,
        M,
    >,
    /// Share of outputs classified by an address-event rule, using
    /// the matching output type as denominator.
    pub output_to_reused_addr_share: WithAddrTypes<LazyPercentCumulativeRolling<PartsPerMillion32>>,
    /// Share of spendable outputs classified by an address-event
    /// rule; `OP_RETURN` outputs are excluded from the denominator.
    pub spendable_output_to_reused_addr_share: LazyPercentCumulativeRolling<PartsPerMillion32>,
    /// Inputs spending from addresses that satisfied an address predicate
    /// before that input: more than one prior lifetime receive for reuse, or
    /// more than one prior lifetime spend for respending. Multiple qualifying
    /// inputs from one address are counted separately.
    pub input_from_reused_addr_count: ColumnarPerBlockCumulativeRolling<
        StoredU64,
        AddrTypeId,
        WithAddrTypes<
            LazyColumnPerBlockCumulativeRolling<StoredU64, AddrTypeId>,
            LazyPerBlockCumulativeRolling<StoredU64>,
        >,
        M,
    >,
    /// Share of inputs spending from addresses that satisfy an address
    /// predicate, using the matching input type as denominator.
    pub input_from_reused_addr_share:
        WithAddrTypes<LazyPercentCumulativeRolling<PartsPerMillion32>>,
    /// Distinct active addresses in the represented block that satisfy the
    /// address predicate after that block's events.
    pub active_reused_addr_count: CountPerBlockRollingAverage<M>,
    /// Share of distinct active addresses in the represented block that
    /// satisfy an address predicate after that block's events.
    pub active_reused_addr_share: PerBlockRollingAverage<StoredF32, StoredF32, M>,
}

impl AddrEventsVecs {
    fn event_shares(
        name: &str,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        all: LazyPercentCumulativeRolling<PartsPerMillion32>,
        numerators: &ByAddrType<LazyColumnPerBlockCumulativeRolling<StoredU64, AddrTypeId>>,
        denominators: &ByAddrType<CachedBlockCountReader>,
    ) -> WithAddrTypes<LazyPercentCumulativeRolling<PartsPerMillion32>> {
        let by_addr_type = AddrTypeId::series(|column, type_name| {
            LazyPercentCumulativeRolling::from_cached_block_count(
                &format!("{type_name}_{name}"),
                version,
                &column.select(numerators).cumulative.height,
                column.select(denominators).clone(),
                cached_starts,
                mappings,
            )
        });
        WithAddrTypes { all, by_addr_type }
    }
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        outputs_by_type: &bitview_plugin_outputs::ByTypeVecs,
        inputs_by_type: &bitview_plugin_inputs::ByTypeVecs,
    ) -> Result<Self> {
        let import_count = |name: &str| {
            ColumnarPerBlockCumulativeRolling::forced_import(
                db,
                &format!("{name}_by_type_cumulative"),
                version,
                |source| {
                    WithAddrTypes::from_columnar_cumulative_source(
                        name,
                        version,
                        source,
                        mappings,
                        cached_starts,
                    )
                },
            )
        };

        let output_to_reused_addr_count = import_count(&format!("output_to_{name}_addr_count"))?;
        let output_share_name = format!("output_to_{name}_addr_share");
        let output_denominators = outputs_by_type.output_count.cached_addr_type_counts();
        let output_to_reused_addr_share = Self::event_shares(
            &output_share_name,
            version,
            mappings,
            cached_starts,
            outputs_by_type.output_count.lazy_share(
                &output_share_name,
                version,
                &output_to_reused_addr_count.all.cumulative.height,
                cached_starts,
                mappings,
            ),
            &output_to_reused_addr_count.by_addr_type,
            &output_denominators,
        );
        let spendable_share_name = format!("spendable_output_to_{name}_addr_share");
        let spendable_output_to_reused_addr_share =
            LazyPercentCumulativeRolling::from_cumulative_ratio::<
                StoredU64,
                StoredU64,
                RatioU64<PartsPerMillion32>,
            >(
                &spendable_share_name,
                version,
                &output_to_reused_addr_count.all.cumulative.height,
                outputs_by_type.spendable_output_count.cached_cumulative(),
                cached_starts,
                mappings,
            );
        let input_from_reused_addr_count = import_count(&format!("input_from_{name}_addr_count"))?;
        let input_share_name = format!("input_from_{name}_addr_share");
        let input_denominators = inputs_by_type.input_count.cached_addr_type_counts();
        let input_from_reused_addr_share = Self::event_shares(
            &input_share_name,
            version,
            mappings,
            cached_starts,
            inputs_by_type.input_count.lazy_share(
                &input_share_name,
                version,
                &input_from_reused_addr_count.all.cumulative.height,
                cached_starts,
                mappings,
            ),
            &input_from_reused_addr_count.by_addr_type,
            &input_denominators,
        );

        let active_reused_addr_count = CountPerBlockRollingAverage::forced_import(
            db,
            &format!("active_{name}_addr_count"),
            version,
            mappings,
            cached_starts,
        )?;
        let active_reused_addr_share = PerBlockRollingAverage::forced_import(
            db,
            &format!("active_{name}_addr_share"),
            version,
            mappings,
            cached_starts,
        )?;

        Ok(Self {
            output_to_reused_addr_count,
            output_to_reused_addr_share,
            spendable_output_to_reused_addr_share,
            input_from_reused_addr_count,
            input_from_reused_addr_share,
            active_reused_addr_count,
            active_reused_addr_share,
        })
    }

    pub fn min_resume_len(&self) -> usize {
        self.output_to_reused_addr_count
            .cumulative
            .len()
            .min(self.input_from_reused_addr_count.cumulative.len())
            .min(self.active_reused_addr_count.block.len())
            .min(self.active_reused_addr_share.block.len())
    }

    pub fn par_iter_height_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        rayon::iter::once(self.output_to_reused_addr_count.stored_mut())
            .chain(rayon::iter::once(
                self.input_from_reused_addr_count.stored_mut(),
            ))
            .chain([
                self.active_reused_addr_count.stored_mut(),
                &mut self.active_reused_addr_share.block as &mut dyn AnyStoredVec,
            ])
    }

    pub fn reset_height(&mut self) -> Result<()> {
        self.output_to_reused_addr_count.reset()?;
        self.input_from_reused_addr_count.reset()?;
        self.active_reused_addr_count.reset()?;
        self.active_reused_addr_share.block.reset()?;
        Ok(())
    }

    #[inline(always)]
    pub fn push_height(
        &mut self,
        uses: &AddrTypeToAddrEventCount,
        spends: &AddrTypeToAddrEventCount,
        active_addr_count: u32,
        active_reused_addr_count: u32,
    ) {
        self.output_to_reused_addr_count.push_block(uses.row());
        self.input_from_reused_addr_count.push_block(spends.row());
        self.active_reused_addr_count
            .push_block(StoredU32::from(active_reused_addr_count));
        // Stored as a percentage in [0, 100] to match the rest of the
        // codebase (Unit.percentage on the website expects 0..100). The
        // `active_addr_count` denominator lives at
        // `addrs.activity.active.all`, passed in here so we can
        // compute the per-block ratio inline.
        let share = if active_addr_count > 0 {
            100.0 * (active_reused_addr_count as f32 / active_addr_count as f32)
        } else {
            0.0
        };
        self.active_reused_addr_share
            .block
            .push(StoredF32::from(share));
    }

    pub fn compute_rest(&mut self, starting_lengths: &Lengths, exit: &Exit) -> Result<()> {
        self.active_reused_addr_share
            .compute_rest(starting_lengths.height, exit)?;
        Ok(())
    }
}
