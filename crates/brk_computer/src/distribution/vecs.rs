use std::{
    mem,
    path::{Path, PathBuf},
};

use brk_cohort::{AddrTypeId, AgeRange, AgeRangeId, CohortContext, EntryPrice};
use brk_error::Result;
use brk_indexer::Indexer;
use brk_plugin::{Plugin, PluginGate};
use brk_traversable::Traversable;
use brk_types::{Cents, Height, StoredF64, SupplyState, Version};
use tracing::{debug, info};
use vecdb::{
    AnyVec, BytesVec, Exit, ImportOptions, ImportableVec, LazyVec, OverflowVec,
    ReadableCloneableVec, ReadableVec, Rw, Stamp, StorageMode, WritableVec,
};

use crate::{
    distribution::{
        compute::{StartMode, determine_start_mode, process_blocks, reset_state},
        state::{AddrStates, BlockState},
    },
    indexes, inputs,
    internal::{
        CachedWindowStartVec, ColumnarPerBlockCumulativeRolling,
        LazyColumnPerBlockCumulativeRolling, PerBlockCumulativeRolling, Windows,
        db_utils::{finalize_db, open_db},
    },
    outputs, price, transactions,
};

use super::inner::Inner;
use super::{
    AddrsDataVecs, AllChainSources, AnyAddrIndexesVecs, CohortMetrics, DB_NAME, UTXOStates,
    addr::{
        AddrActivityVecs, AddrCountsVecs, AddrVecs, AvgAmountVecs, DeltaVecs, ExposedAddrVecs,
        FundedAddrCountsVecs, NewAddrCountVecs, ReusedAddrVecs, TotalAddrCountVecs,
    },
};

const VERSION: Version = Version::new(30 + brk_oracle::VERSION);
const COINDAYS_CREATED_VERSION: Version = Version::ONE;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    pub(crate) plugin_gate: PluginGate,
    #[traversable(skip)]
    inner: Inner,
    #[traversable(skip)]
    pub states_path: PathBuf,

    #[traversable(wrap = "supply", rename = "state")]
    /// Serialized distribution state used to resume cohort computation at each
    /// block height.
    pub supply_state: M::Stored<BytesVec<Height, SupplyState>>,
    #[traversable(wrap = "addrs", rename = "indexes")]
    pub any_addr_indexes: AnyAddrIndexesVecs<M>,
    #[traversable(wrap = "addrs", rename = "data")]
    pub addrs_data: AddrsDataVecs<M>,
    #[traversable(wrap = "cohorts")]
    pub cohorts: CohortMetrics<M>,
    // Computed and stored with distribution, but presented beside the other
    // age-range cointime series to preserve the public series tree.
    /// Coin days accrued by unspent supply between block timestamps, allocated
    /// to the age range in which they accrue. One coin day is one BTC remaining
    /// unspent for one day.
    #[traversable(wrap = "frameworks/cointime/age_range")]
    pub coindays_created: ColumnarPerBlockCumulativeRolling<
        StoredF64,
        AgeRangeId,
        AgeRange<LazyColumnPerBlockCumulativeRolling<StoredF64, AgeRangeId>>,
        M,
    >,
    #[traversable(wrap = "cointime/activity")]
    /// Coin blocks destroyed by spent outputs: each spent output's value in
    /// BTC multiplied by its age in blocks, summed over the represented block.
    pub coinblocks_destroyed: PerBlockCumulativeRolling<StoredF64, M>,
    pub addrs: AddrVecs<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Send + Sync,
{
    fn id(&self) -> &'static str {
        DB_NAME
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }

    fn mutates_existing(&self, vec: &dyn AnyVec) -> bool {
        [
            &self.any_addr_indexes.p2a as &dyn AnyVec,
            &self.any_addr_indexes.p2pk33,
            &self.any_addr_indexes.p2pk65,
            &self.any_addr_indexes.p2pkh,
            &self.any_addr_indexes.p2sh,
            &self.any_addr_indexes.p2tr,
            &self.any_addr_indexes.p2wpkh,
            &self.any_addr_indexes.p2wsh,
            &self.addrs_data.funded,
            &self.addrs_data.empty,
        ]
        .into_iter()
        .any(|mutable| std::ptr::addr_eq(vec, mutable))
    }
}

