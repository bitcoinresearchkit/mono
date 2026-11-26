use std::{sync::Arc, time::Instant};

use bitview_distribution::walk::{Reader, Sources};

use bitview_plugin::{BootstrapAction, ComputePlugin, ComputePluginSet, PluginId, UpdateContext};
use bitview_plugin_addresses::Dependencies as AddressesDependencies;
use bitview_plugin_age::Vecs as Age;
use bitview_plugin_coinflow::Vecs as Coinflow;
use bitview_plugin_holders::Vecs as Holders;
use bitview_plugin_inputs::Vecs as Inputs;
use bitview_plugin_mining::Vecs as Mining;
use bitview_plugin_outputs::Vecs as Outputs;
use bitview_plugin_transactions::Vecs as Transactions;
use bitview_plugin_utxo_set::UrpdHistory;
use bitview_plugin_age::{AgeConsumer, ID as AGE_ID};
use bitview_plugin_bedrock::Dependencies as BedrockDependencies;
use bitview_plugin_blocks::Dependencies as BlocksDependencies;
use bitview_plugin_coinflow::Dependencies as CoinflowDependencies;
use bitview_plugin_cointime::Dependencies as CointimeDependencies;
use bitview_plugin_entry::Dependencies as EntryDependencies;
use bitview_plugin_holders::Dependencies as HoldersDependencies;
use bitview_plugin_inputs::Dependencies as InputsDependencies;
use bitview_plugin_mappings::Dependencies as MappingsDependencies;
use bitview_plugin_market::Dependencies as MarketDependencies;
use bitview_plugin_mining::Dependencies as MiningDependencies;
use bitview_plugin_op_return::Dependencies as OpReturnDependencies;
use bitview_plugin_outputs::Dependencies as OutputsDependencies;
use bitview_plugin_pools::Dependencies as PoolsDependencies;
use bitview_plugin_price::Dependencies as PriceDependencies;
use bitview_plugin_profitability::{ID as PROFITABILITY_ID, ProfitabilityConsumer};
use bitview_plugin_rarity_meter::Dependencies as RarityMeterDependencies;
use bitview_plugin_silent_payments::Dependencies as SilentPaymentsDependencies;
use bitview_plugin_supply::Dependencies as SupplyDependencies;
use bitview_plugin_transactions::Dependencies as TransactionsDependencies;
use bitview_plugin_utxo_set::{Dependencies as UtxoSetDependencies, UrpdConsumer};
use bitview_plugin_utxos::Dependencies as UtxosDependencies;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::Transaction;
use rayon::Scope;
use tracing::info;
use vecdb::ReadableCloneableVec;

use crate::{
    DefaultPlugins,
    dispatch::{Failed, Meet, Root, spread},
    serial,
    timing::{Timed, log_compute, timed_compute},
};

/// The URPD history's compute, timed apart from the UTXO set plugin that holds it.
const URPD: PluginId = PluginId::new("urpd");
/// The walk's own compute: what it spends outside its consumers, reading the sources, moving the
/// origins and finding the age crossings.
const WALK: PluginId = PluginId::new("walk");

impl DefaultPlugins {
    /// Computes the indexer and returns whether its index moved. A reader that failed at the next
    /// height leaves it where it was (and logged why); every dependent ends each update at the
    /// indexer's tip (an update that errs stops the process, and bootstrap computes every plugin),
    /// so then there is nothing to compute, publish or log.
    fn index(&mut self, context: UpdateContext<'_>) -> Result<bool> {
        let indexer_step = serial::step(bitview_plugin_indexer::ID);
        let next = self.indexer.vecs().next_height();
        let start = Instant::now();
        self.indexer.compute((), context)?;
        #[cfg(feature = "bench-cpu")]
        {
            if serial::enabled() {
                self.indexer.wait_for_background()?;
            }
        }
        indexer_step.end();
        let moved = self.indexer.vecs().next_height() != next;
        if moved {
            log_compute(bitview_plugin_indexer::ID, start.elapsed());
        }
        Ok(moved)
    }

    /// A reorg rolls the address state (rewritten in place) back before the indexer rolls back,
    /// each lowering the publication first: address readers never wait for earlier pins to
    /// drain, they find the state at another length than their pin and answer that it is
    /// updating. A rollback that cannot land (`roll_back_to` returns 0) leaves the state for the
    /// address plugin's compute to reset (lowering the publication to genesis first). Bootstrap
    /// has no readers; its plugins roll back as they compute. Returns whether it rolled back.
    fn roll_back_reorg(&mut self, context: UpdateContext<'_>) -> Result<bool> {
        let Some(starting) = self.indexer.reorg()? else {
            return Ok(false);
        };
        let _lock = context.exit().lock();
        self.addresses
            .roll_back_to(usize::from(starting.height), &self.indexer)?;
        self.indexer.roll_back_to(&starting)?;
        Ok(true)
    }

