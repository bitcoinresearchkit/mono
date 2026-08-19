#![doc = include_str!("../README.md")]
#![allow(clippy::type_complexity)]

use brk_error::Result;

use std::{fs, path::Path, thread, time::Instant};

use bitview_plugin::{ComputePlugin, Plugin};
use bitview_traversable::Traversable;
use brk_indexer::Indexer;
use brk_types::{Height, Version};
use tracing::info;
use vecdb::{AnyExportableVec, Exit, Ro, Rw, StorageMode};

use bitview_plugin_distribution::UTXOStates;

#[derive(Traversable)]
pub struct Computer<M: StorageMode = Rw> {
    pub blocks: Box<bitview_plugin_blocks::Vecs<M>>,
    pub mining: Box<bitview_plugin_mining::Vecs<M>>,
    pub transactions: Box<bitview_plugin_transactions::Vecs<M>>,
    pub cointime: Box<bitview_plugin_cointime::Vecs<M>>,
    pub coinflow: Box<bitview_plugin_coinflow::Vecs<M>>,
    pub bedrock: Box<bitview_plugin_bedrock::Vecs<M>>,
    pub capital_sentiment: Box<bitview_plugin_capital_sentiment::Vecs<M>>,
    pub rarity_meter: Box<bitview_plugin_rarity_meter::Vecs<M>>,
    pub constants: Box<bitview_plugin_constants::Vecs>,
    pub indexes: Box<bitview_plugin_indexes::Vecs<M>>,
    pub indicators: Box<bitview_plugin_indicators::Vecs<M>>,
    pub investing: Box<bitview_plugin_investing::Vecs>,
    pub market: Box<bitview_plugin_market::Vecs<M>>,
    pub pools: Box<bitview_plugin_pools::Vecs<M>>,
    pub price: Box<bitview_plugin_price::Vecs<M>>,
    #[traversable(flatten)]
    pub distribution: Box<bitview_plugin_distribution::Vecs<M>>,
    pub supply: Box<bitview_plugin_supply::Vecs<M>>,
    pub inputs: Box<bitview_plugin_inputs::Vecs<M>>,
    pub outputs: Box<bitview_plugin_outputs::Vecs<M>>,
    pub op_return: Box<bitview_plugin_op_return::Vecs<M>>,
}

const VERSION: Version = Version::new(9);

