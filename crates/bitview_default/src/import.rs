use std::{thread, time::Instant};

use bitview_distribution::metrics::ShareTotals;
use bitview_plugin::{ImportContext, Plugin};
use bitview_plugin_addresses::{ID as ADDRESSES_ID, Vecs as Addresses};
use bitview_plugin_age::{ID as AGE_ID, Vecs as Age};
use bitview_plugin_bedrock::{ID as BEDROCK_ID, Vecs as Bedrock};
use bitview_plugin_blocks::{ID as BLOCKS_ID, Vecs as Blocks};
use bitview_plugin_coinflow::{ID as COINFLOW_ID, Vecs as Coinflow};
use bitview_plugin_cointime::{ID as COINTIME_ID, Vecs as Cointime};
use bitview_plugin_entry::{ID as ENTRY_ID, Vecs as Entry};
use bitview_plugin_holders::{ID as HOLDERS_ID, Vecs as Holders};
use bitview_plugin_indexer::{ID as INDEXER_ID, Indexer};
use bitview_plugin_inputs::{ID as INPUTS_ID, Vecs as Inputs};
use bitview_plugin_mappings::{ID as MAPPINGS_ID, Vecs as Mappings};
use bitview_plugin_market::{ID as MARKET_ID, Vecs as Market};
use bitview_plugin_mining::{ID as MINING_ID, Vecs as Mining};
use bitview_plugin_op_return::{ID as OP_RETURN_ID, Vecs as OpReturn};
use bitview_plugin_outputs::{ID as OUTPUTS_ID, Vecs as Outputs};
use bitview_plugin_pools::{ID as POOLS_ID, Vecs as Pools};
use bitview_plugin_price::{ID as PRICE_ID, Vecs as Price};
use bitview_plugin_profitability::{ID as PROFITABILITY_ID, Vecs as Profitability};
use bitview_plugin_rarity_meter::{ID as RARITY_METER_ID, Vecs as RarityMeter};
use bitview_plugin_supply::{ID as SUPPLY_ID, Vecs as Supply};
use bitview_plugin_transactions::{ID as TRANSACTIONS_ID, Vecs as Transactions};
use bitview_plugin_utxo_set::{ID as UTXO_SET_ID, Vecs as UtxoSet};
use bitview_plugin_utxos::{ID as UTXOS_ID, Vecs as Utxos};
use bitview_vecs::shapes::Windows;
use brk_error::Result;
use brk_reader::Reader;
use tracing::info;
use vecdb::ReadableCloneableVec;

use crate::{DefaultPlugins, timing::timed_import};