    /// Closes the composition and removes every plugin's data but the indexer's, so the next
    /// bootstrap computes them all from zero over the indexed chain: the benchmark's serial
    /// recompute.
    #[cfg(feature = "bench-cpu")]
    pub fn remove_computed(self, context: bitview_plugin::ImportContext<'_>) -> Result<()> {
        drop(self);
        let _lock = context.exit().lock();
        bitview_plugin::PluginStorage::remove_plugin_data(
            context,
            |name| name == bitview_plugin_indexer::ID.as_str(),
            "the benchmark computes it again from zero",
        )
    }

    /// Starts each plugin once every plugin its `Dependencies` borrow has computed, keeping each
    /// plugin's internal order, inside rayon's pool. The long branches each run whole on the first
    /// worker to come free, out of reach of other workers' waits: Inputs, Outputs, Transactions
    /// (after Blocks, then Mining, OP_RETURN and Pools), Addresses, UTXOs and Silent payments.
    /// Market runs on the caller, which then takes branches too.
    /// The second of Inputs and Outputs to complete goes on with the UTXO set, the walk of its
    /// history (the URPD history, Age and Profitability), Holders, Coinflow and Entry; the second
    /// of that chain and Mining's to complete goes on with Supply, Cointime, Bedrock and the Rarity
    /// meter, one after another. The scope ends once every plugin has.
    fn compute_dependents(&mut self, context: UpdateContext<'_>) -> Result<()> {
        brk_alloc::collect();
        let failed = Failed::default();
        // Entered once, so every join here and every parallel call in a plugin starts on a pool
        // worker: a push to its own queue that idle workers steal. Called from a thread outside
        // the pool, each would be injected, and workers take injected jobs only when nothing is
        // left to steal: minutes under a full pool.
        let joined = rayon::scope(|scope| self.compute_in_pool(scope, &failed, context));
        joined?;
        failed.into_inner().unwrap().map_or(Ok(()), Err)
    }