impl Computer {
    pub fn forced_import(outputs_path: &Path, indexer: &Indexer) -> Result<Self> {
        info!("Importing computer...");
        let import_start = Instant::now();

        let computed_path = outputs_path.join("computed");

        const STACK_SIZE: usize = 8 * 1024 * 1024;
        let big_thread = || thread::Builder::new().stack_size(STACK_SIZE);

        let indexes = timed("Imported indexes", || -> Result<_> {
            Ok(Box::new(bitview_plugin_indexes::Vecs::forced_import(
                &computed_path,
                VERSION,
                indexer,
            )?))
        })?;

        let (constants, price) = timed("Imported price/constants", || -> Result<_> {
            let constants = Box::new(bitview_plugin_constants::Vecs::new(VERSION, &indexes));
            let price = Box::new(bitview_plugin_price::Vecs::forced_import(
                &computed_path,
                VERSION,
                &indexes,
            )?);
            Ok((constants, price))
        })?;

        let blocks = timed("Imported blocks", || -> Result<_> {
            Ok(Box::new(bitview_plugin_blocks::Vecs::forced_import(
                &computed_path,
                VERSION,
                indexer,
                &indexes,
            )?))
        })?;

        let cached_starts = blocks.lookback.cached_window_starts();

        let (inputs, outputs, mining, transactions, pools, op_return) =
            timed("Imported inputs/outputs/mining/tx/pools/op_return", || {
                thread::scope(|s| -> Result<_> {
                    let inputs_handle = big_thread().spawn_scoped(s, || -> Result<_> {
                        Ok(Box::new(bitview_plugin_inputs::Vecs::forced_import(
                            &computed_path,
                            VERSION,
                            &indexes,
                            &cached_starts,
                        )?))
                    })?;

                    let outputs_handle = big_thread().spawn_scoped(s, || -> Result<_> {
                        Ok(Box::new(bitview_plugin_outputs::Vecs::forced_import(
                            &computed_path,
                            VERSION,
                            &indexes,
                            &cached_starts,
                        )?))
                    })?;

                    let mining_handle = big_thread().spawn_scoped(s, || -> Result<_> {
                        Ok(Box::new(bitview_plugin_mining::Vecs::forced_import(
                            &computed_path,
                            VERSION,
                            indexer,
                            &indexes,
                            &cached_starts,
                        )?))
                    })?;

                    let transactions_handle = big_thread().spawn_scoped(s, || -> Result<_> {
                        Ok(Box::new(bitview_plugin_transactions::Vecs::forced_import(
                            &computed_path,
                            VERSION,
                            indexer,
                            &indexes,
                            &cached_starts,
                        )?))
                    })?;

                    let pools_handle = big_thread().spawn_scoped(s, || -> Result<_> {
                        Ok(Box::new(bitview_plugin_pools::Vecs::forced_import(
                            &computed_path,
                            VERSION,
                            &indexes,
                            &cached_starts,
                        )?))
                    })?;

                    let mining = mining_handle.join().unwrap()?;
                    let block_size = blocks.size.size.cached_cumulative();
                    let chain_fees = mining.rewards.fees.cached_cumulative_sats();
                    let op_return_handle = big_thread().spawn_scoped(s, || -> Result<_> {
                        Ok(Box::new(bitview_plugin_op_return::Vecs::forced_import(
                            &computed_path,
                            VERSION,
                            &indexes,
                            &cached_starts,
                            block_size,
                            chain_fees,
                        )?))
                    })?;
                    let inputs = inputs_handle.join().unwrap()?;
                    let outputs = outputs_handle.join().unwrap()?;
                    let transactions = transactions_handle.join().unwrap()?;
                    let pools = pools_handle.join().unwrap()?;
                    let op_return = op_return_handle.join().unwrap()?;

                    Ok((inputs, outputs, mining, transactions, pools, op_return))
                })
            })?;

        // Market, investing, and distribution are independent; import in parallel.
        let (distribution, market, investing) =
            timed("Imported distribution/market/investing", || {
                thread::scope(|s| -> Result<_> {
                    let market_handle = big_thread().spawn_scoped(s, || -> Result<_> {
                        Ok(Box::new(bitview_plugin_market::Vecs::forced_import(
                            &computed_path,
                            VERSION,
                            &indexes,
                            &blocks,
                            &price,
                        )?))
                    })?;

                    let investing_handle = big_thread().spawn_scoped(s, || -> Result<_> {
                        Ok(Box::new(bitview_plugin_investing::Vecs::forced_import(
                            VERSION, &indexes, &blocks, &price,
                        )?))
                    })?;

                    let distribution = Box::new(bitview_plugin_distribution::Vecs::forced_import(
                        &computed_path,
                        VERSION,
                        &indexes,
                        &cached_starts,
                        &price,
                        &inputs.by_type,
                        &outputs.by_type,
                    )?);

                    let market = market_handle.join().unwrap()?;
                    let investing = investing_handle.join().unwrap()?;
                    Ok((distribution, market, investing))
                })
            })?;

        let all_chain = distribution.all_chain_sources();

        let (cointime, coinflow, bedrock, capital_sentiment, indicators) = timed(
            "Imported cointime/coinflow/bedrock/capital sentiment/indicators",
            || {
                thread::scope(|s| -> Result<_> {
                    let cointime = big_thread().spawn_scoped(s, || -> Result<_> {
                        Ok(Box::new(bitview_plugin_cointime::Vecs::forced_import(
                            &computed_path,
                            VERSION,
                            &indexes,
                            &cached_starts,
                            &price,
                            &mining.rewards.subsidy.cumulative.cents,
                            &all_chain,
                        )?))
                    })?;
                    let coinflow = big_thread().spawn_scoped(s, || -> Result<_> {
                        Ok(Box::new(bitview_plugin_coinflow::Vecs::forced_import(
                            &computed_path,
                            VERSION,
                            &indexes,
                            &price,
                        )?))
                    })?;
                    let bedrock = big_thread().spawn_scoped(s, || -> Result<_> {
                        Ok(Box::new(bitview_plugin_bedrock::Vecs::forced_import(
                            &computed_path,
                            VERSION,
                            &indexes,
                        )?))
                    })?;
                    let capital_sentiment = big_thread().spawn_scoped(s, || -> Result<_> {
                        Ok(Box::new(
                            bitview_plugin_capital_sentiment::Vecs::forced_import(
                                &computed_path,
                                VERSION,
                                &indexes,
                            )?,
                        ))
                    })?;
                    let indicators = Box::new(bitview_plugin_indicators::Vecs::forced_import(
                        &computed_path,
                        VERSION,
                        &indexes,
                        &all_chain,
                        &mining,
                        &distribution,
                        &transactions,
                    )?);
                    Ok((
                        cointime.join().unwrap()?,
                        coinflow.join().unwrap()?,
                        bedrock.join().unwrap()?,
                        capital_sentiment.join().unwrap()?,
                        indicators,
                    ))
                })
            },
        )?;

        let (supply, rarity_meter) = timed("Imported supply/rarity meter", || {
            thread::scope(|s| -> Result<_> {
                let supply = big_thread().spawn_scoped(s, || -> Result<_> {
                    Ok(Box::new(bitview_plugin_supply::Vecs::forced_import(
                        &computed_path,
                        VERSION,
                        &indexes,
                        &cached_starts,
                        &distribution,
                        &cointime,
                        &all_chain,
                        &transactions,
                    )?))
                })?;
                let rarity_meter = big_thread().spawn_scoped(s, || -> Result<_> {
                    Ok(Box::new(bitview_plugin_rarity_meter::Vecs::forced_import(
                        &computed_path,
                        VERSION,
                        &indexes,
                        &distribution,
                        &cointime,
                        &coinflow,
                    )?))
                })?;
                Ok((supply.join().unwrap()?, rarity_meter.join().unwrap()?))
            })
        })?;

        info!("Total import time: {:?}", import_start.elapsed());

        let this = Self {
            blocks,
            mining,
            transactions,
            constants,
            indicators,
            investing,
            market,
            distribution,
            supply,
            pools,
            cointime,
            coinflow,
            bedrock,
            capital_sentiment,
            rarity_meter,
            indexes,
            inputs,
            price,
            outputs,
            op_return,
        };

        Self::retain_databases(&computed_path)?;

        Ok(this)
    }