impl DefaultPlugins {
    pub fn import(context: ImportContext<'_>, reader: &Reader) -> Result<Self> {
        let import_start = Instant::now();
        let indexer = timed_import(INDEXER_ID, || Indexer::import(context, reader))?;

        const STACK_SIZE: usize = 8 * 1024 * 1024;
        let large_stack_thread = || thread::Builder::new().stack_size(STACK_SIZE);

        let mappings = timed_import(MAPPINGS_ID, || Mappings::import(context, &indexer))?;

        let price = timed_import(PRICE_ID, || Price::import(context, &mappings))?;

        let blocks = timed_import(BLOCKS_ID, || Blocks::import(context, &indexer, &mappings))?;

        let window_starts = mappings.lookback.window_starts();

        let (inputs, outputs, mining, transactions, pools, op_return) =
            thread::scope(|scope| -> Result<_> {
                let inputs_handle = large_stack_thread().spawn_scoped(scope, || {
                    timed_import(INPUTS_ID, || {
                        Inputs::import(context, &mappings, &window_starts)
                    })
                })?;

                let outputs_handle = large_stack_thread().spawn_scoped(scope, || {
                    timed_import(OUTPUTS_ID, || {
                        Outputs::import(context, &mappings, &window_starts)
                    })
                })?;

                let mining_handle = large_stack_thread().spawn_scoped(scope, || {
                    timed_import(MINING_ID, || {
                        Mining::import(context, &mappings, &window_starts)
                    })
                })?;

                let transactions_handle = large_stack_thread().spawn_scoped(scope, || {
                    timed_import(TRANSACTIONS_ID, || {
                        Transactions::import(context, &indexer, &mappings, &window_starts)
                    })
                })?;

                let transactions = transactions_handle.join().unwrap()?;
                let block_size = blocks.size.cumulative_source();
                let chain_fees = transactions.fees.total_cumulative.read_only_boxed_clone();
                let op_return_handle = {
                    let indexer = &indexer;
                    let mappings = &mappings;
                    let window_starts = &window_starts;
                    large_stack_thread().spawn_scoped(scope, move || {
                        timed_import(OP_RETURN_ID, || {
                            OpReturn::import(
                                context,
                                indexer,
                                mappings,
                                window_starts,
                                block_size,
                                &chain_fees,
                            )
                        })
                    })
                }?;
                let mining = mining_handle.join().unwrap()?;
                // Pool hash rates read mining's network estimates; the import itself is light.
                let pools = timed_import(POOLS_ID, || {
                    let rate = &mining.hash.hashrate;
                    let network = Windows {
                        _24h: &rate.value.height,
                        _1w: &rate.sma._1w.height,
                        _1m: &rate.sma._1m.height,
                        _1y: &rate.sma._1y.height,
                    };
                    Pools::import(context, &mappings, &window_starts, &network)
                })?;
                let inputs = inputs_handle.join().unwrap()?;
                let outputs = outputs_handle.join().unwrap()?;
                let op_return = op_return_handle.join().unwrap()?;

                Ok((inputs, outputs, mining, transactions, pools, op_return))
            })?;

        let utxo_set = timed_import(UTXO_SET_ID, || {
            UtxoSet::import(
                context,
                &mappings,
                inputs.storage().path(context),
                outputs.storage().path(context),
            )
        })?;

        // Market and Age are independent; import in parallel.
        let (age, market) = thread::scope(|scope| -> Result<_> {
            let market_handle = large_stack_thread().spawn_scoped(scope, || {
                timed_import(MARKET_ID, || Market::import(context, &mappings, &price))
            })?;

            let age = timed_import(AGE_ID, || {
                Age::import(
                    context,
                    &mappings,
                    &window_starts,
                    &price,
                    &utxo_set.supply.read_only_boxed_clone(),
                )
            })?;

            let market = market_handle.join().unwrap()?;
            Ok((age, market))
        })?;

        let all_supply = utxo_set.supply.read_only_boxed_clone();
        let all_capital = age.all_capital().read_only_boxed_clone();
        let totals = ShareTotals {
            supply: &all_supply,
            capital: &all_capital,
        };
        let utxos = timed_import(UTXOS_ID, || {
            Utxos::import(context, &mappings, &window_starts, &price, totals)
        })?;

        let addresses = timed_import(ADDRESSES_ID, || {
            Addresses::import(context, &mappings, &window_starts, &price, totals)
        })?;

        let holders = timed_import(HOLDERS_ID, || {
            Holders::import(
                context,
                &mappings,
                &window_starts,
                &price,
                &utxo_set.supply.read_only_boxed_clone(),
                &age,
            )
        })?;
        let all_chain = holders.all_chain_sources();

        let (cointime, coinflow, bedrock, entry, profitability) =
            thread::scope(|scope| -> Result<_> {
                let cointime = large_stack_thread().spawn_scoped(scope, || {
                    timed_import(COINTIME_ID, || {
                        Cointime::import(
                            context,
                            &mappings,
                            &window_starts,
                            &price,
                            &mining.rewards.subsidy.cumulative.cents,
                            &all_chain,
                            &age,
                        )
                    })
                })?;
                let coinflow = large_stack_thread().spawn_scoped(scope, || {
                    timed_import(COINFLOW_ID, || {
                        Coinflow::import(context, &mappings, &price, &age)
                    })
                })?;
                let bedrock = large_stack_thread().spawn_scoped(scope, || {
                    timed_import(BEDROCK_ID, || {
                        Bedrock::import(
                            context,
                            &mappings,
                            &price.spot.cents.height.read_only_boxed_clone(),
                        )
                    })
                })?;
                let entry = large_stack_thread().spawn_scoped(scope, || {
                    timed_import(ENTRY_ID, || {
                        Entry::import(
                            context,
                            &mappings,
                            &window_starts,
                            &price.spot.cents.height.read_only_boxed_clone(),
                            totals,
                        )
                    })
                })?;
                let profitability = large_stack_thread().spawn_scoped(scope, || {
                    timed_import(PROFITABILITY_ID, || {
                        Profitability::import(
                            context,
                            &mappings,
                            &price.spot.cents.height.read_only_boxed_clone(),
                        )
                    })
                })?;
                Ok((
                    cointime.join().unwrap()?,
                    coinflow.join().unwrap()?,
                    bedrock.join().unwrap()?,
                    entry.join().unwrap()?,
                    profitability.join().unwrap()?,
                ))
            })?;

        let (supply, rarity_meter) = thread::scope(|scope| -> Result<_> {
            let supply = large_stack_thread().spawn_scoped(scope, || {
                timed_import(SUPPLY_ID, || {
                    Supply::import(
                        context,
                        &mappings,
                        &window_starts,
                        &holders,
                        &all_chain,
                        &transactions,
                    )
                })
            })?;
            let rarity_meter = large_stack_thread().spawn_scoped(scope, || {
                timed_import(RARITY_METER_ID, || {
                    RarityMeter::import(context, &mappings, &holders, &cointime, &coinflow)
                })
            })?;
            Ok((supply.join().unwrap()?, rarity_meter.join().unwrap()?))
        })?;

        info!("Imported all plugins in {:.2?}", import_start.elapsed());

        Ok(Self {
            indexer,
            blocks,
            mining,
            transactions,
            market,
            age,
            holders,
            entry,
            profitability,
            utxos,
            addresses,
            supply,
            pools,
            cointime,
            coinflow,
            bedrock,
            rarity_meter,
            mappings,
            inputs,
            price,
            outputs,
            utxo_set,
            op_return,
        })
    }
}