    fn compute_in_pool<'s>(
        &'s mut self,
        scope: &Scope<'s>,
        failed: &'s Failed,
        context: UpdateContext<'s>,
    ) -> Result<()> {
        let indexer = self.indexer.as_ref();
        let from = indexer.compute_start().height;

        timed_compute(
            &mut *self.mappings,
            MappingsDependencies { indexer },
            context,
        )?;
        let mappings = self.mappings.as_ref();
        timed_compute(
            &mut *self.price,
            PriceDependencies { indexer, mappings },
            context,
        )?;
        let price = self.price.as_ref();

        let (supply, cointime, bedrock, rarity_meter) = (
            &mut *self.supply,
            &mut *self.cointime,
            &mut *self.bedrock,
            &mut *self.rarity_meter,
        );
        let tail = Arc::new(Meet::new(
            move |(mining, transactions): (&Mining, &Transactions),
                  (outputs, history, urpd, age, holders, coinflow): (
                &Outputs,
                Reader<'_>,
                &UrpdHistory,
                &Age,
                &Holders,
                &Coinflow,
            )|
                  -> Result<()> {
                // One after another, in place (see `dispatch::spread`).
                timed_compute(
                    &mut *supply,
                    SupplyDependencies {
                        indexer,
                        outputs,
                        mining,
                        price,
                    },
                    context,
                )?;
                timed_compute(
                    &mut *cointime,
                    CointimeDependencies {
                        urpd,
                        indexer,
                        mappings,
                        price,
                        inflation_rate: &supply.inflation_rate,
                        volume_1y: &transactions.transfer_volume.sum._1y,
                        age,
                        holders,
                    },
                    context,
                )?;
                let cointime: &_ = cointime;
                timed_compute(
                    &mut *bedrock,
                    BedrockDependencies {
                        urpd,
                        history: &history,
                        price,
                        indexer,
                        age,
                        holders,
                        cointime,
                        coinflow,
                    },
                    context,
                )?;
                timed_compute(
                    &mut *rarity_meter,
                    RarityMeterDependencies {
                        indexer,
                        price,
                        holders,
                        cointime,
                        coinflow,
                    },
                    context,
                )
            },
        ));

        let (utxo_set, age, profitability, holders, coinflow, entry) = (
            &mut *self.utxo_set,
            &mut *self.age,
            &mut *self.profitability,
            &mut *self.holders,
            &mut *self.coinflow,
            &mut *self.entry,
        );
        let history_tail = tail.clone();
        let history_chain = Arc::new(Meet::new(
            move |inputs: &Inputs, outputs: &Outputs| -> Result<()> {
                let creations = &outputs.creations;
                timed_compute(
                    &mut *utxo_set,
                    UtxoSetDependencies {
                        spends: &inputs.spends,
                        creations,
                        from,
                        end: creations.end(),
                    },
                    context,
                )?;

                // One walk of the history computes everything that follows the coins block by
                // block.
                let walk_step = serial::step(WALK);
                let (history, walk, urpd) = { utxo_set }.walk(&inputs.spends, creations)?;
                let prices = price.spot.cents.height.read_only_boxed_clone();
                let timestamps = mappings.timestamp.monotonic.read_only_boxed_clone();
                let sources = Sources {
                    history: &history,
                    prices: &prices,
                    timestamps: &timestamps,
                };
                let mut cells = Timed::new(URPD, || {
                    UrpdConsumer::new(urpd, sources, from, context.exit())
                })?;
                let mut age_walk = Timed::new(AGE_ID, || age.consumer(sources, from, context))?;
                let mut profitability_walk = Timed::new(PROFITABILITY_ID, || {
                    profitability.consumer(sources, from, context)
                })?;
                let start = Instant::now();
                walk.run(sources, &mut [&mut cells, &mut age_walk, &mut profitability_walk])?;
                walk_step.end();
                log_compute(
                    WALK,
                    start.elapsed().saturating_sub(
                        cells.walked() + age_walk.walked() + profitability_walk.walked(),
                    ),
                );
                cells.finish(drop);
                age_walk.finish(AgeConsumer::keep);
                profitability_walk.finish(ProfitabilityConsumer::keep);
                serial::compact(AGE_ID, Some(age.database()), context)?;
                serial::compact(PROFITABILITY_ID, Some(profitability.database()), context)?;
                let (walk, urpd, age): (&_, &_, &_) = (walk, urpd, age);

                timed_compute(&mut *holders, HoldersDependencies { from, age, price }, context)?;
                // Coinflow and Entry read only this chain's results: here, off the tail's path.
                timed_compute(
                    &mut *coinflow,
                    CoinflowDependencies {
                        urpd,
                        indexer,
                        mappings,
                        price,
                        age,
                    },
                    context,
                )?;
                let capitalized_price = holders
                    .cohorts
                    .all
                    .cost_basis
                    .by_dollar
                    .avg
                    .cents
                    .height
                    .read_only_boxed_clone();
                timed_compute(
                    &mut *entry,
                    EntryDependencies {
                        sources,
                        walk,
                        from,
                        capitalized_price: &capitalized_price,
                    },
                    context,
                )?;
                history_tail.right((outputs, history, urpd, age, &*holders, &*coinflow))
            },
        ));

        let (blocks, transactions, mining, op_return, pools) = (
            &mut *self.blocks,
            &mut *self.transactions,
            &mut *self.mining,
            &mut *self.op_return,
            &mut *self.pools,
        );
        // Blocks, then Transactions, in place: pushed beside Transactions, Blocks could be taken
        // by Market's waits. From zero it adds 4 s to this chain's 830, which ends long before
        // Addresses; at the tip the tail waits for the history side instead (Mining ended after
        // Holders in 1 of 40 single heavy regtest blocks), and Blocks as a root of its own meeting
        // Transactions at Mining measured no faster.
        let mining_chain: Root<'s> = Box::new(move || {
            timed_compute(&mut *blocks, BlocksDependencies { indexer, mappings }, context)?;
            timed_compute(
                &mut *transactions,
                TransactionsDependencies {
                    indexer,
                    mappings,
                    price,
                },
                context,
            )?;
            let (blocks, transactions): (&_, &_) = (blocks, transactions);
            timed_compute(
                &mut *mining,
                MiningDependencies {
                    indexer,
                    mappings,
                    blocks,
                    transactions,
                    price,
                },
                context,
            )?;
            let mining: &_ = mining;
            tail.left((mining, transactions))?;
            // Nothing waits for these: in place, where no wait can take them.
            timed_compute(
                op_return,
                OpReturnDependencies {
                    indexer,
                    fees: &transactions.fees,
                    price,
                },
                context,
            )?;
            timed_compute(
                pools,
                PoolsDependencies {
                    indexer,
                    blocks,
                    transactions,
                    mining,
                    mappings,
                },
                context,
            )
        });

        let (inputs, outputs) = (&mut *self.inputs, &mut *self.outputs);
        let inputs_history = history_chain.clone();
        let inputs: Root<'s> = Box::new(move || {
            timed_compute(&mut *inputs, InputsDependencies { mappings, indexer }, context)?;
            inputs_history.left(inputs)
        });
        let outputs: Root<'s> = Box::new(move || {
            timed_compute(
                &mut *outputs,
                OutputsDependencies {
                    indexer,
                    mappings,
                    price,
                },
                context,
            )?;
            history_chain.right(outputs)
        });
        let (addresses, utxos, silent_payments) = (
            &mut *self.addresses,
            &mut *self.utxos,
            &mut *self.silent_payments,
        );
        let addresses: Root<'s> = Box::new(move || {
            timed_compute(
                addresses,
                AddressesDependencies {
                    indexer,
                    mappings,
                    price,
                },
                context,
            )
        });
        let utxos: Root<'s> = Box::new(move || {
            timed_compute(
                utxos,
                UtxosDependencies {
                    indexer,
                    mappings,
                    price,
                },
                context,
            )
        });
        // Silent payments read only the indexer and Core's block files.
        let silent_payments: Root<'s> = Box::new(move || {
            timed_compute(
                silent_payments,
                SilentPaymentsDependencies { indexer },
                context,
            )
        });
        // Longest path first (from zero on m6-2: Addresses 1188 s, Transactions to the tail's end
        // 830 s, UTXOs 769 s, Outputs 540 s and Inputs 516 s to the tail's end, Silent payments
        // 488 s), so a pool with fewer workers than roots leaves the shorter ones waiting.
        let market = &mut *self.market;
        spread(
            scope,
            failed,
            context,
            vec![addresses, mining_chain, utxos, outputs, inputs, silent_payments],
            || {
                timed_compute(
                    market,
                    MarketDependencies {
                        indexer,
                        price,
                        mappings,
                    },
                    context,
                )
            },
        )
    }
}

