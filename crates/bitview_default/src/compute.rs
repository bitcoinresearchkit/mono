use std::{thread, time::Duration};

use bitview_compute::CACHE_BUDGET;
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_bedrock::{Dependencies as BedrockDependencies, ID as BEDROCK_ID};
use bitview_plugin_blocks::{Dependencies as BlocksDependencies, ID as BLOCKS_ID};
use bitview_plugin_capital_sentiment::{
    Dependencies as CapitalSentimentDependencies, ID as CAPITAL_SENTIMENT_ID,
};
use bitview_plugin_coinflow::{Dependencies as CoinflowDependencies, ID as COINFLOW_ID};
use bitview_plugin_cointime::{Dependencies as CointimeDependencies, ID as COINTIME_ID};
use bitview_plugin_distribution::{
    Dependencies as DistributionDependencies, ID as DISTRIBUTION_ID, UTXOStates,
};
use bitview_plugin_indexer::ID as INDEXER_ID;
use bitview_plugin_indicators::{Dependencies as IndicatorsDependencies, ID as INDICATORS_ID};
use bitview_plugin_inputs::{Dependencies as InputsDependencies, ID as INPUTS_ID};
use bitview_plugin_investing::ID as INVESTING_ID;
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
use bitview_runtime::{BootstrapAction, ComputePluginSet};
use brk_alloc::Mimalloc;
use brk_error::Result;
use rayon::join;
use tracing::info;

use crate::{
    DefaultPlugins,
    timing::{Phase, timed},
};

const REIMPORT_THRESHOLD: u32 = 10_000;

impl DefaultPlugins {
    /// Updates every plugin, enabling indexer collision checks in debug builds.
    fn compute_inner(&mut self, context: UpdateContext<'_>) -> Result<()> {
        self.compute_indexer(context)?;
        Mimalloc::collect();
        self.compute_dependents(context)
    }

    fn compute_indexer(&mut self, context: UpdateContext<'_>) -> Result<()> {
        timed(Phase::Compute, INDEXER_ID, || {
            self.indexer.compute((), context)
        })
    }

    fn compute_dependents(&mut self, context: UpdateContext<'_>) -> Result<()> {
        CACHE_BUDGET.invalidate();

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

            // market, outputs, and (transactions → mining + OP_RETURN) are pairwise
            // independent. Run all three in parallel.
            let market = scope.spawn(|| {
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
                })
            });

            let tx_mining_op_return = scope.spawn(|| -> Result<()> {
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
                                    mappings: self.mappings.as_ref(),
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
                Ok(())
            });

            timed(Phase::Compute, OUTPUTS_ID, || {
                self.outputs.compute(
                    OutputsDependencies {
                        indexer,
                        inputs: self.inputs.as_ref(),
                        blocks: self.blocks.as_ref(),
                        price: self.price.as_ref(),
                    },
                    context,
                )
            })?;

            tx_mining_op_return.join().unwrap()?;
            market.join().unwrap()?;
            Ok(())
        })?;

        timed(Phase::Compute, INVESTING_ID, || {
            self.investing.compute((), context)
        })?;

        let utxo_states = thread::scope(|scope| -> Result<UTXOStates> {
            let pools = scope.spawn(|| {
                timed(Phase::Compute, POOLS_ID, || {
                    self.pools.compute(
                        PoolsDependencies {
                            indexer,
                            price: self.price.as_ref(),
                            mining: self.mining.as_ref(),
                        },
                        context,
                    )
                })
            });

            let utxo_states = timed(Phase::Compute, DISTRIBUTION_ID, || {
                self.distribution.compute(
                    DistributionDependencies {
                        indexer,
                        mappings: self.mappings.as_ref(),
                        inputs: self.inputs.as_ref(),
                        outputs: self.outputs.as_ref(),
                        transactions: self.transactions.as_ref(),
                        price: self.price.as_ref(),
                    },
                    context,
                )
            })?;

            pools.join().unwrap()?;
            Ok(utxo_states)
        })?;

        // Supply feeds Cointime while Coinflow is independent. Bedrock and
        // Rarity Meter then consume both Cointime and Coinflow.
        thread::scope(|scope| -> Result<()> {
            let indicators = scope.spawn(|| {
                timed(Phase::Compute, INDICATORS_ID, || {
                    self.indicators.compute(
                        IndicatorsDependencies {
                            indexer,
                            mining: self.mining.as_ref(),
                            distribution: self.distribution.as_ref(),
                            market: self.market.as_ref(),
                        },
                        context,
                    )
                })
            });
            let capital_sentiment = scope.spawn(|| {
                timed(Phase::Compute, CAPITAL_SENTIMENT_ID, || {
                    self.capital_sentiment.compute(
                        CapitalSentimentDependencies {
                            indexer,
                            mappings: self.mappings.as_ref(),
                            price: self.price.as_ref(),
                            distribution: self.distribution.as_ref(),
                            moving_average: &self.market.moving_average,
                        },
                        context,
                    )
                })
            });

            let (cointime, coinflow) = join(
                || {
                    timed(Phase::Compute, SUPPLY_ID, || {
                        self.supply.compute(
                            SupplyDependencies {
                                indexer,
                                outputs: self.outputs.as_ref(),
                                mining: self.mining.as_ref(),
                                price: self.price.as_ref(),
                            },
                            context,
                        )
                    })?;

                    timed(Phase::Compute, COINTIME_ID, || {
                        self.cointime.compute(
                            CointimeDependencies {
                                indexer,
                                price: self.price.as_ref(),
                                blocks: self.blocks.as_ref(),
                                inflation_rate: &self.supply.inflation_rate,
                                velocity_native: &self.supply.velocity.native,
                                velocity_fiat: &self.supply.velocity.fiat,
                                distribution: self.distribution.as_ref(),
                            },
                            context,
                        )
                    })
                },
                || {
                    timed(Phase::Compute, COINFLOW_ID, || {
                        self.coinflow.compute(
                            CoinflowDependencies {
                                indexer,
                                mappings: self.mappings.as_ref(),
                                distribution: self.distribution.as_ref(),
                            },
                            context,
                        )
                    })
                },
            );
            cointime?;
            coinflow?;

            timed(Phase::Compute, BEDROCK_ID, || {
                self.bedrock.compute(
                    BedrockDependencies {
                        indexer,
                        mappings: self.mappings.as_ref(),
                        distribution: self.distribution.as_ref(),
                        utxo_states: &utxo_states,
                        cointime: self.cointime.as_ref(),
                        coinflow: self.coinflow.as_ref(),
                    },
                    context,
                )
            })?;
            timed(Phase::Compute, RARITY_METER_ID, || {
                self.rarity_meter.compute(
                    RarityMeterDependencies {
                        indexer,
                        bedrock: self.bedrock.as_ref(),
                        distribution: self.distribution.as_ref(),
                        cointime: self.cointime.as_ref(),
                        coinflow: self.coinflow.as_ref(),
                        price: self.price.as_ref(),
                    },
                    context,
                )
            })?;
            capital_sentiment.join().unwrap()?;
            indicators.join().unwrap()?;
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
            chain_height.saturating_sub(*self.indexer.indexed_height())
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

    fn commit(&mut self) -> Result<()> {
        self.indexer.commit()
    }
}
