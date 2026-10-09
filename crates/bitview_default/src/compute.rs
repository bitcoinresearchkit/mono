use std::{sync::mpsc, thread, time::Duration};

use bitview_plugin::{ComputePlugin, Publication, UpdateContext};
use bitview_plugin_addresses::{Dependencies as AddressesDependencies, ID as ADDRESSES_ID};
use bitview_plugin_age::{Dependencies as AgeDependencies, ID as AGE_ID};
use bitview_plugin_bedrock::{Dependencies as BedrockDependencies, ID as BEDROCK_ID};
use bitview_plugin_blocks::{Dependencies as BlocksDependencies, ID as BLOCKS_ID};
use bitview_plugin_capital_sentiment::{
    Dependencies as CapitalSentimentDependencies, ID as CAPITAL_SENTIMENT_ID,
};
use bitview_plugin_coinflow::{Dependencies as CoinflowDependencies, ID as COINFLOW_ID};
use bitview_plugin_cointime::{Dependencies as CointimeDependencies, ID as COINTIME_ID};
use bitview_plugin_entry::{Dependencies as EntryDependencies, ID as ENTRY_ID};
use bitview_plugin_holders::{Dependencies as HoldersDependencies, ID as HOLDERS_ID};
use bitview_plugin_indexer::ID as INDEXER_ID;
use bitview_plugin_indicators::{Dependencies as IndicatorsDependencies, ID as INDICATORS_ID};
use bitview_plugin_inputs::{Dependencies as InputsDependencies, ID as INPUTS_ID};
use bitview_plugin_mappings::{Dependencies as MappingsDependencies, ID as MAPPINGS_ID};
use bitview_plugin_market::{Dependencies as MarketDependencies, ID as MARKET_ID};
use bitview_plugin_mining::{Dependencies as MiningDependencies, ID as MINING_ID};
use bitview_plugin_op_return::{Dependencies as OpReturnDependencies, ID as OP_RETURN_ID};
use bitview_plugin_outputs::{Dependencies as OutputsDependencies, ID as OUTPUTS_ID};
use bitview_plugin_pools::{Dependencies as PoolsDependencies, ID as POOLS_ID};
use bitview_plugin_price::{Dependencies as PriceDependencies, ID as PRICE_ID};
use bitview_plugin_rarity_meter::{Dependencies as RarityMeterDependencies, ID as RARITY_METER_ID};
use bitview_plugin_supply::{Dependencies as SupplyDependencies, ID as SUPPLY_ID};
use bitview_plugin_transactions::{
    Dependencies as TransactionsDependencies, ID as TRANSACTIONS_ID,
};
use bitview_plugin_utxo_history::{Dependencies as UtxoHistoryDependencies, ID as UTXO_HISTORY_ID};
use bitview_plugin_utxos::{Dependencies as UtxosDependencies, ID as UTXOS_ID};
use bitview_runtime::{BootstrapAction, ComputePluginSet};
use bitview_urpd::ReplayInputs;
use brk_error::Result;
use rayon::join;
use tracing::info;
use vecdb::ReadableCloneableVec;

use crate::{
    DefaultPlugins,
    timing::{Phase, timed},
};

const REIMPORT_THRESHOLD: u32 = 10_000;

impl DefaultPlugins {
    /// Updates every plugin, enabling indexer collision checks in debug builds.
    fn compute_inner(&mut self, context: UpdateContext<'_>) -> Result<()> {
        self.compute_indexer(context)?;
        brk_alloc::collect();
        self.compute_dependents(context)
    }

    fn compute_indexer(&mut self, context: UpdateContext<'_>) -> Result<()> {
        timed(Phase::Compute, INDEXER_ID, || {
            self.indexer.compute((), context)
        })
    }

