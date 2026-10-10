use bitview_collections::Windows;
use std::{thread, time::Instant};

use bitview_distribution::metrics::ShareTotals;
use bitview_plugin::{ImportContext, Plugin};
use bitview_plugin_addresses::{ID as ADDRESSES_ID, Vecs as Addresses};
use bitview_plugin_age::{ID as AGE_ID, Vecs as Age};
use bitview_plugin_bedrock::{ID as BEDROCK_ID, Vecs as Bedrock};
use bitview_plugin_blocks::{ID as BLOCKS_ID, Vecs as Blocks};
use bitview_plugin_capital_sentiment::{ID as CAPITAL_SENTIMENT_ID, Vecs as CapitalSentiment};
use bitview_plugin_coinflow::{ID as COINFLOW_ID, Vecs as Coinflow};
use bitview_plugin_cointime::{ID as COINTIME_ID, Vecs as Cointime};
use bitview_plugin_entry::{ID as ENTRY_ID, Vecs as Entry};
use bitview_plugin_holders::{ID as HOLDERS_ID, Vecs as Holders};
use bitview_plugin_indexer::{ID as INDEXER_ID, Indexer};
use bitview_plugin_indicators::{ID as INDICATORS_ID, Vecs as Indicators};
use bitview_plugin_inputs::{ID as INPUTS_ID, Vecs as Inputs};
use bitview_plugin_mappings::{ID as MAPPINGS_ID, Vecs as Mappings};
use bitview_plugin_market::{ID as MARKET_ID, Vecs as Market};
use bitview_plugin_mining::{ID as MINING_ID, Vecs as Mining};
use bitview_plugin_op_return::{ID as OP_RETURN_ID, Vecs as OpReturn};
use bitview_plugin_outputs::{ID as OUTPUTS_ID, Vecs as Outputs};
use bitview_plugin_pools::{ID as POOLS_ID, Vecs as Pools};
use bitview_plugin_price::{ID as PRICE_ID, Vecs as Price};
use bitview_plugin_rarity_meter::{ID as RARITY_METER_ID, Vecs as RarityMeter};
use bitview_plugin_supply::{ID as SUPPLY_ID, Vecs as Supply};
use bitview_plugin_transactions::{ID as TRANSACTIONS_ID, Vecs as Transactions};
use bitview_plugin_utxo_set::{ID as UTXO_SET_ID, Vecs as UtxoSet};
use bitview_plugin_utxos::{ID as UTXOS_ID, Vecs as Utxos};
use brk_error::Result;
use brk_reader::Reader;
use tracing::info;
use vecdb::ReadableCloneableVec;

use crate::{
    DefaultPlugins,
    timing::{Phase, timed},
};

