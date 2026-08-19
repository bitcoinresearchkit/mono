use std::{fs, io::Error, path::Path, thread, time::Instant};

use bitview_plugin::PLUGIN_DATA_DIR;
use bitview_plugin_bedrock::Vecs as Bedrock;
use bitview_plugin_blocks::Vecs as Blocks;
use bitview_plugin_capital_sentiment::Vecs as CapitalSentiment;
use bitview_plugin_coinflow::Vecs as Coinflow;
use bitview_plugin_cointime::Vecs as Cointime;
use bitview_plugin_constants::Vecs as Constants;
use bitview_plugin_distribution::Vecs as Distribution;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_indexes::Vecs as Indexes;
use bitview_plugin_indicators::Vecs as Indicators;
use bitview_plugin_inputs::Vecs as Inputs;
use bitview_plugin_investing::Vecs as Investing;
use bitview_plugin_market::Vecs as Market;
use bitview_plugin_mining::Vecs as Mining;
use bitview_plugin_op_return::Vecs as OpReturn;
use bitview_plugin_outputs::Vecs as Outputs;
use bitview_plugin_pools::Vecs as Pools;
use bitview_plugin_price::Vecs as Price;
use bitview_plugin_rarity_meter::Vecs as RarityMeter;
use bitview_plugin_supply::Vecs as Supply;
use bitview_plugin_transactions::Vecs as Transactions;
use brk_error::Result;
use brk_types::Version;
use tracing::info;

use crate::{DefaultPlugins, timed};

const VERSION: Version = Version::new(9);

// Startup-only compatibility shim. It runs before any compute plugin opens its data.
// TODO: Remove once automatic migration from the pre-plugin layout is no longer supported.
fn migrate_legacy_computed_dir(outputs_path: &Path, plugins_path: &Path) -> Result<()> {
    let legacy = outputs_path.join("computed");
    fs::create_dir_all(plugins_path)?;
    if !legacy.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(&legacy)? {
        let entry = entry?;
        let destination = plugins_path.join(entry.file_name());
        if destination.exists() {
            return Err(Error::other(format!(
                "Both legacy and plugin data exist for {:?}",
                entry.file_name()
            ))
            .into());
        }
        fs::rename(entry.path(), destination)?;
    }
    fs::remove_dir(&legacy)?;
    info!("Moved legacy computed data into plugin directories");
    Ok(())
}