impl ComputePluginSet for DefaultPlugins {
    fn bootstrap_compute(&mut self, context: UpdateContext<'_>) -> Result<BootstrapAction> {
        let blocks_behind = if cfg!(debug_assertions) {
            0
        } else {
            let chain_height = self.indexer.reader().client().get_last_height()?;
            // Local progress: the published height can sit at a rollback floor.
            chain_height.saturating_sub(*self.indexer.vecs().next_height())
        };
        let far_behind = blocks_behind > bitview_plugin_indexer::CATCH_UP_BLOCKS;
        if far_behind {
            info!("Indexing {blocks_behind} blocks before initializing metrics");
        }

        // A pass that moved nothing would reimport the same plugins into the same failure: the
        // steady-state loop retries it every second.
        if self.index(context)? && far_behind {
            return Ok(BootstrapAction::Reimport);
        }
        self.compute_dependents(context)?;

        Ok(BootstrapAction::Ready)
    }

    fn compute(&mut self, context: UpdateContext<'_>) -> Result<bool> {
        // Not during the bootstrap: the merge would compete with its computers, and a reimport
        // would wait for it.
        self.indexer.stores().resume_compaction();
        let rolled_back = self.roll_back_reorg(context)?;
        if !self.index(context)? && !rolled_back {
            return Ok(false);
        }
        self.compute_dependents(context)?;
        Ok(true)
    }

    fn commit(&mut self, complete: bool) -> Result<()> {
        let lengths = {
            let _step = serial::step(bitview_plugin_indexer::ID);
            self.indexer.prepare_commit()?
        };
        // Address state changes in place: it is published with the lengths, under the lock its
        // readers take before they pin, so they read the state of the publication they pin.
        let _step = serial::step(bitview_plugin_addresses::ID);
        self.addresses.publish_state_with(|| {
            let _step = serial::step(bitview_plugin_indexer::ID);
            self.indexer.publish(lengths, complete)
        })
    }

    fn write_published(&mut self) -> Result<()> {
        {
            let _step = serial::step(bitview_plugin_addresses::ID);
            self.addresses.write_published_state()?;
        }
        // What the update's writes held back until it was published (a day's closes).
        let start = Instant::now();
        let mut wrote = false;
        for vec in self.iter_any_stored_mut() {
            wrote |= vec.write_held()?;
        }
        if wrote {
            info!("Wrote held rows in {:.2?}", start.elapsed());
        }
        Ok(())
    }

    // The reads an update spends most of its time on when idle minutes left them on disk: the
    // indexer's lookups and the addresses' data. Not the spend index: its writes read their pages
    // from disk either way (rawdb's indexed writes on macOS).
    fn warm(&self, txs: &[Arc<Transaction>]) {
        self.addresses.warm(&self.indexer.warm(txs));
    }
}
