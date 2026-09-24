use std::{mem, path::PathBuf};

use bitview_cohort::{AddrTypeId, AgeRange, CohortContext, EntryPrice};
use bitview_collections::Windows;
use bitview_plugin::{ComputePlugin, ImportContext, Plugin, PluginStorage, UpdateContext};
use bitview_plugin_inputs::ByTypeVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_outputs::ByTypeVecs as OutputsByTypeVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_traversable::Traversable;
use bitview_urpd::{AgeBoundsMetrics, AgeCutoffs};
use bitview_vecs::{DailyMappings, LazyWindowStartVec, PerBlockCumulativeRolling};
use brk_error::Result;
use brk_oracle::VERSION as ORACLE_VERSION;
use brk_types::{Cents, Date, Height, StoredF64, SupplyState, Version};
use tracing::{debug, info, warn};
use vecdb::{
    AnyVec, BytesVec, Database, ImportOptions, ImportableVec, ReadableCloneableVec, ReadableVec,
    Rw, Stamp, StorageMode, WritableVec,
};

use super::{
    AddrStateVecs, AllChainSources, CohortMetrics, UTXOStates,
    addr::{
        AddrActivityVecs, AddrCountsVecs, AddrVecs, AvgAmountVecs, DeltaVecs, ExposedAddrVecs,
        FundedAddrCountsVecs, NewAddrCountVecs, ReusedAddrVecs, TotalAddrCountVecs,
    },
    inner::Inner,
};
use crate::{
    Dependencies, STORAGE,
    compute::{ComputeContext, StartMode, determine_start_mode, process_blocks},
    state::{AddrStates, BlockState},
};

const COMPUTE_VERSION: Version = Version::new(30 + ORACLE_VERSION);
const COINDAYS_CREATED_VERSION: Version = Version::ONE;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    inner: M::WriteOnly<Inner>,
    #[traversable(skip)]
    pub states_path: PathBuf,

    #[traversable(wrap = "supply", rename = "state")]
    /// Serialized distribution state used to resume cohort computation at each
    /// block height.
    pub supply_state: M::Stored<BytesVec<Height, SupplyState>>,
    #[traversable(wrap = "addrs", rename = "state")]
    /// Persistent state for every indexed address. Each address type uses a
    /// compact primary vector; funded addresses and empty addresses whose
    /// lifetime totals do not fit inline reference shared sidecars.
    pub addr_state: AddrStateVecs<M>,
    pub cohorts: CohortMetrics<M>,
    #[traversable(wrap = "cohorts/urpd")]
    pub age_bounds: AgeBoundsMetrics<M>,
    // Computed with distribution and presented beside cointime's age-range metrics.
    /// Coin days accrued by unspent supply between block timestamps, allocated
    /// to the age range in which they accrue. One coin day is one BTC remaining
    /// unspent for one day.
    #[traversable(wrap = "cointime/age_range")]
    pub coindays_created: AgeRange<PerBlockCumulativeRolling<StoredF64, M>>,
    #[traversable(wrap = "cointime/activity")]
    /// Coin blocks destroyed by spent outputs: each spent output's value in
    /// BTC multiplied by its age in blocks, summed over the represented block.
    pub coinblocks_destroyed: PerBlockCumulativeRolling<StoredF64, M>,
    pub addrs: AddrVecs<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}

const SAVED_STAMPED_CHANGES: u16 = 10;