    /// Removes database folders that are no longer in use.
    fn retain_databases(computed_path: &Path) -> Result<()> {
        const EXPECTED_DBS: &[&str] = &[
            bitview_plugin_blocks::ID.as_str(),
            bitview_plugin_mining::ID.as_str(),
            bitview_plugin_transactions::ID.as_str(),
            bitview_plugin_cointime::ID.as_str(),
            bitview_plugin_coinflow::ID.as_str(),
            bitview_plugin_bedrock::ID.as_str(),
            bitview_plugin_capital_sentiment::ID.as_str(),
            bitview_plugin_rarity_meter::ID.as_str(),
            bitview_plugin_indicators::ID.as_str(),
            bitview_plugin_indexes::ID.as_str(),
            bitview_plugin_market::ID.as_str(),
            bitview_plugin_pools::ID.as_str(),
            bitview_plugin_price::ID.as_str(),
            bitview_plugin_distribution::ID.as_str(),
            bitview_plugin_supply::ID.as_str(),
            bitview_plugin_inputs::ID.as_str(),
            bitview_plugin_outputs::ID.as_str(),
            bitview_plugin_op_return::ID.as_str(),
        ];

        if !computed_path.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(computed_path)? {
            let entry = entry?;
            let file_type = entry.file_type()?;

            if !file_type.is_dir() {
                continue;
            }

            if let Some(name) = entry.file_name().to_str()
                && !name.starts_with('_')
                && !EXPECTED_DBS.contains(&name)
            {
                info!("Removing obsolete database folder: {}", name);
                let path = entry.path();
                fs::remove_dir_all(&path)
                    .map_err(|e| std::io::Error::other(format!("remove_dir_all {path:?}: {e}")))?;
            }
        }

        Ok(())
    }

