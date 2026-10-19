use std::thread;

use bitview_plugin::{BootstrapAction, ComputePluginSet, UpdateContext};
use bitview_plugin_addresses::Dependencies as AddressesDependencies;
use bitview_plugin_age::Dependencies as AgeDependencies;
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
use bitview_plugin_profitability::Dependencies as ProfitabilityDependencies;
use bitview_plugin_rarity_meter::Dependencies as RarityMeterDependencies;
use bitview_plugin_supply::Dependencies as SupplyDependencies;
use bitview_plugin_transactions::Dependencies as TransactionsDependencies;
use bitview_plugin_utxo_set::Dependencies as UtxoSetDependencies;
use bitview_plugin_utxos::Dependencies as UtxosDependencies;
use bitview_urpd::ReplayInputs;
use brk_error::Result;
use rayon::join;
use tracing::info;
use vecdb::ReadableCloneableVec;

use crate::{DefaultPlugins, timing::timed_compute};

const REIMPORT_THRESHOLD: u32 = 10_000;

impl DefaultPlugins {
    fn compute_all(&mut self, context: UpdateContext<'_>) -> Result<()> {
        timed_compute(&mut *self.indexer, (), context)?;
        brk_alloc::collect();
        self.compute_dependents(context)
    }

    /// A reorg rolls the address state (rewritten in place) back before the indexer rolls back,
    /// each lowering the publication first: address readers never wait for earlier pins to
    /// drain, they find the state at another length than their pin and answer that it is
    /// updating. A rollback that cannot land (`roll_back_to` returns 0) leaves the state for the
    /// address plugin's compute to reset (lowering the publication to genesis first). Bootstrap
    /// has no readers; its plugins roll back as they compute.
    fn roll_back_reorg(&mut self, context: UpdateContext<'_>) -> Result<()> {
        let Some(starting) = self.indexer.reorg()? else {
            return Ok(());
        };
        let _lock = context.exit().lock();
        self.addresses
            .roll_back_to(usize::from(starting.height), &self.indexer)?;
        self.indexer.roll_back_to(&starting)
    }

    /// Starts each plugin once every plugin its `Dependencies` borrow has computed, keeping each
    /// plugin's internal order. Branches beside the main chain (Outputs → UTXO set → Age → Holders
    /// → Cointime) are scoped threads; forks that rejoin before going on are `rayon::join`s. A
    /// branch spawns its own dependents and hands over its plugin as soon as it completes; every
    /// branch is joined before the composition commits.
    fn compute_dependents(&mut self, context: UpdateContext<'_>) -> Result<()> {
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
            let inputs = scope.spawn(|| -> Result<_> {
                timed_compute(
                    &mut *self.inputs,
                    InputsDependencies { mappings, indexer },
                    context,
                )?;
                Ok(self.inputs.as_ref())
            });
            let market = scope.spawn(|| {
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
            let utxos = scope.spawn(|| {
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
            let addresses = scope.spawn(|| {
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

            // Blocks beside Transactions (only Mining reads Blocks), then OP_RETURN beside Mining
            // and Pools after it: the branch hands Mining over once it completes.
            let (op_return, pools) = (&mut *self.op_return, &mut *self.pools);
            let mining = scope.spawn(|| -> Result<_> {
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
                let op_return = scope.spawn(move || {
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
                let mining = self.mining.as_ref();
                let pools = scope.spawn(move || {
                    timed_compute(pools, PoolsDependencies { indexer, mining }, context)
                });
                Ok((mining, [op_return, pools]))
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
            let supply = scope.spawn(move || -> Result<_> {
                let (mining, mining_tail) = mining.join().unwrap()?;
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
                Ok((&*supply, mining_tail))
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

            let history = &self.utxo_set.reader(&inputs.spends, creations)?;
            let urpd = ReplayInputs {
                history,
                prices: &price.spot.cents.height,
                timestamps: &mappings.timestamp.monotonic,
            };
            let prices = &price.spot.cents.height.read_only_boxed_clone();
            let timestamps = &mappings.timestamp.monotonic.read_only_boxed_clone();
            thread::scope(|scope| -> Result<()> {
                // Profitability reads only the history, prices and timestamps.
                let profitability = scope.spawn(|| {
                    timed_compute(
                        &mut *self.profitability,
                        ProfitabilityDependencies {
                            history,
                            from,
                            prices,
                            timestamps,
                        },
                        context,
                    )
                });
                timed_compute(
                    &mut *self.age,
                    AgeDependencies {
                        history,
                        from,
                        mappings,
                        price,
                    },
                    context,
                )?;
                let age = self.age.as_ref();

                // Coinflow reads Age, not Holders.
                let coinflow = scope.spawn(|| -> Result<_> {
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
                    HoldersDependencies {
                        history,
                        from,
                        age,
                        mappings,
                        price,
                    },
                    context,
                )?;
                let holders = self.holders.as_ref();

                let capitalized_price = holders
                    .cohorts
                    .all
                    .cost_basis
                    .per_dollar
                    .avg
                    .cents
                    .height
                    .read_only_boxed_clone();
                let entry = &mut *self.entry;
                let entry = scope.spawn(move || {
                    timed_compute(
                        entry,
                        EntryDependencies {
                            history,
                            from,
                            prices,
                            timestamps,
                            capitalized_price: &capitalized_price,
                        },
                        context,
                    )
                });

                let (supply, mining_tail) = supply.join().unwrap()?;
                timed_compute(
                    &mut *self.cointime,
                    CointimeDependencies {
                        urpd,
                        indexer,
                        mappings,
                        price,
                        inflation_rate: &supply.inflation_rate,
                        velocity_btc: &supply.velocity.btc,
                        velocity_usd: &supply.velocity.usd,
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
                            RarityMeterDependencies {
                                indexer,
                                holders,
                                cointime,
                                coinflow,
                                price,
                            },
                            context,
                        )
                    },
                );
                bedrock?;
                rarity_meter?;
                for branch in mining_tail.into_iter().chain([entry, profitability]) {
                    branch.join().unwrap()?;
                }
                Ok(())
            })?;
            for branch in [market, utxos, addresses] {
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

    fn compute(&mut self, context: UpdateContext<'_>) -> Result<()> {
        self.roll_back_reorg(context)?;
        self.compute_all(context)
    }

    fn commit(&mut self, complete: bool) -> Result<()> {
        let lengths = self.indexer.prepare_commit()?;
        // Address state changes in place: it is published with the lengths, under the lock its
        // readers take before they pin, so they read the state of the publication they pin.
        self.addresses
            .publish_state_with(|| self.indexer.publish(lengths, complete))
    }

    fn write_published(&mut self) -> Result<()> {
        self.addresses.write_published_state()
    }
}