impl Vecs {
    pub fn all_chain_sources(&self) -> AllChainSources {
        AllChainSources::new(self.cohorts.all_supply(), self.cohorts.all_market_cap())
    }

    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        prices: &PriceVecs,
        inputs_by_type: &ByTypeVecs,
        outputs_by_type: &OutputsByTypeVecs,
    ) -> Result<Self> {
        let db_path = STORAGE.path(context);
        let states_path = db_path.join("states");
        let db = STORAGE.open_database(context, 20_000_000)?;

        let version = STORAGE.schema_version();
        let spot_price = prices.spot.cents.height.read_only_boxed_clone();

        let cohorts =
            CohortMetrics::forced_import(&db, version, mappings, window_starts, &spot_price)?;

        let addr_state = AddrStateVecs::forced_import(&db, version)?;

        let funded_addr_count =
            FundedAddrCountsVecs::forced_import(&db, version, mappings, window_starts)?;
        let empty_addr_count =
            AddrCountsVecs::forced_import(&db, "empty_addr_count", version, mappings)?;
        let addr_activity = AddrActivityVecs::forced_import(&db, version, mappings, window_starts)?;

        // Stored total = addr_count + empty_addr_count (global + per-type, with all derived mappings)
        let total_addr_count = TotalAddrCountVecs::forced_import(&db, version, mappings)?;

        // Per-block delta of total (global + per-type)
        let new_addr_count =
            NewAddrCountVecs::new(version, &total_addr_count, mappings, window_starts);

        // Reused address tracking (counts + per-block uses + percent).
        // `reused_*` uses the receive-side predicate (funded_txo_count > 1,
        // industry standard). `respent_*` uses the spend-side counterpart
        // (spent_txo_count > 1, strictly more restrictive).
        let reused_addr_count = ReusedAddrVecs::forced_import(
            &db,
            "reused",
            version,
            mappings,
            window_starts,
            &spot_price,
            outputs_by_type,
            inputs_by_type,
            cohorts.all_supply(),
        )?;
        let respent_addr_count = ReusedAddrVecs::forced_import(
            &db,
            "respent",
            version,
            mappings,
            window_starts,
            &spot_price,
            outputs_by_type,
            inputs_by_type,
            cohorts.all_supply(),
        )?;

        // Exposed address tracking (counts + supply) - quantum / pubkey-exposure sense
        let exposed_addr_vecs = ExposedAddrVecs::forced_import(
            &db,
            version,
            mappings,
            &spot_price,
            cohorts.all_supply(),
        )?;

        // Growth rate: delta change + rate (global + per-type)
        let delta = DeltaVecs::new(version, &funded_addr_count.counts, window_starts, mappings);

        // Average amount (supply / utxo_count, supply / funded_addr_count) for `all` and per addr type.
        let all_chain = AllChainSources::new(cohorts.all_supply(), cohorts.all_market_cap());
        let avg_amount = AvgAmountVecs::forced_import(
            &db,
            version,
            mappings,
            &spot_price,
            &all_chain,
            &cohorts.outputs.unspent_count.cohorts.utxo.all.height,
            &funded_addr_count.counts.all.height,
        )?;

        let age_bounds =
            AgeBoundsMetrics::forced_import(&db, version, &DailyMappings::new(mappings))?;
        let this = Self {
            supply_state: BytesVec::forced_import_with(
                ImportOptions::new(&db, "supply_state", version)
                    .with_saved_stamped_changes(SAVED_STAMPED_CHANGES),
            )?,

            addrs: AddrVecs {
                funded: funded_addr_count,
                empty: empty_addr_count,
                activity: addr_activity,
                total: total_addr_count,
                new: new_addr_count,
                reused: reused_addr_count,
                respent: respent_addr_count,
                exposed: exposed_addr_vecs,
                delta,
                avg_amount,
            },

            cohorts,
            age_bounds,

            coindays_created: AgeRange::try_from_fn(|id| {
                PerBlockCumulativeRolling::forced_import(
                    &db,
                    &format!(
                        "{}_coindays_created",
                        CohortContext::Utxo.full_name(id.cohort())
                    ),
                    version + COINDAYS_CREATED_VERSION + Version::ONE,
                    mappings,
                    window_starts,
                )
            })?,

            coinblocks_destroyed: PerBlockCumulativeRolling::forced_import(
                &db,
                "coinblocks_destroyed",
                version + Version::TWO,
                mappings,
                window_starts,
            )?,

            addr_state,
            db,
            inner: Inner::default(),
            states_path,
        };

        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }

    /// Reset in-memory caches that become stale after rollback.
    fn reset_in_memory_caches(&mut self) {
        self.inner.reset();
    }

    fn reset_state(
        &mut self,
        utxo_states: &mut UTXOStates,
        addr_states: &mut AddrStates,
    ) -> Result<()> {
        self.supply_state.reset()?;
        self.addrs.reset_height()?;
        self.addr_state.reset()?;
        utxo_states.reset()?;
        addr_states.reset()?;
        Ok(())
    }

    fn min_resume_len(&self) -> Height {
        self.cohorts
            .min_resume_len()
            .min(Height::from(self.supply_state.len()))
            .min(self.addr_state.min_stamped_len())
            .min(Height::from(self.addrs.min_resume_len()))
            .min(Height::from(
                self.coindays_created
                    .iter()
                    .map(|v| v.cumulative.height.len())
                    .min()
                    .unwrap_or_default(),
            ))
            .min(Height::from(self.coinblocks_destroyed.block.len()))
    }
}