    pub fn compute(&mut self, indexer: &mut Indexer, exit: &Exit) -> Result<()> {
        indexer.begin_update();
        bitview_compute::CACHE_BUDGET.invalidate();

        let compute_start = Instant::now();

        timed("Computed indexes", || {
            compute_plugin(
                self.indexes.as_mut(),
                bitview_plugin_indexes::Dependencies { indexer },
                exit,
            )
        })?;

        thread::scope(|scope| -> Result<()> {
            timed("Computed blocks", || {
                compute_plugin(
                    self.blocks.as_mut(),
                    bitview_plugin_blocks::Dependencies { indexer },
                    exit,
                )
            })?;

            let (inputs_result, prices_result) = rayon::join(
                || {
                    timed("Computed inputs", || {
                        compute_plugin(
                            self.inputs.as_mut(),
                            bitview_plugin_inputs::Dependencies {
                                indexer,
                                blocks: self.blocks.as_ref(),
                            },
                            exit,
                        )
                    })
                },
                || {
                    timed("Computed price", || {
                        compute_plugin(
                            self.price.as_mut(),
                            bitview_plugin_price::Dependencies { indexer },
                            exit,
                        )
                    })
                },
            );
            inputs_result?;
            prices_result?;

            // market, outputs, and (transactions → mining + OP_RETURN) are pairwise
            // independent. Run all three in parallel.
            let market = scope.spawn(|| {
                timed("Computed market", || {
                    compute_plugin(
                        self.market.as_mut(),
                        bitview_plugin_market::Dependencies {
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
                    compute_plugin(
                        self.transactions.as_mut(),
                        bitview_plugin_transactions::Dependencies {
                            indexer,
                            inputs: self.inputs.as_ref(),
                            indexes: self.indexes.as_ref(),
                            blocks: self.blocks.as_ref(),
                            price: self.price.as_ref(),
                        },
                        exit,
                    )
                })?;

                let (mining, op_return) = rayon::join(
                    || {
                        timed("Computed mining", || {
                            compute_plugin(
                                self.mining.as_mut(),
                                bitview_plugin_mining::Dependencies {
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
                            compute_plugin(
                                self.op_return.as_mut(),
                                bitview_plugin_op_return::Dependencies {
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
                compute_plugin(
                    self.outputs.as_mut(),
                    bitview_plugin_outputs::Dependencies {
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

        compute_plugin(self.investing.as_mut(), (), exit)?;

        let utxo_states = thread::scope(|scope| -> Result<UTXOStates> {
            let pools = scope.spawn(|| {
                timed("Computed pools", || {
                    compute_plugin(
                        self.pools.as_mut(),
                        bitview_plugin_pools::Dependencies {
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
                compute_plugin(
                    self.distribution.as_mut(),
                    bitview_plugin_distribution::Dependencies {
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
                    compute_plugin(
                        self.indicators.as_mut(),
                        bitview_plugin_indicators::Dependencies {
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
                    compute_plugin(
                        self.capital_sentiment.as_mut(),
                        bitview_plugin_capital_sentiment::Dependencies {
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

            let (supply, coinflow) = rayon::join(
                || {
                    timed("Computed supply", || {
                        compute_plugin(
                            self.supply.as_mut(),
                            bitview_plugin_supply::Dependencies {
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
                        compute_plugin(
                            self.coinflow.as_mut(),
                            bitview_plugin_coinflow::Dependencies {
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
                compute_plugin(
                    self.cointime.as_mut(),
                    bitview_plugin_cointime::Dependencies {
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

            let (bedrock, rarity_meter) = rayon::join(
                || {
                    timed("Computed bedrock", || {
                        compute_plugin(
                            self.bedrock.as_mut(),
                            bitview_plugin_bedrock::Dependencies {
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
                        compute_plugin(
                            self.rarity_meter.as_mut(),
                            bitview_plugin_rarity_meter::Dependencies {
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

        indexer.finish_update()?;
        self.finish_update();

        info!("Total compute time: {:?}", compute_start.elapsed());
        Ok(())
    }

    fn finish_update(&self) {
        self.blocks.gate().finish_update();
        self.mining.gate().finish_update();
        self.transactions.gate().finish_update();
        self.cointime.gate().finish_update();
        self.coinflow.gate().finish_update();
        self.bedrock.gate().finish_update();
        self.capital_sentiment.gate().finish_update();
        self.rarity_meter.gate().finish_update();
        self.indexes.gate().finish_update();
        self.indicators.gate().finish_update();
        self.investing.gate().finish_update();
        self.market.gate().finish_update();
        self.pools.gate().finish_update();
        self.price.gate().finish_update();
        self.distribution.gate().finish_update();
        self.supply.gate().finish_update();
        self.inputs.gate().finish_update();
        self.outputs.gate().finish_update();
        self.op_return.gate().finish_update();
    }
}

impl Computer<Ro> {
    /// Live computer stamp for diagnostics. Derived from
    /// `distribution.supply_state`'s stamp. For data reads use
    /// `Query::height` (clamped against the safe-lengths snapshot).
    pub fn computed_height(&self) -> Height {
        Height::from(self.distribution.supply_state.stamp())
    }
}

macro_rules! impl_iter_plugins {
    ($($field:ident),+ $(,)?) => {
        impl_iter_plugins!(@mode Ro, $($field),+);
        impl_iter_plugins!(@mode Rw, $($field),+);
    };
    (@mode $mode:ty, $($field:ident),+) => {
        impl Computer<$mode> {
            pub fn iter_plugin_visible(
                &self,
            ) -> impl Iterator<Item = (&dyn Plugin, &dyn AnyExportableVec)> {
                use bitview_traversable::Traversable;
                std::iter::empty()
                    $(.chain(self.$field.iter_any_visible().map(|v| (self.$field.as_ref() as &dyn Plugin, v))))+
            }
        }
    };
}

impl_iter_plugins!(
    blocks,
    mining,
    transactions,
    cointime,
    coinflow,
    bedrock,
    capital_sentiment,
    rarity_meter,
    constants,
    indicators,
    indexes,
    investing,
    market,
    pools,
    price,
    distribution,
    supply,
    inputs,
    outputs,
    op_return
);

fn compute_plugin<P>(
    plugin: &mut P,
    dependencies: P::Dependencies<'_>,
    exit: &Exit,
) -> Result<P::Output>
where
    P: ComputePlugin,
{
    plugin.gate().begin_update();
    plugin.compute(dependencies, exit)
}

fn timed<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let result = f();
    info!("{label} in {:?}", start.elapsed());
    result
}