const SAVED_STAMPED_CHANGES: u16 = 10;
/// Version of the persisted funded-address data layout.
const FUNDED_ADDR_DATA_VERSION: Version = Version::new(3);

impl Vecs {
    pub fn all_chain_sources(&self) -> AllChainSources {
        AllChainSources::new(self.cohorts.all_supply(), self.cohorts.all_market_cap())
    }

    pub fn forced_import(
        parent: &Path,
        parent_version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        prices: &price::Vecs,
        inputs_by_type: &inputs::ByTypeVecs,
        outputs_by_type: &outputs::ByTypeVecs,
    ) -> Result<Self> {
        let db_path = parent.join(DB_NAME);
        let states_path = db_path.join("states");

        let db = open_db(parent, DB_NAME, 20_000_000)?;
        db.set_min_regions(50_000)?;

        let version = parent_version + VERSION;
        let spot_price = prices.spot.cents.height.read_only_cached_boxed_clone();

        let cohorts =
            CohortMetrics::forced_import(&db, version, indexes, cached_starts, &spot_price)?;

        // Create address data vecs first so we can also use them for identity mappings.
        let funded_addr_data_version = version + FUNDED_ADDR_DATA_VERSION;
        let funded_addr_index_to_funded_addr_data = OverflowVec::forced_import_with(
            ImportOptions::new(&db, "funded_addr_data", funded_addr_data_version)
                .with_saved_stamped_changes(SAVED_STAMPED_CHANGES),
        )?;
        let empty_addr_index_to_empty_addr_data = OverflowVec::forced_import_with(
            ImportOptions::new(&db, "empty_addr_data", version)
                .with_saved_stamped_changes(SAVED_STAMPED_CHANGES),
        )?;

        // Identity mappings for traversable
        let funded_addr_index = LazyVec::init(
            "funded_addr_index",
            funded_addr_data_version,
            funded_addr_index_to_funded_addr_data.read_only_boxed_clone(),
            |index, _| index,
        );
        let empty_addr_index = LazyVec::init(
            "empty_addr_index",
            version,
            empty_addr_index_to_empty_addr_data.read_only_boxed_clone(),
            |index, _| index,
        );

        let funded_addr_count =
            FundedAddrCountsVecs::forced_import(&db, version, indexes, cached_starts)?;
        let empty_addr_count =
            AddrCountsVecs::forced_import(&db, "empty_addr_count", version, indexes)?;
        let addr_activity = AddrActivityVecs::forced_import(&db, version, indexes, cached_starts)?;

        // Stored total = addr_count + empty_addr_count (global + per-type, with all derived indexes)
        let total_addr_count = TotalAddrCountVecs::forced_import(&db, version, indexes)?;

        // Per-block delta of total (global + per-type)
        let new_addr_count =
            NewAddrCountVecs::new(version, &total_addr_count, indexes, cached_starts);

        // Reused address tracking (counts + per-block uses + percent).
        // `reused_*` uses the receive-side predicate (funded_txo_count > 1,
        // industry standard). `respent_*` uses the spend-side counterpart
        // (spent_txo_count > 1, strictly more restrictive).
        let reused_addr_count = ReusedAddrVecs::forced_import(
            &db,
            "reused",
            version,
            indexes,
            cached_starts,
            &spot_price,
            outputs_by_type,
            inputs_by_type,
            cohorts.all_supply(),
        )?;
        let respent_addr_count = ReusedAddrVecs::forced_import(
            &db,
            "respent",
            version,
            indexes,
            cached_starts,
            &spot_price,
            outputs_by_type,
            inputs_by_type,
            cohorts.all_supply(),
        )?;

        // Exposed address tracking (counts + supply) - quantum / pubkey-exposure sense
        let exposed_addr_vecs = ExposedAddrVecs::forced_import(
            &db,
            version,
            indexes,
            &spot_price,
            cohorts.all_supply(),
        )?;

        // Growth rate: delta change + rate (global + per-type)
        let delta = DeltaVecs::new(version, &funded_addr_count.counts, cached_starts, indexes);

        // Average amount (supply / utxo_count, supply / funded_addr_count) for `all` and per addr type.
        let all_chain = AllChainSources::new(cohorts.all_supply(), cohorts.all_market_cap());
        let avg_amount = AvgAmountVecs::forced_import(
            &db,
            version,
            indexes,
            &spot_price,
            &all_chain,
            &cohorts.outputs.unspent_count.cohorts.all.height,
            &funded_addr_count.counts.all.height,
        )?;

        let this = Self {
            plugin_gate: Default::default(),
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
                funded_index: funded_addr_index,
                empty_index: empty_addr_index,
            },

            cohorts,

            coindays_created: ColumnarPerBlockCumulativeRolling::forced_import(
                &db,
                &CohortContext::Utxo.prefixed("age_range_coindays_created_cumulative"),
                version + COINDAYS_CREATED_VERSION,
                |source| {
                    AgeRangeId::series(CohortContext::Utxo, |column, name| {
                        LazyColumnPerBlockCumulativeRolling::new(
                            &format!("{name}_coindays_created"),
                            version,
                            source,
                            column,
                            indexes,
                            cached_starts,
                        )
                    })
                },
            )?,

            coinblocks_destroyed: PerBlockCumulativeRolling::forced_import(
                &db,
                "coinblocks_destroyed",
                version + Version::TWO,
                indexes,
                cached_starts,
            )?,

            any_addr_indexes: AnyAddrIndexesVecs::forced_import(&db, version)?,
            addrs_data: AddrsDataVecs {
                funded: funded_addr_index_to_funded_addr_data,
                empty: empty_addr_index_to_empty_addr_data,
            },
            inner: Inner::new(db),
            states_path,
        };