impl DefaultPlugins {
    pub fn forced_import(outputs_path: &Path, indexer: Indexer) -> Result<Self> {
        info!("Importing plugins...");
        let import_start = Instant::now();
        let plugins_path = outputs_path.join(PLUGIN_DATA_DIR);
        migrate_legacy_computed_dir(outputs_path, &plugins_path)?;

        const STACK_SIZE: usize = 8 * 1024 * 1024;
        let big_thread = || thread::Builder::new().stack_size(STACK_SIZE);

        let indexes = timed("Imported indexes", || -> Result<_> {
            Ok(Box::new(Indexes::forced_import(
                &plugins_path,
                VERSION,
                &indexer,
            )?))
        })?;

        let (constants, price) = timed("Imported price/constants", || -> Result<_> {
            let constants = Box::new(Constants::new(VERSION, &indexes));
            let price = Box::new(Price::forced_import(&plugins_path, VERSION, &indexes)?);
            Ok((constants, price))
        })?;

        let blocks = timed("Imported blocks", || -> Result<_> {
            Ok(Box::new(Blocks::forced_import(
                &plugins_path,
                VERSION,
                &indexer,
                &indexes,
            )?))
        })?;

        let cached_starts = blocks.lookback.cached_window_starts();

        let (inputs, outputs, mining, transactions, pools, op_return) =
            timed("Imported inputs/outputs/mining/tx/pools/op_return", || {
                thread::scope(|scope| -> Result<_> {
                    let inputs_handle = big_thread().spawn_scoped(scope, || -> Result<_> {
                        Ok(Box::new(Inputs::forced_import(
                            &plugins_path,
                            VERSION,
                            &indexes,
                            &cached_starts,
                        )?))
                    })?;

                    let outputs_handle = big_thread().spawn_scoped(scope, || -> Result<_> {
                        Ok(Box::new(Outputs::forced_import(
                            &plugins_path,
                            VERSION,
                            &indexes,
                            &cached_starts,
                        )?))
                    })?;

                    let mining_handle = big_thread().spawn_scoped(scope, || -> Result<_> {
                        Ok(Box::new(Mining::forced_import(
                            &plugins_path,
                            VERSION,
                            &indexer,
                            &indexes,
                            &cached_starts,
                        )?))
                    })?;

                    let transactions_handle =
                        big_thread().spawn_scoped(scope, || -> Result<_> {
                            Ok(Box::new(Transactions::forced_import(
                                &plugins_path,
                                VERSION,
                                &indexer,
                                &indexes,
                                &cached_starts,
                            )?))
                        })?;

                    let pools_handle = big_thread().spawn_scoped(scope, || -> Result<_> {
                        Ok(Box::new(Pools::forced_import(
                            &plugins_path,
                            VERSION,
                            &indexes,
                            &cached_starts,
                        )?))
                    })?;

                    let mining = mining_handle.join().unwrap()?;
                    let block_size = blocks.size.size.cached_cumulative();
                    let chain_fees = mining.rewards.fees.cached_cumulative_sats();
                    let op_return_handle = big_thread().spawn_scoped(scope, || -> Result<_> {
                        Ok(Box::new(OpReturn::forced_import(
                            &plugins_path,
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
                thread::scope(|scope| -> Result<_> {
                    let market_handle = big_thread().spawn_scoped(scope, || -> Result<_> {
                        Ok(Box::new(Market::forced_import(
                            &plugins_path,
                            VERSION,
                            &indexes,
                            &blocks,
                            &price,
                        )?))
                    })?;

                    let investing_handle = big_thread().spawn_scoped(scope, || -> Result<_> {
                        Ok(Box::new(Investing::forced_import(
                            VERSION, &indexes, &blocks, &price,
                        )?))
                    })?;

                    let distribution = Box::new(Distribution::forced_import(
                        &plugins_path,
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
                thread::scope(|scope| -> Result<_> {
                    let cointime = big_thread().spawn_scoped(scope, || -> Result<_> {
                        Ok(Box::new(Cointime::forced_import(
                            &plugins_path,
                            VERSION,
                            &indexes,
                            &cached_starts,
                            &price,
                            &mining.rewards.subsidy.cumulative.cents,
                            &all_chain,
                        )?))
                    })?;
                    let coinflow = big_thread().spawn_scoped(scope, || -> Result<_> {
                        Ok(Box::new(Coinflow::forced_import(
                            &plugins_path,
                            VERSION,
                            &indexes,
                            &price,
                        )?))
                    })?;
                    let bedrock = big_thread().spawn_scoped(scope, || -> Result<_> {
                        Ok(Box::new(Bedrock::forced_import(
                            &plugins_path,
                            VERSION,
                            &indexes,
                        )?))
                    })?;
                    let capital_sentiment = big_thread().spawn_scoped(scope, || -> Result<_> {
                        Ok(Box::new(CapitalSentiment::forced_import(
                            &plugins_path,
                            VERSION,
                            &indexes,
                        )?))
                    })?;
                    let indicators = Box::new(Indicators::forced_import(
                        &plugins_path,
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
            thread::scope(|scope| -> Result<_> {
                let supply = big_thread().spawn_scoped(scope, || -> Result<_> {
                    Ok(Box::new(Supply::forced_import(
                        &plugins_path,
                        VERSION,
                        &indexes,
                        &cached_starts,
                        &distribution,
                        &cointime,
                        &all_chain,
                        &transactions,
                    )?))
                })?;
                let rarity_meter = big_thread().spawn_scoped(scope, || -> Result<_> {
                    Ok(Box::new(RarityMeter::forced_import(
                        &plugins_path,
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
            indexes,
            inputs,
            price,
            outputs,
            op_return,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_computed_data_moves_to_plugin_roots() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let legacy = dir.path().join("computed");
        let plugins = dir.path().join(PLUGIN_DATA_DIR);
        let blocks = legacy.join("blocks");
        fs::create_dir_all(&blocks)?;
        fs::write(blocks.join("marker"), b"data")?;

        migrate_legacy_computed_dir(dir.path(), &plugins)?;

        assert!(!legacy.exists());
        assert_eq!(fs::read(plugins.join("blocks/marker"))?, b"data");
        Ok(())
    }

    #[test]
    fn legacy_migration_never_overwrites_plugin_data() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let legacy = dir.path().join("computed/blocks");
        let plugins = dir.path().join(PLUGIN_DATA_DIR);
        let current = plugins.join("blocks");
        fs::create_dir_all(&legacy)?;
        fs::create_dir_all(&current)?;
        fs::write(legacy.join("marker"), b"legacy")?;
        fs::write(current.join("marker"), b"current")?;

        assert!(migrate_legacy_computed_dir(dir.path(), &plugins).is_err());
        assert_eq!(fs::read(legacy.join("marker"))?, b"legacy");
        assert_eq!(fs::read(current.join("marker"))?, b"current");
        Ok(())
    }
}
