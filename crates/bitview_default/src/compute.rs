use std::{sync::Arc, thread, time::Instant};

use bitview_distribution::walk::Sources;

use bitview_plugin::{BootstrapAction, ComputePlugin, ComputePluginSet, PluginId, UpdateContext};
use bitview_plugin_addresses::Dependencies as AddressesDependencies;
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
use bitview_primitives::Lengths;
#[cfg(feature = "bench-cpu")]
use brk_error::Error;
use brk_error::Result;
#[cfg(feature = "bench-cpu")]
use brk_types::Height;
use brk_types::Transaction;
use tracing::info;
use vecdb::ReadableCloneableVec;

use crate::{
    DefaultPlugins,
    dispatch::{join, spawn},
    serial,
    timing::{Timed, log_compute, timed_compute},
};

const REIMPORT_THRESHOLD: u32 = 10_000;
/// The URPD history's compute, timed apart from the UTXO set plugin that holds it.
const URPD: PluginId = PluginId::new("urpd");
/// The walk's own compute: what it spends outside its consumers, reading the sources, moving the
/// origins and finding the age crossings.
const WALK: PluginId = PluginId::new("walk");

impl DefaultPlugins {
    fn compute_all(&mut self, context: UpdateContext<'_>) -> Result<()> {
        timed_compute(&mut *self.indexer, (), context)?;
        self.compute_dependents(context)
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
        self.roll_back(&starting, context)?;
        Ok(true)
    }

    /// Rolls back to `starting`, the address state first. Returns the height the address state
    /// landed on: 0 when it could not.
    fn roll_back(&mut self, starting: &Lengths, context: UpdateContext<'_>) -> Result<usize> {
        let _lock = context.exit().lock();
        let landed = self
            .addresses
            .roll_back_to(usize::from(starting.height), &self.indexer)?;
        self.indexer.roll_back_to(starting)?;
        Ok(landed)
    }

    /// Rolls every plugin back to before `first`, the start of the last `UNDO_TAIL` blocks the
    /// bulk sync wrote with undo, for the benchmark's replay of them.
    #[cfg(feature = "bench-cpu")]
    pub fn rewind_for_replay(&mut self, first: Height, context: UpdateContext<'_>) -> Result<()> {
        let starting = self.indexer.lengths_before(first)?;
        if self.roll_back(&starting, context)? != usize::from(first) {
            return Err(Error::Internal("Address undo does not reach the replay's first block"));
        }
        Ok(())
    }

    /// Stops the updates that follow at `tip`, for the benchmark's replay of a recorded prefix.
    #[cfg(feature = "bench-cpu")]
    pub fn cap_tip(&mut self, tip: Height) {
        self.indexer.cap_tip(tip);
    }

    /// The height of the last block the composition has indexed.
    #[cfg(feature = "bench-cpu")]
    pub fn tip_height(&self) -> Option<Height> {
        self.indexer.vecs().next_height().decremented()
    }