pub fn flush(vecs: &Vecs) -> Result<()> {
    vecs.db.flush()?;
    Ok(())
}

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;
    type Output = UTXOStates;

    /// Main computation loop.
    ///
    /// Processes blocks to compute UTXO and address cohort metrics:
    /// 1. Recovers state from checkpoints or starts fresh
    /// 2. Iterates through blocks, processing outputs/inputs in parallel
    /// 3. Flushes checkpoints periodically
    /// 4. Computes aggregate cohorts from separate cohorts
    /// 5. Computes derived metrics
    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<Self::Output> {
        let Dependencies {
            indexer,
            mappings,
            inputs,
            outputs,
            transactions,
            price: prices,
        } = dependencies;
        let exit = context.exit();
        self.db.sync_bg_tasks()?;
        let mut utxo_states = UTXOStates::new(&self.states_path);
        let mut addr_states = AddrStates::new(&self.states_path);

        let base_version = COMPUTE_VERSION
            + [
                prices.spot.cents.height.version(),
                mappings.timestamp.monotonic.version(),
                indexer.vecs().transactions.first_tx_index.version(),
                indexer.vecs().outputs.first_txout_index.version(),
                indexer.vecs().inputs.first_txin_index.version(),
                transactions.count.total.block.version(),
                outputs.count.total.sum.version(),
                inputs.count.sum.version(),
                mappings.tx_index.output_count.version(),
                mappings.tx_index.input_count.version(),
                indexer.vecs().outputs.value.version(),
                indexer.vecs().outputs.output_type.version(),
                indexer.vecs().outputs.type_index.version(),
                inputs.value.version(),
                indexer.vecs().inputs.outpoint.version(),
                indexer.vecs().inputs.output_type.version(),
                indexer.vecs().inputs.type_index.version(),
                indexer.vecs().addrs.p2pk65.first_index.version(),
                indexer.vecs().addrs.p2pk33.first_index.version(),
                indexer.vecs().addrs.p2pkh.first_index.version(),
                indexer.vecs().addrs.p2sh.first_index.version(),
                indexer.vecs().addrs.p2wpkh.first_index.version(),
                indexer.vecs().addrs.p2wsh.first_index.version(),
                indexer.vecs().addrs.p2tr.first_index.version(),
                indexer.vecs().addrs.p2a.first_index.version(),
            ]
            .into_iter()
            .sum::<Version>();

        debug!("validating computed versions");
        self.supply_state
            .validate_computed_version_or_reset(base_version)?;
        self.cohorts.validate_computed_versions(base_version)?;
        for vec in self.coindays_created.iter_mut() {
            vec.validate_computed_version_or_reset(base_version)?;
        }
        debug!("computed versions validated");

        let starting_lengths = indexer.safe_lengths();

        // 1. Find the height from which the block loop can safely resume.
        let current_height = Height::from(self.supply_state.len());
        let min_resume_len = self.min_resume_len();

        // 2. Determine start mode and recover/reset state
        // Clamp to starting_lengths.height to handle reorg (indexer may require earlier start)
        let resume_target = current_height.min(starting_lengths.height);
        if resume_target < current_height {
            warn!(
                "Reorg detected: rolling back from {} to {}",
                current_height, resume_target
            );
        }
        let start_mode = determine_start_mode(min_resume_len, resume_target);

        // Try to resume from checkpoint, fall back to fresh start if needed
        let recovered_height = match start_mode {
            StartMode::Resume(height) => {
                // Roll back only on a reorg. A clean resume has nothing to undo, and an
                // interrupted run wrote no rollback metadata (periodic flushes use
                // with_changes=false; only the final write creates the `changes/` dir),
                // so `rollback_before` would fail with `NotFound`.
                let chain_state_rollback = (height < current_height)
                    .then(|| self.supply_state.rollback_before(Stamp::from(height)));

                let recovered = self.recover_state(
                    height,
                    chain_state_rollback,
                    &mut utxo_states,
                    &mut addr_states,
                )?;

                debug!("recover_state completed, starting_height={}", recovered);
                recovered
            }
            StartMode::Fresh => Height::ZERO,
        };

        debug!("recovered_height={}", recovered_height);

        let needs_fresh_start = recovered_height.is_zero();
        let needs_rollback = recovered_height < current_height;

        if needs_fresh_start || needs_rollback {
            self.reset_in_memory_caches();
        }

        if needs_fresh_start {
            self.reset_state(&mut utxo_states, &mut addr_states)?;
            info!("Building distribution history from genesis...");
        }

        // Populate price/timestamp caches from the prices module.
        // Must happen AFTER rollback/reset (which invalidates caches) but BEFORE
        // chain_state rebuild (which reads from them).
        let cache_target_len = prices
            .spot
            .cents
            .height
            .len()
            .min(mappings.timestamp.monotonic.len());
        let cache_current_len = self.inner.prices.len();
        if cache_target_len < cache_current_len {
            self.inner.prices.truncate(cache_target_len);
            self.inner.timestamps.truncate(cache_target_len);
            self.inner.price_range_max.truncate(cache_target_len);
        } else if cache_target_len > cache_current_len {
            let new_prices = prices
                .spot
                .cents
                .height
                .collect_range_at(cache_current_len, cache_target_len);
            let new_timestamps = mappings
                .timestamp
                .monotonic
                .collect_range_at(cache_current_len, cache_target_len);
            self.inner.prices.extend(new_prices);
            self.inner.timestamps.extend(new_timestamps);
        }
        self.inner.price_range_max.extend(&self.inner.prices);

        // Take chain_state out of self to avoid borrow conflicts
        let mut chain_state = mem::take(&mut self.inner.chain_state);

        // Recover or reuse chain_state
        let starting_height = if recovered_height.is_zero() {
            Height::ZERO
        } else if chain_state.len() == usize::from(recovered_height) {
            // Normal resume: chain_state already matches, reuse as-is
            debug!(
                "reusing in-memory chain_state ({} entries)",
                chain_state.len()
            );
            recovered_height
        } else {
            debug!("rebuilding chain_state from stored values");

            let end = usize::from(recovered_height);
            debug!("building supply_state vec for {} heights", recovered_height);
            let supply_state_data: Vec<_> = self.supply_state.collect_range_at(0, end);
            let capitalized_price_data: Vec<_> = self
                .cohorts
                .realized
                .capitalized_price
                .series
                .all
                .cents
                .height
                .collect_range_at(0, end);

            let mut entry_anchor = Cents::ZERO;
            chain_state = supply_state_data
                .into_iter()
                .enumerate()
                .map(|(h, supply)| {
                    let price = self.inner.prices[h];
                    let entry = EntryPrice::from_is_discount(
                        entry_anchor == Cents::ZERO || price <= entry_anchor,
                    );
                    entry_anchor = capitalized_price_data[h];

                    BlockState {
                        supply,
                        entry,
                        price,
                        timestamp: self.inner.timestamps[h],
                    }
                })
                .collect();
            debug!("chain_state rebuilt");

            recovered_height
        };

        // 3. Get last height from indexer
        let last_height = Height::from(indexer.vecs().blocks.blockhash.len().saturating_sub(1));
        debug!(
            "last_height={}, starting_height={}",
            last_height, starting_height
        );

        // Invalidate snapshots from the recovered day. If the next block belongs
        // to a new day (or the chain ends here), republish the recovered day now.
        let previous_date = usize::from(starting_height)
            .checked_sub(1)
            .and_then(|height| self.inner.timestamps.get(height))
            .copied()
            .map(Date::from);
        let next_date = self
            .inner
            .timestamps
            .get(usize::from(starting_height))
            .copied()
            .map(Date::from);
        utxo_states.prepare_urpds(&self.states_path, previous_date, next_date)?;

        // 4. Process blocks
        if starting_height <= last_height {
            debug!("calling process_blocks");

            let prices = mem::take(&mut self.inner.prices);
            let timestamps = mem::take(&mut self.inner.timestamps);
            let price_range_max = mem::take(&mut self.inner.price_range_max);
            let compute = ComputeContext {
                starting_height,
                last_height,
                height_to_timestamp: &timestamps,
                height_to_price: &prices,
                price_range_max: &price_range_max,
            };
            let entry_anchor = starting_height
                .decremented()
                .and_then(|height| {
                    self.cohorts
                        .realized
                        .capitalized_price
                        .series
                        .all
                        .cents
                        .height
                        .collect_one(height)
                })
                .unwrap_or(Cents::ZERO);

            process_blocks(
                self,
                &mut utxo_states,
                &mut addr_states,
                indexer,
                mappings,
                inputs,
                outputs,
                transactions,
                &compute,
                &mut chain_state,
                entry_anchor,
                exit,
            )?;

            self.inner.prices = prices;
            self.inner.timestamps = timestamps;
            self.inner.price_range_max = price_range_max;
        }

        // Put chain_state back
        self.inner.chain_state = chain_state;

        // 5. Compute rest part1 (day1 mappings)
        debug!("Computing rest part 1...");
        self.cohorts.compute_rest_part1(&starting_lengths, exit)?;

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

        // 7. Compute rest part2 (relative metrics)
        debug!("Computing rest part 2...");
        self.cohorts.compute_rest_part2(&starting_lengths, exit)?;

        let from = mappings
            .height
            .recompute_day(starting_lengths.height)
            .map(usize::from)
            .unwrap_or_default();
        self.age_bounds.compute(
            self.supply_state.version(),
            from,
            &mappings.day1.date,
            |day, date| {
                utxo_states.with_urpd_entries(
                    &self.states_path,
                    date,
                    usize::from(day) + 1 == mappings.day1.date.len(),
                    |entries| AgeCutoffs::from_age_entries(entries),
                )
            },
            exit,
        )?;
        context.compact_database(&self.db);
        Ok(utxo_states)
    }
}
