#![doc = include_str!("../README.md")]
#![allow(clippy::type_complexity)]

use std::{fs, path::Path, thread, time::Instant};

use bitview_plugin::Plugin;
use brk_error::Result;
use brk_indexer::Indexer;
use brk_traversable::Traversable;
use brk_types::{Height, Version};
use tracing::info;
use vecdb::{AnyExportableVec, Exit, Ro, Rw, StorageMode};

use distribution::UTXOStates;

mod blocks;
mod constants;
mod distribution;
mod frameworks;
pub mod indexes;
mod indicators;
mod inputs;
mod internal;
mod investing;
mod market;
mod mining;
mod models;
mod op_return;
mod outputs;
mod pools;
pub mod price;
mod supply;
mod transactions;

#[derive(Traversable)]
pub struct Computer<M: StorageMode = Rw> {
    pub blocks: Box<blocks::Vecs<M>>,
    pub mining: Box<mining::Vecs<M>>,
    pub transactions: Box<transactions::Vecs<M>>,
    pub frameworks: Box<frameworks::Vecs<M>>,
    pub models: Box<models::Vecs<M>>,
    pub constants: Box<constants::Vecs>,
    pub indexes: Box<indexes::Vecs<M>>,
    pub indicators: Box<indicators::Vecs<M>>,
    pub investing: Box<investing::Vecs>,
    pub market: Box<market::Vecs<M>>,
    pub pools: Box<pools::Vecs<M>>,
    pub price: Box<price::Vecs<M>>,
    #[traversable(flatten)]
    pub distribution: Box<distribution::Vecs<M>>,
    pub supply: Box<supply::Vecs<M>>,
    pub inputs: Box<inputs::Vecs<M>>,
    pub outputs: Box<outputs::Vecs<M>>,
    pub op_return: Box<op_return::Vecs<M>>,
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
            Ok(Box::new(indexes::Vecs::forced_import(
                &computed_path,
                VERSION,
                indexer,
            )?))
        })?;

        let (constants, price) = timed("Imported price/constants", || -> Result<_> {
            let constants = Box::new(constants::Vecs::new(VERSION, &indexes));
            let price = Box::new(price::Vecs::forced_import(
                &computed_path,
                VERSION,
                &indexes,
            )?);
            Ok((constants, price))
        })?;

        let blocks = timed("Imported blocks", || -> Result<_> {
            Ok(Box::new(blocks::Vecs::forced_import(
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
                        Ok(Box::new(inputs::Vecs::forced_import(
                            &computed_path,
                            VERSION,
                            &indexes,
                            &cached_starts,
                        )?))
                    })?;

                    let outputs_handle = big_thread().spawn_scoped(s, || -> Result<_> {
                        Ok(Box::new(outputs::Vecs::forced_import(
                            &computed_path,
                            VERSION,
                            &indexes,
                            &cached_starts,
                        )?))
                    })?;

                    let mining_handle = big_thread().spawn_scoped(s, || -> Result<_> {
                        Ok(Box::new(mining::Vecs::forced_import(
                            &computed_path,
                            VERSION,
                            indexer,
                            &indexes,
                            &cached_starts,
                        )?))
                    })?;

                    let transactions_handle = big_thread().spawn_scoped(s, || -> Result<_> {
                        Ok(Box::new(transactions::Vecs::forced_import(
                            &computed_path,
                            VERSION,
                            indexer,
                            &indexes,
                            &cached_starts,
                        )?))
                    })?;

                    let pools_handle = big_thread().spawn_scoped(s, || -> Result<_> {
                        Ok(Box::new(pools::Vecs::forced_import(
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
                        Ok(Box::new(op_return::Vecs::forced_import(
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
                        Ok(Box::new(market::Vecs::forced_import(
                            &computed_path,
                            VERSION,
                            &indexes,
                            &blocks,
                            &price,
                        )?))
                    })?;

                    let investing_handle = big_thread().spawn_scoped(s, || -> Result<_> {
                        Ok(Box::new(investing::Vecs::forced_import(
                            VERSION, &indexes, &blocks, &price,
                        )?))
                    })?;

                    let distribution = Box::new(distribution::Vecs::forced_import(
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

        let (frameworks, indicators) = timed("Imported frameworks/indicators", || {
            thread::scope(|s| -> Result<_> {
                let frameworks_handle = big_thread().spawn_scoped(s, || -> Result<_> {
                    Ok(Box::new(frameworks::Vecs::forced_import(
                        &computed_path,
                        VERSION,
                        &indexes,
                        &cached_starts,
                        &price,
                        &mining.rewards.subsidy.cumulative.cents,
                        &all_chain,
                    )?))
                })?;
                let indicators = Box::new(indicators::Vecs::forced_import(
                    &computed_path,
                    VERSION,
                    &indexes,
                    &all_chain,
                    &mining,
                    &distribution,
                    &transactions,
                )?);
                let frameworks = frameworks_handle.join().unwrap()?;
                Ok((frameworks, indicators))
            })
        })?;

        let supply = timed("Imported supply", || -> Result<_> {
            Ok(Box::new(supply::Vecs::forced_import(
                &computed_path,
                VERSION,
                &indexes,
                &cached_starts,
                supply::ImportSources::new(
                    &distribution,
                    &frameworks.cointime,
                    &all_chain,
                    &transactions,
                ),
            )?))
        })?;

        let models = timed("Imported models", || -> Result<_> {
            Ok(Box::new(models::Vecs::forced_import(
                &computed_path,
                VERSION,
                &indexes,
                &distribution,
                &frameworks,
            )?))
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
            frameworks,
            models,
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
            blocks::DB_NAME,
            mining::DB_NAME,
            transactions::DB_NAME,
            frameworks::DB_NAME,
            models::DB_NAME,
            indicators::DB_NAME,
            indexes::DB_NAME,
            market::DB_NAME,
            pools::DB_NAME,
            price::DB_NAME,
            distribution::DB_NAME,
            supply::DB_NAME,
            inputs::DB_NAME,
            outputs::DB_NAME,
            op_return::DB_NAME,
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
        internal::CACHE_BUDGET.invalidate();

        let compute_start = Instant::now();

        self.indexes.gate().begin_update();
        timed("Computed indexes", || self.indexes.compute(indexer, exit))?;

        thread::scope(|scope| -> Result<()> {
            self.blocks.gate().begin_update();
            timed("Computed blocks", || self.blocks.compute(indexer, exit))?;

            let (inputs_result, prices_result) = rayon::join(
                || {
                    self.inputs.gate().begin_update();
                    timed("Computed inputs", || {
                        self.inputs.compute(indexer, &self.blocks, exit)
                    })
                },
                || {
                    self.price.gate().begin_update();
                    timed("Computed price", || self.price.compute(indexer, exit))
                },
            );
            inputs_result?;
            prices_result?;

            // market, outputs, and (transactions → mining + OP_RETURN) are pairwise
            // independent. Run all three in parallel.
            let market = scope.spawn(|| {
                self.market.gate().begin_update();
                timed("Computed market", || {
                    self.market
                        .compute(indexer, &self.price, &self.indexes, &self.blocks, exit)
                })
            });

            let tx_mining_op_return = scope.spawn(|| -> Result<()> {
                self.transactions.gate().begin_update();
                timed("Computed transactions", || {
                    self.transactions.compute(
                        indexer,
                        &self.inputs,
                        &self.indexes,
                        &self.blocks,
                        &self.price,
                        exit,
                    )
                })?;

                let (mining, op_return) = rayon::join(
                    || {
                        self.mining.gate().begin_update();
                        timed("Computed mining", || {
                            self.mining.compute(
                                indexer,
                                &self.indexes,
                                &self.blocks,
                                &self.transactions,
                                &self.price,
                                exit,
                            )
                        })
                    },
                    || {
                        self.op_return.gate().begin_update();
                        timed("Computed OP_RETURN", || {
                            self.op_return
                                .compute(indexer, &self.transactions.fees, exit)
                        })
                    },
                );
                mining?;
                op_return?;
                Ok(())
            });

            self.outputs.gate().begin_update();
            timed("Computed outputs", || {
                self.outputs
                    .compute(indexer, &self.inputs, &self.blocks, &self.price, exit)
            })?;

            tx_mining_op_return.join().unwrap()?;
            market.join().unwrap()?;
            Ok(())
        })?;

        self.investing.gate().begin_update();
        self.investing.invalidate_cache();

        let utxo_states = thread::scope(|scope| -> Result<UTXOStates> {
            let pools = scope.spawn(|| {
                self.pools.gate().begin_update();
                timed("Computed pools", || {
                    self.pools
                        .compute(indexer, &self.indexes, &self.price, &self.mining, exit)
                })
            });

            self.distribution.gate().begin_update();
            let utxo_states = timed("Computed distribution", || {
                self.distribution.compute(
                    indexer,
                    &self.indexes,
                    &self.inputs,
                    &self.outputs,
                    &self.transactions,
                    &self.price,
                    exit,
                )
            })?;

            pools.join().unwrap()?;
            Ok(utxo_states)
        })?;

        // Indicators doesn't depend on supply or either framework — run it in
        // the background alongside their sequential computation.
        thread::scope(|scope| -> Result<()> {
            let indicators = scope.spawn(|| {
                self.indicators.gate().begin_update();
                timed("Computed indicators", || {
                    self.indicators.compute(
                        indexer,
                        &self.mining,
                        &self.distribution,
                        &self.market,
                        exit,
                    )
                })
            });

            self.supply.gate().begin_update();
            timed("Computed supply", || {
                self.supply
                    .compute(indexer, &self.outputs, &self.mining, &self.price, exit)
            })?;

            self.frameworks.gate().begin_update();
            timed("Computed frameworks", || {
                self.frameworks.compute(
                    indexer,
                    &self.indexes,
                    &self.price,
                    &self.blocks,
                    &self.supply,
                    &self.distribution,
                    exit,
                )
            })?;

            self.models.gate().begin_update();
            timed("Computed models", || {
                self.models.compute(
                    indexer,
                    &self.indexes,
                    &self.price,
                    &self.distribution,
                    &utxo_states,
                    &self.frameworks,
                    &self.market.moving_average,
                    exit,
                )
            })?;

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
        self.frameworks.gate().finish_update();
        self.models.gate().finish_update();
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
                use brk_traversable::Traversable;
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
    frameworks,
    models,
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

fn timed<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let result = f();
    info!("{label} in {:?}", start.elapsed());
    result
}