impl DefaultPlugins {
    pub fn import(context: ImportContext<'_>, reader: &Reader) -> Result<Self> {
        info!("Importing plugins...");
        let import_start = Instant::now();
        let indexer = timed(Phase::Import, INDEXER_ID, || {
            Indexer::import(context, reader)
        })?;

        const STACK_SIZE: usize = 8 * 1024 * 1024;
        let big_thread = || thread::Builder::new().stack_size(STACK_SIZE);

        let mappings = timed(Phase::Import, MAPPINGS_ID, || -> Result<_> {
            Ok(Box::new(Mappings::import(context, &indexer)?))
        })?;

        let price = timed(Phase::Import, PRICE_ID, || -> Result<_> {
            Ok(Box::new(Price::import(context, &mappings)?))
        })?;

        let blocks = timed(Phase::Import, BLOCKS_ID, || -> Result<_> {
            Ok(Box::new(Blocks::import(context, &indexer, &mappings)?))
        })?;

        let window_starts = blocks.lookback.window_starts();

        let (inputs, outputs, mining, transactions, pools, op_return) =
            thread::scope(|scope| -> Result<_> {
                let inputs_handle = big_thread().spawn_scoped(scope, || -> Result<_> {
                    timed(Phase::Import, INPUTS_ID, || {
                        Ok(Box::new(Inputs::import(
                            context,
                            &mappings,
                            &window_starts,
                        )?))
                    })
                })?;

                let outputs_handle = big_thread().spawn_scoped(scope, || -> Result<_> {
                    timed(Phase::Import, OUTPUTS_ID, || {
                        Ok(Box::new(Outputs::import(
                            context,
                            &mappings,
                            &window_starts,
                        )?))
                    })
                })?;

                let mining_handle = big_thread().spawn_scoped(scope, || -> Result<_> {
                    timed(Phase::Import, MINING_ID, || {
                        Ok(Box::new(Mining::import(
                            context,
                            &mappings,
                            &window_starts,
                        )?))
                    })
                })?;

                let transactions_handle = big_thread().spawn_scoped(scope, || -> Result<_> {
                    timed(Phase::Import, TRANSACTIONS_ID, || {
                        Ok(Box::new(Transactions::import(
                            context,
                            &indexer,
                            &mappings,
                            &window_starts,
                        )?))
                    })
                })?;

                let mining = mining_handle.join().unwrap()?;
                let block_size = blocks.size.size.cumulative_source();
                let chain_fees = mining
                    .rewards
                    .fees
                    .cumulative_sats_source()
                    .read_only_boxed_clone();
                let op_return_handle = {
                    let mappings = &mappings;
                    let window_starts = &window_starts;
                    big_thread().spawn_scoped(scope, move || -> Result<_> {
                        timed(Phase::Import, OP_RETURN_ID, || {
                            Ok(Box::new(OpReturn::import(
                                context,
                                mappings,
                                window_starts,
                                block_size,
                                &chain_fees,
                            )?))
                        })
                    })
                }?;
                // Pool hash rates read mining's network estimates; the import itself is light.
                let pools = timed(Phase::Import, POOLS_ID, || -> Result<_> {
                    let rate = &mining.hashrate.rate;
                    let network = Windows {
                        _24h: &rate.block.height,
                        _1w: &rate.sma._1w.height,
                        _1m: &rate.sma._1m.height,
                        _1y: &rate.sma._1y.height,
                    };
                    Ok(Box::new(Pools::import(
                        context,
                        &mappings,
                        &window_starts,
                        &network,
                    )?))
                })?;
                let inputs = inputs_handle.join().unwrap()?;
                let outputs = outputs_handle.join().unwrap()?;
                let transactions = transactions_handle.join().unwrap()?;
                let op_return = op_return_handle.join().unwrap()?;

                Ok((inputs, outputs, mining, transactions, pools, op_return))
            })?;

        let utxo_set = timed(Phase::Import, UTXO_SET_ID, || -> Result<_> {
            Ok(Box::new(UtxoSet::import(
                context,
                &mappings,
                inputs.storage().path(context),
                outputs.storage().path(context),
            )?))
        })?;

        // Market and distribution are independent; import in parallel.
        let (age, market) = thread::scope(|scope| -> Result<_> {
            let market_handle = big_thread().spawn_scoped(scope, || -> Result<_> {
                timed(Phase::Import, MARKET_ID, || {
                    Ok(Box::new(Market::import(
                        context, &mappings, &blocks, &price,
                    )?))
                })
            })?;

            let age = timed(Phase::Import, AGE_ID, || -> Result<_> {
                Ok(Box::new(Age::import(
                    context,
                    &mappings,
                    &window_starts,
                    &price,
                    &utxo_set.supply.read_only_boxed_clone(),
                )?))
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
        let utxos = timed(Phase::Import, UTXOS_ID, || -> Result<_> {
            Ok(Box::new(Utxos::import(
                context,
                &mappings,
                &window_starts,
                &price,
                totals,
            )?))
        })?;

        let addresses = timed(Phase::Import, ADDRESSES_ID, || -> Result<_> {
            Ok(Box::new(Addresses::import(
                context,
                &mappings,
                &window_starts,
                &price,
                totals,
            )?))
        })?;

        let holders = timed(Phase::Import, HOLDERS_ID, || -> Result<_> {
            Ok(Box::new(Holders::import(
                context,
                &mappings,
                &window_starts,
                &price,
                &utxo_set.supply.read_only_boxed_clone(),
            )?))
        })?;
        let all_chain = holders.all_chain_sources();

        let (cointime, coinflow, bedrock, capital_sentiment, indicators, entry) =
            thread::scope(|scope| -> Result<_> {
                let cointime = big_thread().spawn_scoped(scope, || -> Result<_> {
                    timed(Phase::Import, COINTIME_ID, || {
                        Ok(Box::new(Cointime::import(
                            context,
                            &mappings,
                            &window_starts,
                            &price,
                            &mining.rewards.subsidy.cumulative.cents,
                            &all_chain,
                            &age,
                        )?))
                    })
                })?;
                let coinflow = big_thread().spawn_scoped(scope, || -> Result<_> {
                    timed(Phase::Import, COINFLOW_ID, || {
                        Ok(Box::new(Coinflow::import(
                            context, &mappings, &price, &age,
                        )?))
                    })
                })?;
                let bedrock = big_thread().spawn_scoped(scope, || -> Result<_> {
                    timed(Phase::Import, BEDROCK_ID, || {
                        Ok(Box::new(Bedrock::import(context, &mappings)?))
                    })
                })?;
                let capital_sentiment = big_thread().spawn_scoped(scope, || -> Result<_> {
                    timed(Phase::Import, CAPITAL_SENTIMENT_ID, || {
                        Ok(Box::new(CapitalSentiment::import(context, &mappings)?))
                    })
                })?;
                let entry = big_thread().spawn_scoped(scope, || -> Result<_> {
                    timed(Phase::Import, ENTRY_ID, || {
                        Ok(Box::new(Entry::import(
                            context,
                            &mappings,
                            &window_starts,
                            &price.spot.cents.height.read_only_boxed_clone(),
                            totals,
                        )?))
                    })
                })?;
                let indicators = timed(Phase::Import, INDICATORS_ID, || -> Result<_> {
                    Ok(Box::new(Indicators::import(
                        context,
                        &mappings,
                        &all_chain,
                        &mining,
                        &holders,
                        &transactions,
                    )?))
                })?;
                Ok((
                    cointime.join().unwrap()?,
                    coinflow.join().unwrap()?,
                    bedrock.join().unwrap()?,
                    capital_sentiment.join().unwrap()?,
                    indicators,
                    entry.join().unwrap()?,
                ))
            })?;

        let (supply, rarity_meter) = thread::scope(|scope| -> Result<_> {
            let supply = big_thread().spawn_scoped(scope, || -> Result<_> {
                timed(Phase::Import, SUPPLY_ID, || {
                    Ok(Box::new(Supply::import(
                        context,
                        &mappings,
                        &window_starts,
                        &holders,
                        &all_chain,
                        &transactions,
                    )?))
                })
            })?;
            let rarity_meter = big_thread().spawn_scoped(scope, || -> Result<_> {
                timed(Phase::Import, RARITY_METER_ID, || {
                    Ok(Box::new(RarityMeter::import(
                        context, &mappings, &holders, &cointime, &coinflow,
                    )?))
                })
            })?;
            Ok((supply.join().unwrap()?, rarity_meter.join().unwrap()?))
        })?;

        info!("Imported all plugins in {:.2?}", import_start.elapsed());

        Ok(Self {
            indexer: Box::new(indexer),
            blocks,
            mining,
            transactions,
            indicators,
            market,
            age,
            holders,
            entry,
            utxos,
            addresses,
            supply,
            pools,
            cointime,
            coinflow,
            bedrock,
            capital_sentiment,
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
