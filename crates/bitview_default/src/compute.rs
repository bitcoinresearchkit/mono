use std::{sync::Arc, thread};

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
use bitview_plugin_silent_payments::Dependencies as SilentPaymentsDependencies;
use bitview_plugin_supply::Dependencies as SupplyDependencies;
use bitview_plugin_transactions::Dependencies as TransactionsDependencies;
use bitview_plugin_utxo_set::Dependencies as UtxoSetDependencies;
use bitview_plugin_utxos::Dependencies as UtxosDependencies;
use bitview_urpd::ReplayInputs;
use brk_error::Result;
use brk_types::Transaction;
use rayon::join;
use tracing::info;
use vecdb::ReadableCloneableVec;

use crate::{DefaultPlugins, timing::timed_compute};

const REIMPORT_THRESHOLD: u32 = 10_000;

impl DefaultPlugins {
    fn compute_all(&mut self, context: UpdateContext<'_>) -> Result<()> {
        timed_compute(&mut *self.indexer, (), context)?;
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
        let lock = context.exit().lock();
        self.addresses
            .roll_back_to(usize::from(starting.height), &self.indexer, &lock)?;
        self.indexer.roll_back_to(&starting, &lock)
    }

    /// Starts each plugin once every plugin its `Dependencies` borrow has computed, keeping each
    /// plugin's internal order. Branches beside the main chain (Outputs → UTXO set → Age → Holders
    /// → Cointime) are scoped threads; forks that rejoin before going on are `rayon::join`s. A
    /// branch spawns its own dependents and hands over its plugin as soon as it completes; every
    /// branch is joined before the composition commits.
    fn compute_dependents(&mut self, context: UpdateContext<'_>) -> Result<()> {
        let Self {
            indexer,
            mappings,
            price,
            blocks,
            inputs,
            outputs,
            transactions,
            mining,
            op_return,
            silent_payments,
            pools,
            utxo_set,
            market,
            age,
            utxos,
            addresses,
            holders,
            entry,
            profitability,
            supply,
            cointime,
            coinflow,
            bedrock,
            rarity_meter,
        } = self;
        let context = context.dependents(indexer.compute_start());
        let indexer = indexer.as_ref();

        // Price reads only the indexer.
        let (mappings_result, price_result) = join(
            || {
                timed_compute(
                    &mut **mappings,
                    MappingsDependencies { indexer },
                    context,
                )
            },
            || timed_compute(&mut **price, PriceDependencies { indexer }, context),
        );
        mappings_result?;
        price_result?;
        let mappings = mappings.as_ref();
        let price = price.as_ref();
        let spot_cents = &price.spot.cents.height;

        thread::scope(|scope| -> Result<()> {
            // Only the UTXO set reads inputs' spends.
            let inputs = scope.spawn(|| -> Result<_> {
                timed_compute(
                    &mut **inputs,
                    InputsDependencies { mappings, indexer },
                    context,
                )?;
                Ok(inputs.as_ref())
            });
            let market = scope.spawn(|| {
                timed_compute(
                    &mut **market,
                    MarketDependencies {
                        price,
                        mappings,
                    },
                    context,
                )
            });
            let utxos = scope.spawn(|| {
                timed_compute(
                    &mut **utxos,
                    UtxosDependencies {
                        indexer,
                        mappings,
                        price,
                    },
                    context,
                )
            });
            let addresses = scope.spawn(|| {
                timed_compute(
                    &mut **addresses,
                    AddressesDependencies {
                        indexer,
                        mappings,
                        price,
                    },
                    context,
                )
            });
            // Silent payments read only the indexer and Core's block files.
            let silent_payments = scope.spawn(|| {
                timed_compute(
                    &mut **silent_payments,
                    SilentPaymentsDependencies { indexer },
                    context,
                )
            });

            // Blocks beside Transactions (only Mining reads Blocks), then OP_RETURN beside Mining
            // and Pools after it: the branch hands Mining over once it completes.
            let (op_return, pools) = (&mut **op_return, &mut **pools);
            let mining = scope.spawn(|| -> Result<_> {
                let (blocks_result, transactions_result) = join(
                    || {
                        timed_compute(
                            &mut **blocks,
                            BlocksDependencies { indexer, mappings },
                            context,
                        )
                    },
                    || {
                        timed_compute(
                            &mut **transactions,
                            TransactionsDependencies {
                                indexer,
                                mappings,
                                spot_cents,
                            },
                            context,
                        )
                    },
                );
                blocks_result?;
                transactions_result?;
                let transactions = transactions.as_ref();
                let op_return = scope.spawn(move || {
                    timed_compute(
                        op_return,
                        OpReturnDependencies {
                            indexer,
                            fee: &transactions.fees.fee.tx,
                            spot_cents,
                        },
                        context,
                    )
                });
                timed_compute(
                    &mut **mining,
                    MiningDependencies {
                        mappings,
                        difficulty_hashrate: &blocks.difficulty.hashrate,
                        coinbase_value: &transactions.fees.coinbase_value,
                        total_fees: &transactions.fees.total,
                        spot_cents,
                    },
                    context,
                )?;
                let mining = mining.as_ref();
                let pools = scope.spawn(move || {
                    timed_compute(
                        pools,
                        PoolsDependencies {
                            indexer,
                            coinbase_sats: &mining.rewards.coinbase.block.sats,
                            coinbase_cents: &mining.rewards.coinbase.block.cents,
                            fee_sats: &mining.rewards.fees.block.sats,
                            fee_cents: &mining.rewards.fees.block.cents,
                        },
                        context,
                    )
                });
                Ok((mining, transactions, [op_return, pools]))
            });

            timed_compute(
                &mut **outputs,
                OutputsDependencies {
                    indexer,
                    mappings,
                    price,
                },
                context,
            )?;
            let outputs = outputs.as_ref();

            // Supply reads Outputs, Mining and Price: it runs beside the history chain.
            let supply = &mut **supply;
            let supply = scope.spawn(move || -> Result<_> {
                let (mining, transactions, mining_tail) = mining.join().unwrap()?;
                timed_compute(
                    &mut *supply,
                    SupplyDependencies {
                        outputs,
                        mining,
                        price,
                    },
                    context,
                )?;
                Ok((&*supply, transactions, mining_tail))
            });

            let inputs = inputs.join().unwrap()?;
            let creations = &outputs.creations;
            timed_compute(
                &mut **utxo_set,
                UtxoSetDependencies {
                    spends: &inputs.spends,
                    creations,
                    end: creations.end(),
                },
                context,
            )?;

            let history = &utxo_set.reader(&inputs.spends, creations)?;
            let urpd = ReplayInputs {
                history,
                prices: spot_cents,
                timestamps: &mappings.timestamp.monotonic,
            };
            let prices = &spot_cents.read_only_boxed_clone();
            let timestamps = &mappings.timestamp.monotonic.read_only_boxed_clone();
            thread::scope(|scope| -> Result<()> {
                // Profitability reads only the history, prices and timestamps.
                let profitability = scope.spawn(|| {
                    timed_compute(
                        &mut **profitability,
                        ProfitabilityDependencies {
                            history,
                            prices,
                            timestamps,
                        },
                        context,
                    )
                });
                timed_compute(
                    &mut **age,
                    AgeDependencies {
                        history,
                        mappings,
                        price,
                    },
                    context,
                )?;
                let age = age.as_ref();

                // Coinflow reads Age, not Holders.
                let coinflow = scope.spawn(|| -> Result<_> {
                    timed_compute(
                        &mut **coinflow,
                        CoinflowDependencies {
                            urpd,
                            mappings,
                            price,
                            age,
                        },
                        context,
                    )?;
                    Ok(coinflow.as_ref())
                });
                timed_compute(
                    &mut **holders,
                    HoldersDependencies {
                        history,
                        age,
                        mappings,
                        price,
                    },
                    context,
                )?;
                let holders = holders.as_ref();

                let capitalized_price = holders.all_capitalized_price();
                let entry = &mut **entry;
                let entry = scope.spawn(move || {
                    timed_compute(
                        entry,
                        EntryDependencies {
                            history,
                            prices,
                            timestamps,
                            capitalized_price: &capitalized_price,
                        },
                        context,
                    )
                });

                let (supply, transactions, mining_tail) = supply.join().unwrap()?;
                timed_compute(
                    &mut **cointime,
                    CointimeDependencies {
                        urpd,
                        mappings,
                        price,
                        inflation_rate: &supply.inflation_rate,
                        volume_1y: &transactions.transfer_volume.sum._1y,
                        age,
                        holders,
                    },
                    context,
                )?;
                let cointime = cointime.as_ref();
                let coinflow = coinflow.join().unwrap()?;

                let (bedrock, rarity_meter) = join(
                    || {
                        timed_compute(
                            &mut **bedrock,
                            BedrockDependencies {
                                urpd,
                                age,
                                holders,
                                cointime,
                                coinflow,
                            },
                            context,
                        )
                    },
                    || {
                        timed_compute(
                            &mut **rarity_meter,
                            RarityMeterDependencies {
                                holders,
                                cointime,
                                coinflow,
                                spot_cents,
                            },
                            context,
                        )
                    },
                );
                bedrock?;
                rarity_meter?;
                for branch in mining_tail.into_iter().chain([entry, profitability]) {
                    branch.join().unwrap()?;
                }
                Ok(())
            })?;
            for branch in [market, utxos, addresses, silent_payments] {
                branch.join().unwrap()?;
            }
            Ok(())
        })
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
            timed_compute(&mut *self.indexer, (), context)?;
            return Ok(BootstrapAction::Reimport);
        }

        self.compute_all(context)?;

        Ok(BootstrapAction::Ready)
    }

    fn compute(&mut self, context: UpdateContext<'_>) -> Result<()> {
        self.roll_back_reorg(context)?;
        self.compute_all(context)
    }

    fn commit(&mut self, complete: bool) -> Result<()> {
        let prepared = self.indexer.prepare_commit()?;
        // Address state changes in place: it is published with the lengths, under the lock its
        // readers take before they pin, so they read the state of the publication they pin.
        self.addresses
            .publish_state_with(|| self.indexer.publish(prepared, complete))
    }

    fn write_published(&mut self) -> Result<()> {
        self.addresses.write_published_state()
    }

    // The reads an update spends most of its time on when idle minutes left them on disk: the
    // indexer's lookups and the addresses' data. Not the spend index: its writes read their pages
    // from disk either way (rawdb's indexed writes on macOS).
    fn warm(&self, txs: &[Arc<Transaction>]) {
        self.addresses.warm(&self.indexer.warm(txs));
    }
}