    /// Starts each plugin once every plugin its `Dependencies` borrow has computed, keeping each
    /// plugin's internal order. Branches beside the main chain (Outputs → UTXO set → the walk of
    /// its history, which computes the URPD history, Age and Profitability → Holders → Cointime)
    /// are scoped threads; forks that rejoin before going on are `rayon::join`s. A
    /// branch spawns its own dependents and hands over its plugin as soon as it completes; every
    /// branch is joined before the composition commits.
    fn compute_dependents(&mut self, context: UpdateContext<'_>) -> Result<()> {
        brk_alloc::collect();
        let indexer = self.indexer.as_ref();
        let from = indexer.compute_start().height;

        // Price reads only the indexer.
        let (mappings, price) = join(
            || {
                timed_compute(
                    &mut *self.mappings,
                    MappingsDependencies { indexer },
                    context,
                )
            },
            || timed_compute(&mut *self.price, PriceDependencies { indexer }, context),
        );
        mappings?;
        price?;
        let mappings = self.mappings.as_ref();
        let price = self.price.as_ref();

        thread::scope(|scope| -> Result<()> {
            // Only the UTXO set reads inputs' spends.
            let inputs = spawn(scope, || -> Result<_> {
                timed_compute(
                    &mut *self.inputs,
                    InputsDependencies { mappings, indexer },
                    context,
                )?;
                Ok(self.inputs.as_ref())
            });
            let market = spawn(scope, || {
                timed_compute(
                    &mut *self.market,
                    MarketDependencies {
                        indexer,
                        price,
                        mappings,
                    },
                    context,
                )
            });
            let utxos = spawn(scope, || {
                timed_compute(
                    &mut *self.utxos,
                    UtxosDependencies {
                        indexer,
                        mappings,
                        price,
                    },
                    context,
                )
            });
            let addresses = spawn(scope, || {
                timed_compute(
                    &mut *self.addresses,
                    AddressesDependencies {
                        indexer,
                        mappings,
                        price,
                    },
                    context,
                )
            });
            // Silent payments read only the indexer and Core's block files.
            let silent_payments = spawn(scope, || {
                timed_compute(
                    &mut *self.silent_payments,
                    SilentPaymentsDependencies { indexer },
                    context,
                )
            });

            // Blocks beside Transactions, then OP_RETURN beside Mining and Pools after it (Mining
            // and Pools read Blocks): the branch hands Mining over once it completes.
            let (op_return, pools) = (&mut *self.op_return, &mut *self.pools);
            let mining = spawn(scope, || -> Result<_> {
                let (blocks, transactions) = join(
                    || {
                        timed_compute(
                            &mut *self.blocks,
                            BlocksDependencies { indexer, mappings },
                            context,
                        )
                    },
                    || {
                        timed_compute(
                            &mut *self.transactions,
                            TransactionsDependencies {
                                indexer,
                                mappings,
                                price,
                            },
                            context,
                        )
                    },
                );
                blocks?;
                transactions?;
                let transactions = self.transactions.as_ref();
                let op_return = spawn(scope, move || {
                    timed_compute(
                        op_return,
                        OpReturnDependencies {
                            indexer,
                            fees: &transactions.fees,
                            price,
                        },
                        context,
                    )
                });
                timed_compute(
                    &mut *self.mining,
                    MiningDependencies {
                        indexer,
                        mappings,
                        blocks: self.blocks.as_ref(),
                        transactions,
                        price,
                    },
                    context,
                )?;
                let (blocks, mining) = (self.blocks.as_ref(), self.mining.as_ref());
                let pools = spawn(scope, move || {
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
                Ok((mining, transactions, [op_return, pools]))
            });

            timed_compute(
                &mut *self.outputs,
                OutputsDependencies {
                    indexer,
                    mappings,
                    price,
                },
                context,
            )?;
            let outputs = self.outputs.as_ref();

            // Supply reads Outputs, Mining and Price: it runs beside the history chain.
            let supply = &mut *self.supply;
            let supply = spawn(scope, move || -> Result<_> {
                let (mining, transactions, mining_tail) = mining.join().unwrap()?;
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
                Ok((&*supply, transactions, mining_tail))
            });

            let inputs = inputs.join().unwrap()?;
            let creations = &outputs.creations;
            timed_compute(
                &mut *self.utxo_set,
                UtxoSetDependencies {
                    spends: &inputs.spends,
                    creations,
                    from,
                    end: creations.end(),
                },
                context,
            )?;

            // One walk of the history computes everything that follows the coins block by block.
            let walk_step = serial::step(WALK);
            let (history, walk, urpd) = self.utxo_set.walk(&inputs.spends, creations)?;
            let sources = Sources {
                history: &history,
                prices: &price.spot.cents.height.read_only_boxed_clone(),
                timestamps: &mappings.timestamp.monotonic.read_only_boxed_clone(),
            };
            let mut cells = Timed::new(URPD, || {
                UrpdConsumer::new(urpd, sources, from, context.exit())
            })?;
            let mut age = Timed::new(AGE_ID, || self.age.consumer(sources, from, context))?;
            let mut profitability = Timed::new(PROFITABILITY_ID, || {
                self.profitability.consumer(sources, from, context)
            })?;
            let start = Instant::now();
            walk.run(sources, &mut [&mut cells, &mut age, &mut profitability])?;
            walk_step.end();
            log_compute(
                WALK,
                start
                    .elapsed()
                    .saturating_sub(cells.walked() + age.walked() + profitability.walked()),
            );
            cells.finish(drop);
            age.finish(AgeConsumer::keep);
            profitability.finish(ProfitabilityConsumer::keep);
            let (history, walk, urpd) = (sources.history, &*walk, &*urpd);
            let age = self.age.as_ref();

            thread::scope(|scope| -> Result<()> {

                // Coinflow reads Age, not Holders.
                let coinflow = spawn(scope, || -> Result<_> {
                    timed_compute(
                        &mut *self.coinflow,
                        CoinflowDependencies {
                            urpd,
                            indexer,
                            mappings,
                            price,
                            age,
                        },
                        context,
                    )?;
                    Ok(self.coinflow.as_ref())
                });
                timed_compute(
                    &mut *self.holders,
                    HoldersDependencies { from, age, price },
                    context,
                )?;
                let holders = self.holders.as_ref();

                let capitalized_price = holders
                    .cohorts
                    .all
                    .cost_basis
                    .by_dollar
                    .avg
                    .cents
                    .height
                    .read_only_boxed_clone();
                let entry = &mut *self.entry;
                let entry = spawn(scope, move || {
                    timed_compute(
                        entry,
                        EntryDependencies {
                            sources,
                            walk,
                            from,
                            capitalized_price: &capitalized_price,
                        },
                        context,
                    )
                });

                let (supply, transactions, mining_tail) = supply.join().unwrap()?;
                timed_compute(
                    &mut *self.cointime,
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
                let cointime = self.cointime.as_ref();
                let coinflow = coinflow.join().unwrap()?;

                let (bedrock, rarity_meter) = join(
                    || {
                        timed_compute(
                            &mut *self.bedrock,
                            BedrockDependencies {
                                urpd,
                                history,
                                price,
                                indexer,
                                age,
                                holders,
                                cointime,
                                coinflow,
                            },
                            context,
                        )
                    },
                    || {
                        timed_compute(
                            &mut *self.rarity_meter,
                            RarityMeterDependencies { indexer, price },
                            context,
                        )
                    },
                );
                bedrock?;
                rarity_meter?;
                for branch in mining_tail.into_iter().chain([entry]) {
                    branch.join().unwrap()?;
                }
                Ok(())
            })?;
            for branch in [market, utxos, addresses, silent_payments] {
                branch.join().unwrap()?;
            }
            Ok(())
        })
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

        if blocks_behind > REIMPORT_THRESHOLD {
            info!("Indexing {blocks_behind} blocks before initializing metrics");
            timed_compute(&mut *self.indexer, (), context)?;
            return Ok(BootstrapAction::Reimport);
        }

        self.compute_all(context)?;

        Ok(BootstrapAction::Ready)
    }

    fn compute(&mut self, context: UpdateContext<'_>) -> Result<bool> {
        let indexer_step = serial::step(bitview_plugin_indexer::ID);
        let rolled_back = self.roll_back_reorg(context)?;
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
        // Every dependent ends each update at the indexer's tip (an update that errs stops the
        // process, and bootstrap computes every plugin): unless the index moved, there is nothing
        // to compute, publish or log (a reader that failed logged why).
        if !rolled_back && self.indexer.vecs().next_height() == next {
            return Ok(false);
        }
        log_compute(bitview_plugin_indexer::ID, start.elapsed());
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
        let _step = serial::step(bitview_plugin_addresses::ID);
        self.addresses.write_published_state()
    }

    // The reads an update spends most of its time on when idle minutes left them on disk: the
    // indexer's lookups and the addresses' data. Not the spend index: its writes read their pages
    // from disk either way (rawdb's indexed writes on macOS).
    fn warm(&self, txs: &[Arc<Transaction>]) {
        self.addresses.warm(&self.indexer.warm(txs));
    }
}
