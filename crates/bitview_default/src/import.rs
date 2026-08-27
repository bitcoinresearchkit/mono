use std::{thread, time::Instant};

use bitview_plugin::ImportContext;
use bitview_plugin_bedrock::{ID as BEDROCK_ID, Vecs as Bedrock};
use bitview_plugin_blocks::{ID as BLOCKS_ID, Vecs as Blocks};
use bitview_plugin_capital_sentiment::{ID as CAPITAL_SENTIMENT_ID, Vecs as CapitalSentiment};
use bitview_plugin_coinflow::{ID as COINFLOW_ID, Vecs as Coinflow};
use bitview_plugin_cointime::{ID as COINTIME_ID, Vecs as Cointime};
use bitview_plugin_constants::{ID as CONSTANTS_ID, Vecs as Constants};
use bitview_plugin_distribution::{ID as DISTRIBUTION_ID, Vecs as Distribution};
use bitview_plugin_indexer::{ID as INDEXER_ID, Indexer};
use bitview_plugin_indicators::{ID as INDICATORS_ID, Vecs as Indicators};
use bitview_plugin_inputs::{ID as INPUTS_ID, Vecs as Inputs};
use bitview_plugin_investing::{ID as INVESTING_ID, Vecs as Investing};
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
use brk_error::Result;
use brk_reader::Reader;
use tracing::info;

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

        let constants = timed(Phase::Import, CONSTANTS_ID, || {
            Box::new(Constants::new(&mappings))
        });
        let price = timed(Phase::Import, PRICE_ID, || -> Result<_> {
            Ok(Box::new(Price::import(context, &mappings)?))
        })?;

        let blocks = timed(Phase::Import, BLOCKS_ID, || -> Result<_> {
            Ok(Box::new(Blocks::import(context, &indexer, &mappings)?))
        })?;

        let cached_starts = blocks.lookback.cached_window_starts();

        let (inputs, outputs, mining, transactions, pools, op_return) =
            thread::scope(|scope| -> Result<_> {
                let inputs_handle = big_thread().spawn_scoped(scope, || -> Result<_> {
                    timed(Phase::Import, INPUTS_ID, || {
                        Ok(Box::new(Inputs::import(
                            context,
                            &mappings,
                            &cached_starts,
                        )?))
                    })
                })?;

                let outputs_handle = big_thread().spawn_scoped(scope, || -> Result<_> {
                    timed(Phase::Import, OUTPUTS_ID, || {
                        Ok(Box::new(Outputs::import(
                            context,
                            &mappings,
                            &cached_starts,
                        )?))
                    })
                })?;

                let mining_handle = big_thread().spawn_scoped(scope, || -> Result<_> {
                    timed(Phase::Import, MINING_ID, || {
                        Ok(Box::new(Mining::import(
                            context,
                            &indexer,
                            &mappings,
                            &cached_starts,
                        )?))
                    })
                })?;

                let transactions_handle = big_thread().spawn_scoped(scope, || -> Result<_> {
                    timed(Phase::Import, TRANSACTIONS_ID, || {
                        Ok(Box::new(Transactions::import(
                            context,
                            &indexer,
                            &mappings,
                            &cached_starts,
                        )?))
                    })
                })?;

                let pools_handle = big_thread().spawn_scoped(scope, || -> Result<_> {
                    timed(Phase::Import, POOLS_ID, || {
                        Ok(Box::new(Pools::import(context, &mappings, &cached_starts)?))
                    })
                })?;

                let mining = mining_handle.join().unwrap()?;
                let block_size = blocks.size.size.cached_cumulative();
                let chain_fees = mining.rewards.fees.cached_cumulative_sats();
                let op_return_handle = big_thread().spawn_scoped(scope, || -> Result<_> {
                    timed(Phase::Import, OP_RETURN_ID, || {
                        Ok(Box::new(OpReturn::import(
                            context,
                            &mappings,
                            &cached_starts,
                            block_size,
                            chain_fees,
                        )?))
                    })
                })?;
                let inputs = inputs_handle.join().unwrap()?;
                let outputs = outputs_handle.join().unwrap()?;
                let transactions = transactions_handle.join().unwrap()?;
                let pools = pools_handle.join().unwrap()?;
                let op_return = op_return_handle.join().unwrap()?;

                Ok((inputs, outputs, mining, transactions, pools, op_return))
            })?;

        // Market, investing, and distribution are independent; import in parallel.
        let (distribution, market, investing) = thread::scope(|scope| -> Result<_> {
            let market_handle = big_thread().spawn_scoped(scope, || -> Result<_> {
                timed(Phase::Import, MARKET_ID, || {
                    Ok(Box::new(Market::import(
                        context, &mappings, &blocks, &price,
                    )?))
                })
            })?;

            let investing_handle = big_thread().spawn_scoped(scope, || -> Result<_> {
                timed(Phase::Import, INVESTING_ID, || {
                    Ok(Box::new(Investing::import(&mappings, &blocks, &price)?))
                })
            })?;

            let distribution = timed(Phase::Import, DISTRIBUTION_ID, || -> Result<_> {
                Ok(Box::new(Distribution::import(
                    context,
                    &mappings,
                    &cached_starts,
                    &price,
                    &inputs.by_type,
                    &outputs.by_type,
                )?))
            })?;

            let market = market_handle.join().unwrap()?;
            let investing = investing_handle.join().unwrap()?;
            Ok((distribution, market, investing))
        })?;

        let all_chain = distribution.all_chain_sources();

        let (cointime, coinflow, bedrock, capital_sentiment, indicators) =
            thread::scope(|scope| -> Result<_> {
                let cointime = big_thread().spawn_scoped(scope, || -> Result<_> {
                    timed(Phase::Import, COINTIME_ID, || {
                        Ok(Box::new(Cointime::import(
                            context,
                            &mappings,
                            &cached_starts,
                            &price,
                            &mining.rewards.subsidy.cumulative.cents,
                            &all_chain,
                        )?))
                    })
                })?;
                let coinflow = big_thread().spawn_scoped(scope, || -> Result<_> {
                    timed(Phase::Import, COINFLOW_ID, || {
                        Ok(Box::new(Coinflow::import(context, &mappings, &price)?))
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
                let indicators = timed(Phase::Import, INDICATORS_ID, || -> Result<_> {
                    Ok(Box::new(Indicators::import(
                        context,
                        &mappings,
                        &all_chain,
                        &mining,
                        &distribution,
                        &transactions,
                    )?))
                })?;
                Ok((
                    cointime.join().unwrap()?,
                    coinflow.join().unwrap()?,
                    bedrock.join().unwrap()?,
                    capital_sentiment.join().unwrap()?,
                    indicators,
                ))
            })?;

        let (supply, rarity_meter) = thread::scope(|scope| -> Result<_> {
            let supply = big_thread().spawn_scoped(scope, || -> Result<_> {
                timed(Phase::Import, SUPPLY_ID, || {
                    Ok(Box::new(Supply::import(
                        context,
                        &mappings,
                        &cached_starts,
                        &distribution,
                        &cointime,
                        &all_chain,
                        &transactions,
                    )?))
                })
            })?;
            let rarity_meter = big_thread().spawn_scoped(scope, || -> Result<_> {
                timed(Phase::Import, RARITY_METER_ID, || {
                    Ok(Box::new(RarityMeter::import(
                        context,
                        &mappings,
                        &distribution,
                        &cointime,
                        &coinflow,
                    )?))
                })
            })?;
            Ok((supply.join().unwrap()?, rarity_meter.join().unwrap()?))
        })?;

        info!("Total import time: {:?}", import_start.elapsed());

        Ok(Self {
            indexer: Box::new(indexer),
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
            mappings,
            inputs,
            price,
            outputs,
            op_return,
        })
    }
}