    fn compute_dependents(&mut self, context: UpdateContext<'_>) -> Result<()> {
        let indexer = self.indexer.as_ref();

        timed(Phase::Compute, MAPPINGS_ID, || {
            self.mappings
                .compute(MappingsDependencies { indexer }, context)
        })?;

        thread::scope(|scope| -> Result<()> {
            timed(Phase::Compute, BLOCKS_ID, || {
                self.blocks.compute(BlocksDependencies { indexer }, context)
            })?;

            let (inputs_result, prices_result) = join(
                || {
                    timed(Phase::Compute, INPUTS_ID, || {
                        self.inputs.compute(
                            InputsDependencies {
                                mappings: self.mappings.as_ref(),
                                indexer,
                                blocks: self.blocks.as_ref(),
                            },
                            context,
                        )
                    })
                },
                || {
                    timed(Phase::Compute, PRICE_ID, || {
                        self.price.compute(PriceDependencies { indexer }, context)
                    })
                },
            );
            inputs_result?;
            prices_result?;

            // Market, UTXOs, Addresses, Outputs → History → Age → Holders, and Transactions →
            // Mining + OP_RETURN are independent complete-plugin branches.
            let market = scope.spawn(|| -> Result<_> {
                timed(Phase::Compute, MARKET_ID, || {
                    self.market.compute(
                        MarketDependencies {
                            indexer,
                            price: self.price.as_ref(),
                            mappings: self.mappings.as_ref(),
                            blocks: self.blocks.as_ref(),
                        },
                        context,
                    )
                })?;
                Ok(self.market.as_ref())
            });

            let tx_mining_op_return = scope.spawn(|| -> Result<_> {
                timed(Phase::Compute, TRANSACTIONS_ID, || {
                    self.transactions.compute(
                        TransactionsDependencies {
                            indexer,
                            inputs: self.inputs.as_ref(),
                            mappings: self.mappings.as_ref(),
                            blocks: self.blocks.as_ref(),
                            price: self.price.as_ref(),
                        },
                        context,
                    )
                })?;

                let (mining, op_return) = join(
                    || {
                        timed(Phase::Compute, MINING_ID, || {
                            self.mining.compute(
                                MiningDependencies {
                                    indexer,
                                    blocks: self.blocks.as_ref(),
                                    transactions: self.transactions.as_ref(),
                                    price: self.price.as_ref(),
                                },
                                context,
                            )
                        })
                    },
                    || {
                        timed(Phase::Compute, OP_RETURN_ID, || {
                            self.op_return.compute(
                                OpReturnDependencies {
                                    indexer,
                                    fees: &self.transactions.fees,
                                },
                                context,
                            )
                        })
                    },
                );
                mining?;
                op_return?;
                Ok(self.mining.as_ref())
            });

            // Addresses' block loop runs alongside UTXOs: only the shares and average balances it
            // derives afterwards wait for the per-type supply UTXOs sends once computed.
            let (type_supply_tx, type_supply_rx) = mpsc::sync_channel(1);
            let utxos = scope.spawn(|| -> Result<_> {
                let type_supply = type_supply_tx;
                timed(Phase::Compute, UTXOS_ID, || {
                    self.utxos.compute(
                        UtxosDependencies {
                            indexer,
                            mappings: &self.mappings,
                            input_values: &self.inputs.value,
                            price: &self.price,
                        },
                        context,
                    )
                })?;
                let _ = type_supply.send(self.utxos.type_supply());
                Ok(self.utxos.as_ref())
            });
            let addresses = scope.spawn(|| {
                timed(Phase::Compute, ADDRESSES_ID, || {
                    self.addresses.compute(
                        AddressesDependencies {
                            indexer,
                            mappings: &self.mappings,
                            input_values: &self.inputs.value,
                            price: &self.price,
                            type_supply: type_supply_rx,
                        },
                        context,
                    )
                })
            });

            timed(Phase::Compute, OUTPUTS_ID, || {
                self.outputs.compute(
                    OutputsDependencies {
                        indexer,
                        blocks: &self.blocks,
                        price: &self.price,
                    },
                    context,
                )
            })?;
            let creations = &self.outputs.creations;

            timed(Phase::Compute, UTXO_HISTORY_ID, || {
                self.utxo_history.compute(
                    UtxoHistoryDependencies {
                        spends: self.inputs.origins.spends(),
                        creations,
                        from: indexer.safe_lengths().height,
                        end: creations.end(),
                    },
                    context,
                )
            })?;

            let history = self
                .utxo_history
                .reader(self.inputs.origins.spends(), creations)?;
            timed(Phase::Compute, AGE_ID, || {
                self.age.compute(
                    AgeDependencies {
                        history: &history,
                        from: indexer.safe_lengths().height,
                        mappings: self.mappings.as_ref(),
                        price: self.price.as_ref(),
                    },
                    context,
                )
            })?;
            timed(Phase::Compute, HOLDERS_ID, || {
                self.holders.compute(
                    HoldersDependencies {
                        history: &history,
                        from: indexer.safe_lengths().height,
                        age: &self.age,
                        mappings: &self.mappings,
                        price: &self.price,
                    },
                    context,
                )
            })?;
            let urpd = ReplayInputs {
                history: &history,
                prices: &self.price.spot.cents.height,
                timestamps: &self.mappings.timestamp.monotonic,
            };
            let entry_prices = self.price.spot.cents.height.read_only_boxed_clone();
            let entry_timestamps = self.mappings.timestamp.monotonic.read_only_boxed_clone();
            let capitalized_price = self
                .holders
                .cohorts
                .all
                .realized
                .capitalized_price
                .cents
                .height
                .read_only_boxed_clone();
            thread::scope(|scope| -> Result<()> {
                let entry = scope.spawn(|| {
                    timed(Phase::Compute, ENTRY_ID, || {
                        self.entry.compute(
                            EntryDependencies {
                                history: &history,
                                from: indexer.safe_lengths().height,
                                prices: &entry_prices,
                                timestamps: &entry_timestamps,
                                capitalized_price: &capitalized_price,
                            },
                            context,
                        )
                    })
                });
                let coinflow = scope.spawn(|| -> Result<_> {
                    timed(Phase::Compute, COINFLOW_ID, || {
                        self.coinflow.compute(
                            CoinflowDependencies {
                                urpd,
                                indexer,
                                mappings: self.mappings.as_ref(),
                                age: self.age.as_ref(),
                            },
                            context,
                        )
                    })?;
                    Ok(self.coinflow.as_ref())
                });
                let capital_sentiment = scope.spawn(|| -> Result<_> {
                    let market = market.join().unwrap()?;
                    timed(Phase::Compute, CAPITAL_SENTIMENT_ID, || {
                        self.capital_sentiment.compute(
                            CapitalSentimentDependencies {
                                indexer,
                                price: self.price.as_ref(),
                                holders: self.holders.as_ref(),
                                moving_average: &market.moving_average,
                            },
                            context,
                        )
                    })?;
                    Ok(market)
                });
                let mining = tx_mining_op_return.join().unwrap()?;
                let pools = scope.spawn(|| {
                    timed(Phase::Compute, POOLS_ID, || {
                        self.pools.compute(
                            PoolsDependencies {
                                indexer,
                                price: &self.price,
                                mining,
                            },
                            context,
                        )
                    })
                });
                timed(Phase::Compute, SUPPLY_ID, || {
                    self.supply.compute(
                        SupplyDependencies {
                            indexer,
                            outputs: self.outputs.as_ref(),
                            mining,
                            price: self.price.as_ref(),
                        },
                        context,
                    )
                })?;

                timed(Phase::Compute, COINTIME_ID, || {
                    self.cointime.compute(
                        CointimeDependencies {
                            urpd,
                            indexer,
                            price: self.price.as_ref(),
                            blocks: self.blocks.as_ref(),
                            inflation_rate: &self.supply.inflation_rate,
                            velocity_native: &self.supply.velocity.native,
                            velocity_fiat: &self.supply.velocity.fiat,
                            age: self.age.as_ref(),
                            holders: self.holders.as_ref(),
                        },
                        context,
                    )
                })?;
                let coinflow = coinflow.join().unwrap()?;

                timed(Phase::Compute, BEDROCK_ID, || {
                    self.bedrock.compute(
                        BedrockDependencies {
                            urpd,
                            indexer,
                            mappings: &self.mappings,
                            age: &self.age,
                            holders: &self.holders,
                            cointime: &self.cointime,
                            coinflow,
                        },
                        context,
                    )
                })?;
                timed(Phase::Compute, RARITY_METER_ID, || {
                    self.rarity_meter.compute(
                        RarityMeterDependencies {
                            indexer,
                            bedrock: self.bedrock.as_ref(),
                            holders: self.holders.as_ref(),
                            cointime: self.cointime.as_ref(),
                            coinflow,
                            price: self.price.as_ref(),
                        },
                        context,
                    )
                })?;
                let market = capital_sentiment.join().unwrap()?;
                let utxos = utxos.join().unwrap()?;
                addresses.join().unwrap()?;
                timed(Phase::Compute, INDICATORS_ID, || {
                    self.indicators.compute(
                        IndicatorsDependencies {
                            utxos,
                            indexer,
                            mining,
                            age: self.age.as_ref(),
                            holders: self.holders.as_ref(),
                            market,
                        },
                        context,
                    )
                })?;
                pools.join().unwrap()?;
                entry.join().unwrap()?;
                Ok(())
            })?;
            Ok(())
        })?;

        Ok(())
    }
}

impl ComputePluginSet for DefaultPlugins {
    fn publication(&self) -> &Publication {
        self.indexer.publication()
    }

    fn bootstrap_compute(&mut self, context: UpdateContext<'_>) -> Result<BootstrapAction> {
        let blocks_behind = if cfg!(debug_assertions) {
            0
        } else {
            let chain_height = self.indexer.reader().client().get_last_height()?;
            // Local progress: the published height can sit at a rollback floor.
            chain_height.saturating_sub(*self.indexer.vecs().next_height())
        };

        if blocks_behind > REIMPORT_THRESHOLD {
            info!(
                "Indexing {blocks_behind} blocks before initializing metrics; starting in 10 seconds..."
            );
            thread::sleep(Duration::from_secs(10));

            self.compute_indexer(context)?;
            return Ok(BootstrapAction::Reimport);
        }

        self.compute_inner(context)?;

        Ok(BootstrapAction::Ready)
    }

    fn compute(&mut self, context: UpdateContext<'_>) -> Result<()> {
        self.compute_inner(context)
    }

    fn commit(&mut self, complete: bool) -> Result<()> {
        self.indexer.commit(complete)
    }
}
