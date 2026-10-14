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
    /// Updates every plugin, enabling indexer collision checks in debug builds.
    fn compute_inner(&mut self, context: UpdateContext<'_>) -> Result<()> {
        self.compute_indexer(context)?;
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

    fn compute_indexer(&mut self, context: UpdateContext<'_>) -> Result<()> {
        timed_compute(&mut *self.indexer, (), context)
    }

    fn compute_dependents(&mut self, context: UpdateContext<'_>) -> Result<()> {
        let indexer = self.indexer.as_ref();

        timed_compute(
            &mut *self.mappings,
            MappingsDependencies { indexer },
            context,
        )?;

        thread::scope(|scope| -> Result<()> {
            timed_compute(
                &mut *self.blocks,
                BlocksDependencies {
                    indexer,
                    mappings: self.mappings.as_ref(),
                },
                context,
            )?;

            // Only the UTXO set reads inputs' spends: everything before it runs alongside.
            let inputs = scope.spawn(|| -> Result<_> {
                timed_compute(
                    &mut *self.inputs,
                    InputsDependencies {
                        mappings: self.mappings.as_ref(),
                        indexer,
                    },
                    context,
                )?;
                Ok(self.inputs.as_ref())
            });
            timed_compute(&mut *self.price, PriceDependencies { indexer }, context)?;

            // Market, UTXOs, Addresses, Outputs → History → Age → Holders, and Transactions →
            // Mining + OP_RETURN are independent complete-plugin branches.
            let market = scope.spawn(|| {
                timed_compute(
                    &mut *self.market,
                    MarketDependencies {
                        indexer,
                        price: self.price.as_ref(),
                        mappings: self.mappings.as_ref(),
                    },
                    context,
                )
            });

            let tx_mining_op_return = scope.spawn(|| -> Result<_> {
                timed_compute(
                    &mut *self.transactions,
                    TransactionsDependencies {
                        indexer,
                        mappings: self.mappings.as_ref(),
                        price: self.price.as_ref(),
                    },
                    context,
                )?;

                let (mining, op_return) = join(
                    || {
                        timed_compute(
                            &mut *self.mining,
                            MiningDependencies {
                                indexer,
                                mappings: self.mappings.as_ref(),
                                blocks: self.blocks.as_ref(),
                                transactions: self.transactions.as_ref(),
                                price: self.price.as_ref(),
                            },
                            context,
                        )
                    },
                    || {
                        timed_compute(
                            &mut *self.op_return,
                            OpReturnDependencies {
                                indexer,
                                fees: &self.transactions.fees,
                                price: self.price.as_ref(),
                            },
                            context,
                        )
                    },
                );
                mining?;
                op_return?;
                Ok(self.mining.as_ref())
            });

            // Addresses run alongside UTXOs: they read neither.
            let utxos = scope.spawn(|| {
                timed_compute(
                    &mut *self.utxos,
                    UtxosDependencies {
                        indexer,
                        mappings: &self.mappings,
                        price: &self.price,
                    },
                    context,
                )
            });
            let addresses = scope.spawn(|| {
                timed_compute(
                    &mut *self.addresses,
                    AddressesDependencies {
                        indexer,
                        mappings: &self.mappings,
                        price: &self.price,
                    },
                    context,
                )
            });

            timed_compute(
                &mut *self.outputs,
                OutputsDependencies {
                    indexer,
                    mappings: &self.mappings,
                    price: &self.price,
                },
                context,
            )?;
            let creations = &self.outputs.creations;
            let inputs = inputs.join().unwrap()?;

            timed_compute(
                &mut *self.utxo_set,
                UtxoSetDependencies {
                    spends: &inputs.spends,
                    creations,
                    from: indexer.compute_start().height,
                    end: creations.end(),
                },
                context,
            )?;

            let history = self.utxo_set.reader(&inputs.spends, creations)?;
            timed_compute(
                &mut *self.age,
                AgeDependencies {
                    history: &history,
                    from: indexer.compute_start().height,
                    mappings: self.mappings.as_ref(),
                    price: self.price.as_ref(),
                },
                context,
            )?;
            timed_compute(
                &mut *self.holders,
                HoldersDependencies {
                    history: &history,
                    from: indexer.compute_start().height,
                    age: &self.age,
                    mappings: &self.mappings,
                    price: &self.price,
                },
                context,
            )?;
            let urpd = ReplayInputs {
                history: &history,
                prices: &self.price.spot.cents.height,
                timestamps: &self.mappings.timestamp.monotonic,
            };
            let prices = self.price.spot.cents.height.read_only_boxed_clone();
            let timestamps = self.mappings.timestamp.monotonic.read_only_boxed_clone();
            let capitalized_price = self
                .holders
                .cohorts
                .all
                .cost_basis
                .per_dollar
                .avg
                .cents
                .height
                .read_only_boxed_clone();
            thread::scope(|scope| -> Result<()> {
                let entry = scope.spawn(|| {
                    timed_compute(
                        &mut *self.entry,
                        EntryDependencies {
                            history: &history,
                            from: indexer.compute_start().height,
                            prices: &prices,
                            timestamps: &timestamps,
                            capitalized_price: &capitalized_price,
                        },
                        context,
                    )
                });
                let profitability = scope.spawn(|| {
                    timed_compute(
                        &mut *self.profitability,
                        ProfitabilityDependencies {
                            history: &history,
                            from: indexer.compute_start().height,
                            prices: &prices,
                            timestamps: &timestamps,
                        },
                        context,
                    )
                });
                let coinflow = scope.spawn(|| -> Result<_> {
                    timed_compute(
                        &mut *self.coinflow,
                        CoinflowDependencies {
                            urpd,
                            indexer,
                            mappings: self.mappings.as_ref(),
                            price: self.price.as_ref(),
                            age: self.age.as_ref(),
                        },
                        context,
                    )?;
                    Ok(self.coinflow.as_ref())
                });
                let mining = tx_mining_op_return.join().unwrap()?;
                let pools = scope.spawn(|| {
                    timed_compute(
                        &mut *self.pools,
                        PoolsDependencies { indexer, mining },
                        context,
                    )
                });
                timed_compute(
                    &mut *self.supply,
                    SupplyDependencies {
                        indexer,
                        outputs: self.outputs.as_ref(),
                        mining,
                        price: self.price.as_ref(),
                    },
                    context,
                )?;

                timed_compute(
                    &mut *self.cointime,
                    CointimeDependencies {
                        urpd,
                        indexer,
                        mappings: self.mappings.as_ref(),
                        price: self.price.as_ref(),
                        inflation_rate: &self.supply.inflation_rate,
                        velocity_btc: &self.supply.velocity.btc,
                        velocity_usd: &self.supply.velocity.usd,
                        age: self.age.as_ref(),
                        holders: self.holders.as_ref(),
                    },
                    context,
                )?;
                let coinflow = coinflow.join().unwrap()?;

                timed_compute(
                    &mut *self.bedrock,
                    BedrockDependencies {
                        urpd,
                        indexer,
                        age: &self.age,
                        holders: &self.holders,
                        cointime: &self.cointime,
                        coinflow,
                    },
                    context,
                )?;
                timed_compute(
                    &mut *self.rarity_meter,
                    RarityMeterDependencies {
                        indexer,
                        holders: self.holders.as_ref(),
                        cointime: self.cointime.as_ref(),
                        coinflow,
                        price: self.price.as_ref(),
                    },
                    context,
                )?;
                pools.join().unwrap()?;
                entry.join().unwrap()?;
                profitability.join().unwrap()?;
                Ok(())
            })?;
            market.join().unwrap()?;
            utxos.join().unwrap()?;
            addresses.join().unwrap()?;
            Ok(())
        })?;

        Ok(())
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
            self.compute_indexer(context)?;
            return Ok(BootstrapAction::Reimport);
        }

        self.compute_inner(context)?;

        Ok(BootstrapAction::Ready)
    }

    fn compute(&mut self, context: UpdateContext<'_>) -> Result<()> {
        self.roll_back_reorg(context)?;
        self.compute_inner(context)
    }

    fn commit(&mut self, complete: bool) -> Result<()> {
        let lengths = self.indexer.prepare_commit()?;
        // Address state changes in place: it is written with the lengths, under the lock its
        // readers take before they pin, so they read the state of the publication they pin.
        let stable = self.addresses.state.stable();
        let mut written = stable.write();
        self.addresses.publish_state(&mut written)?;
        self.indexer.publish(lengths, complete)
    }
}
