use std::{thread, time::Duration};

use bitview_compute::CACHE_BUDGET;
use bitview_plugin::ComputePlugin;
use bitview_plugin_bedrock::Dependencies as BedrockDependencies;
use bitview_plugin_blocks::Dependencies as BlocksDependencies;
use bitview_plugin_capital_sentiment::Dependencies as CapitalSentimentDependencies;
use bitview_plugin_coinflow::Dependencies as CoinflowDependencies;
use bitview_plugin_cointime::Dependencies as CointimeDependencies;
use bitview_plugin_distribution::{Dependencies as DistributionDependencies, UTXOStates};
use bitview_plugin_indexes::Dependencies as IndexesDependencies;
use bitview_plugin_indicators::Dependencies as IndicatorsDependencies;
use bitview_plugin_inputs::Dependencies as InputsDependencies;
use bitview_plugin_market::Dependencies as MarketDependencies;
use bitview_plugin_mining::Dependencies as MiningDependencies;
use bitview_plugin_op_return::Dependencies as OpReturnDependencies;
use bitview_plugin_outputs::Dependencies as OutputsDependencies;
use bitview_plugin_pools::Dependencies as PoolsDependencies;
use bitview_plugin_price::Dependencies as PriceDependencies;
use bitview_plugin_rarity_meter::Dependencies as RarityMeterDependencies;
use bitview_plugin_supply::Dependencies as SupplyDependencies;
use bitview_plugin_transactions::Dependencies as TransactionsDependencies;
use bitview_runtime::{BootstrapAction, ComputePluginSet};
use brk_alloc::Mimalloc;
use brk_error::Result;
use rayon::join;
use tracing::info;
use vecdb::Exit;

use crate::{DefaultPlugins, timed};

const REIMPORT_THRESHOLD: u32 = 10_000;