        finalize_db(&this.inner.db, &this)?;
        Ok(this)
    }

    /// Reset in-memory caches that become stale after rollback.
    fn reset_in_memory_caches(&mut self) {
        self.cohorts.invalidate_caches();
        self.inner.reset();
    }

    /// Main computation loop.
    ///
    /// Processes blocks to compute UTXO and address cohort metrics:
    /// 1. Recovers state from checkpoints or starts fresh
    /// 2. Iterates through blocks, processing outputs/inputs in parallel
    /// 3. Flushes checkpoints periodically
    /// 4. Computes aggregate cohorts from separate cohorts
    /// 5. Computes derived metrics
    #[allow(clippy::too_many_arguments)]
    pub fn compute(
        &mut self,
        indexer: &Indexer,
        indexes: &indexes::Vecs,
        inputs: &inputs::Vecs,
        outputs: &outputs::Vecs,
        transactions: &transactions::Vecs,
        prices: &price::Vecs,
        exit: &Exit,
    ) -> Result<UTXOStates> {
        self.inner.db.sync_bg_tasks()?;
        let mut utxo_states = UTXOStates::new(&self.states_path);
        let mut addr_states = AddrStates::new(&self.states_path);

        let base_version = VERSION
            + [
                prices.spot.cents.height.version(),
                indexes.timestamp.monotonic.version(),
                indexer.vecs().transactions.first_tx_index.version(),
                indexer.vecs().outputs.first_txout_index.version(),
                indexer.vecs().inputs.first_txin_index.version(),
                transactions.count.total.block.version(),
                outputs.count.total.sum.version(),
                inputs.count.sum.version(),
                indexes.tx_index.output_count.version(),
                indexes.tx_index.input_count.version(),
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
        self.coindays_created
            .cumulative
            .validate_computed_version_or_reset(base_version)?;
        debug!("computed versions validated");

        let starting_lengths = indexer.safe_lengths();

        // 1. Find the height from which the block loop can safely resume.
        let current_height = Height::from(self.supply_state.len());
        let min_resume_len = self.min_resume_len();

        // 2. Determine start mode and recover/reset state
        // Clamp to starting_lengths.height to handle reorg (indexer may require earlier start)
        let resume_target = current_height.min(starting_lengths.height);
        if resume_target < current_height {
            info!(
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

                debug!(
                    "recover_state completed, starting_height={}",
                    recovered.starting_height
                );
                recovered.starting_height
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
            self.supply_state.reset()?;
            self.addrs.reset_height()?;
            reset_state(
                &mut self.any_addr_indexes,
                &mut self.addrs_data,
                &mut utxo_states,
                &mut addr_states,
            )?;
            info!("State recovery: fresh start");
        }

        // Populate price/timestamp caches from the prices module.
        // Must happen AFTER rollback/reset (which invalidates caches) but BEFORE
        // chain_state rebuild (which reads from them).
        let cache_target_len = prices
            .spot
            .cents
            .height
            .len()
            .min(indexes.timestamp.monotonic.len());
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
            let new_timestamps = indexes
                .timestamp
                .monotonic
                .collect_range_at(cache_current_len, cache_target_len);
            self.inner.prices.extend(new_prices);
            self.inner.timestamps.extend(new_timestamps);
        }
        self.inner.price_range_max.extend(&self.inner.prices);

        // Take chain_state and tx_index_to_height out of self to avoid borrow conflicts
        let mut chain_state = mem::take(&mut self.inner.chain_state);
        let mut tx_index_to_height = mem::take(&mut self.inner.tx_index_to_height);

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

            // Truncate RangeMap to match (entries are immutable, safe to keep)
            tx_index_to_height.truncate(end);

            recovered_height
        };

        // 3. Get last height from indexer
        let last_height = Height::from(indexer.vecs().blocks.blockhash.len().saturating_sub(1));
        debug!(
            "last_height={}, starting_height={}",
            last_height, starting_height
        );

        // 4. Process blocks
        if starting_height <= last_height {
            debug!("calling process_blocks");

            let prices = mem::take(&mut self.inner.prices);
            let timestamps = mem::take(&mut self.inner.timestamps);
            let price_range_max = mem::take(&mut self.inner.price_range_max);
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
                indexes,
                inputs,
                outputs,
                transactions,
                starting_height,
                last_height,
                &mut chain_state,
                &mut tx_index_to_height,
                entry_anchor,
                &prices,
                &timestamps,
                &price_range_max,
                exit,
            )?;

            self.inner.prices = prices;
            self.inner.timestamps = timestamps;
            self.inner.price_range_max = price_range_max;
        }

        // Put chain_state and tx_index_to_height back
        self.inner.chain_state = chain_state;
        self.inner.tx_index_to_height = tx_index_to_height;

        // 5. Compute rest part1 (day1 mappings)
        info!("Computing rest part 1...");
        self.cohorts.compute_rest_part1(&starting_lengths, exit)?;

        // 6b. Compute address metrics derived from stored per-type sources.
        let type_supply = &self.cohorts.supply.total.cohorts.type_;
        let type_outputs = &self.cohorts.outputs.unspent_count.cohorts.type_;
        let type_supply_sats =
            AddrTypeId::series(|column, _| &type_supply.get(column.output_type()).sats.height);
        let type_utxo_counts =
            AddrTypeId::series(|column, _| &type_outputs.get(column.output_type()).height);
        self.addrs
            .reused
            .compute_rest(&starting_lengths, &type_supply_sats, exit)?;
        self.addrs
            .respent
            .compute_rest(&starting_lengths, &type_supply_sats, exit)?;
        self.addrs
            .exposed
            .compute_rest(&starting_lengths, &type_supply_sats, exit)?;

        let type_funded_addr_counts = AddrTypeId::series(|column, _| {
            &column.select(&self.addrs.funded.counts.by_addr_type).height
        });
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
        info!("Computing rest part 2...");
        self.cohorts.compute_rest_part2(&starting_lengths, exit)?;

        let exit = exit.clone();
        self.inner.db.run_bg(move |db| {
            let _lock = exit.lock();
            db.compact_deferred_default()
        });
        Ok(utxo_states)
    }

    pub fn flush(&self) -> Result<()> {
        self.inner.db.flush()?;
        Ok(())
    }

    fn min_resume_len(&self) -> Height {
        self.cohorts
            .min_resume_len()
            .min(Height::from(self.supply_state.len()))
            .min(self.any_addr_indexes.min_stamped_len())
            .min(self.addrs_data.min_stamped_len())
            .min(Height::from(self.addrs.min_resume_len()))
            .min(Height::from(self.coindays_created.cumulative.len()))
            .min(Height::from(self.coinblocks_destroyed.block.len()))
    }
}