impl DefaultPlugins {
    /// Updates every plugin, enabling indexer collision checks in debug builds.
    fn compute_inner(&mut self, exit: &Exit) -> Result<()> {
        self.indexer.compute((), exit)?;
        Mimalloc::collect();
        CACHE_BUDGET.invalidate();

        let indexer = self.indexer.as_ref();

        timed("Computed indexes", || {
            self.indexes.compute(IndexesDependencies { indexer }, exit)
        })?;

        thread::scope(|scope| -> Result<()> {
            timed("Computed blocks", || {
                self.blocks.compute(BlocksDependencies { indexer }, exit)
            })?;

            let (inputs_result, prices_result) = join(
                || {
                    timed("Computed inputs", || {
                        self.inputs.compute(
                            InputsDependencies {
                                indexer,
                                blocks: self.blocks.as_ref(),
                            },
                            exit,
                        )
                    })
                },
                || {
                    timed("Computed price", || {
                        self.price.compute(PriceDependencies { indexer }, exit)
                    })
                },
            );
            inputs_result?;
            prices_result?;

            // market, outputs, and (transactions → mining + OP_RETURN) are pairwise
            // independent. Run all three in parallel.
            let market = scope.spawn(|| {
                timed("Computed market", || {
                    self.market.compute(
                        MarketDependencies {
                            indexer,
                            price: self.price.as_ref(),
                            indexes: self.indexes.as_ref(),
                            blocks: self.blocks.as_ref(),
                        },
                        exit,
                    )
                })
            });

            let tx_mining_op_return = scope.spawn(|| -> Result<()> {
                timed("Computed transactions", || {
                    self.transactions.compute(
                        TransactionsDependencies {
                            indexer,
                            inputs: self.inputs.as_ref(),
                            indexes: self.indexes.as_ref(),
                            blocks: self.blocks.as_ref(),
                            price: self.price.as_ref(),
                        },
                        exit,
                    )
                })?;

                let (mining, op_return) = join(
                    || {
                        timed("Computed mining", || {
                            self.mining.compute(
                                MiningDependencies {
                                    indexer,
                                    indexes: self.indexes.as_ref(),
                                    blocks: self.blocks.as_ref(),
                                    transactions: self.transactions.as_ref(),
                                    price: self.price.as_ref(),
                                },
                                exit,
                            )
                        })
                    },
                    || {
                        timed("Computed OP_RETURN", || {
                            self.op_return.compute(
                                OpReturnDependencies {
                                    indexer,
                                    fees: &self.transactions.fees,
                                },
                                exit,
                            )
                        })
                    },
                );
                mining?;
                op_return?;
                Ok(())
            });

            timed("Computed outputs", || {
                self.outputs.compute(
                    OutputsDependencies {
                        indexer,
                        inputs: self.inputs.as_ref(),
                        blocks: self.blocks.as_ref(),
                        price: self.price.as_ref(),
                    },
                    exit,
                )
            })?;

            tx_mining_op_return.join().unwrap()?;
            market.join().unwrap()?;
            Ok(())
        })?;

        self.investing.compute((), exit)?;

        let utxo_states = thread::scope(|scope| -> Result<UTXOStates> {
            let pools = scope.spawn(|| {
                timed("Computed pools", || {
                    self.pools.compute(
                        PoolsDependencies {
                            indexer,
                            indexes: self.indexes.as_ref(),
                            price: self.price.as_ref(),
                            mining: self.mining.as_ref(),
                        },
                        exit,
                    )
                })
            });

            let utxo_states = timed("Computed distribution", || {
                self.distribution.compute(
                    DistributionDependencies {
                        indexer,
                        indexes: self.indexes.as_ref(),
                        inputs: self.inputs.as_ref(),
                        outputs: self.outputs.as_ref(),
                        transactions: self.transactions.as_ref(),
                        price: self.price.as_ref(),
                    },
                    exit,
                )
            })?;

            pools.join().unwrap()?;
            Ok(utxo_states)
        })?;

        // Supply, Coinflow, Capital Sentiment, and indicators are independent.
        // Cointime follows supply; Bedrock and Rarity Meter then consume both
        // Cointime and Coinflow.
        thread::scope(|scope| -> Result<()> {
            let indicators = scope.spawn(|| {
                timed("Computed indicators", || {
                    self.indicators.compute(
                        IndicatorsDependencies {
                            indexer,
                            mining: self.mining.as_ref(),
                            distribution: self.distribution.as_ref(),
                            market: self.market.as_ref(),
                        },
                        exit,
                    )
                })
            });
            let capital_sentiment = scope.spawn(|| {
                timed("Computed capital sentiment", || {
                    self.capital_sentiment.compute(
                        CapitalSentimentDependencies {
                            indexer,
                            indexes: self.indexes.as_ref(),
                            price: self.price.as_ref(),
                            distribution: self.distribution.as_ref(),
                            moving_average: &self.market.moving_average,
                        },
                        exit,
                    )
                })
            });

            let (supply, coinflow) = join(
                || {
                    timed("Computed supply", || {
                        self.supply.compute(
                            SupplyDependencies {
                                indexer,
                                outputs: self.outputs.as_ref(),
                                mining: self.mining.as_ref(),
                                price: self.price.as_ref(),
                            },
                            exit,
                        )
                    })
                },
                || {
                    timed("Computed coinflow", || {
                        self.coinflow.compute(
                            CoinflowDependencies {
                                indexer,
                                indexes: self.indexes.as_ref(),
                                distribution: self.distribution.as_ref(),
                            },
                            exit,
                        )
                    })
                },
            );
            supply?;
            coinflow?;

            timed("Computed cointime", || {
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
                    exit,
                )
            })?;

            let (bedrock, rarity_meter) = join(
                || {
                    timed("Computed bedrock", || {
                        self.bedrock.compute(
                            BedrockDependencies {
                                indexer,
                                indexes: self.indexes.as_ref(),
                                distribution: self.distribution.as_ref(),
                                utxo_states: &utxo_states,
                                cointime: self.cointime.as_ref(),
                                coinflow: self.coinflow.as_ref(),
                            },
                            exit,
                        )
                    })
                },
                || {
                    timed("Computed rarity meter", || {
                        self.rarity_meter.compute(
                            RarityMeterDependencies {
                                indexer,
                                distribution: self.distribution.as_ref(),
                                cointime: self.cointime.as_ref(),
                                coinflow: self.coinflow.as_ref(),
                                price: self.price.as_ref(),
                            },
                            exit,
                        )
                    })
                },
            );
            bedrock?;
            rarity_meter?;
            capital_sentiment.join().unwrap()?;
            indicators.join().unwrap()?;
            Ok(())
        })?;

        Ok(())
    }
}

impl ComputePluginSet for DefaultPlugins {
    fn bootstrap_compute(&mut self, exit: &Exit) -> Result<BootstrapAction> {
        let blocks_behind = if cfg!(debug_assertions) {
            0
        } else {
            let chain_height = self.indexer.reader().client().get_last_height()?;
            chain_height.saturating_sub(*self.indexer.indexed_height())
        };

        if blocks_behind > REIMPORT_THRESHOLD {
            info!("---");
            info!("Computing {blocks_behind} blocks before starting server...");
            info!("---");
            thread::sleep(Duration::from_secs(10));
        }

        self.compute_inner(exit)?;

        Ok(if blocks_behind > REIMPORT_THRESHOLD {
            BootstrapAction::Reimport
        } else {
            BootstrapAction::Ready
        })
    }

    fn compute(&mut self, exit: &Exit) -> Result<()> {
        self.compute_inner(exit)
    }

    fn prepare_publish(&mut self) -> Result<()> {
        self.indexer.prepare_publish()
    }
}
