use brk_error::{Error, OptionData, Result};

use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_indexer::Indexer;
use brk_exit::Exit;
use brk_oracle::{
    Config, Oracle, START_HEIGHT_FAST, START_HEIGHT_SLOW, bin_to_cents, cents_to_bin,
    pre_oracle_prices_from,
};
use brk_types::Cents;
use tracing::info;
use vecdb::{AnyStoredVec, AnyVec, Database, ReadableVec, VecIndex, WritableVec};

use super::Vecs;
use crate::{Dependencies, feed_blocks_for_warmup, feed_blocks_with};

impl Vecs {
    fn compute_prices(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        let source_version = [
            indexer.vecs().transactions.txid.version(),
            indexer.vecs().transactions.first_tx_index.version(),
            indexer.vecs().outputs.first_txout_index.version(),
            indexer.vecs().transactions.first_txout_index.version(),
            indexer.vecs().outputs.value.version(),
            indexer.vecs().outputs.output_type.version(),
        ]
        .into_iter()
        .sum();
        let total_heights = indexer.vecs().blocks.timestamp.len();
        {
            let _lock = exit.lock();
            self.spot
                .cents
                .height
                .validate_computed_version_or_reset(source_version)?;

            // Reorg: truncate to starting_lengths
            self.spot
                .cents
                .height
                .truncate_if_needed_at(starting_height.to_usize())?;
        }

        let seed_len = total_heights.min(START_HEIGHT_SLOW);
        let seed_start = self.spot.cents.height.len();
        for cents in pre_oracle_prices_from(seed_start).take(seed_len.saturating_sub(seed_start)) {
            self.spot.cents.height.push(cents);
        }

        if self.spot.cents.height.len() >= total_heights {
            return Ok(());
        }

        let committed = self.spot.cents.height.len();
        let config = Config::for_height(committed);
        let prev_cents = self
            .spot
            .cents
            .height
            .collect_one_at(committed - 1)
            .data()?;
        let seed_bin = cents_to_bin(
            prev_cents
                .finite_inner()
                .ok_or(Error::Internal("Invalid oracle seed price"))? as f64,
        );
        let warmup = config.window_size.min(committed - START_HEIGHT_SLOW);
        let mut warmed = Ok(());
        let mut oracle = Oracle::from_checkpoint(seed_bin, config, |o| {
            warmed = feed_blocks_for_warmup(o, indexer, (committed - warmup)..committed, None);
        });
        warmed?;

        let num_new = total_heights - committed;
        info!("Computing {num_new} oracle prices ({warmup} warmup blocks)...");

        // Slow cold-start EMA up to START_HEIGHT_FAST, then switch to the fast
        // mature-market EMA. Steady-state runs start past START_HEIGHT_FAST and skip
        // the slow segment entirely.
        {
            let mut processed = 0usize;
            let mut next_progress = 10u8;
            let mut push_ref_bin = |ref_bin| {
                self.spot
                    .cents
                    .height
                    .push(Cents::new(bin_to_cents(ref_bin)));

                processed += 1;
                let progress = (processed * 100 / num_new) as u8;
                while num_new >= 100 && progress >= next_progress {
                    info!("Oracle price computation: {next_progress}%");
                    next_progress += 10;
                }
            };

            if committed < START_HEIGHT_FAST {
                let slow_end = START_HEIGHT_FAST.min(total_heights);
                feed_blocks_with(
                    &mut oracle,
                    indexer,
                    committed..slow_end,
                    None,
                    |_, _, ref_bin| push_ref_bin(ref_bin),
                )?;
                if slow_end == START_HEIGHT_FAST {
                    oracle.reconfigure(Config::default());
                }
            }

            let fast_start = committed.max(START_HEIGHT_FAST);
            if fast_start < total_heights {
                feed_blocks_with(
                    &mut oracle,
                    indexer,
                    fast_start..total_heights,
                    None,
                    |_, _, ref_bin| push_ref_bin(ref_bin),
                )?;
            }
        }

        info!("Computed {num_new} oracle prices.");

        Ok(())
    }
}

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn database(&self) -> Option<&Database> {
        Some(&self.db)
    }

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies { indexer } = dependencies;
        let exit = context.exit();

        self.compute_prices(indexer, exit)?;
        {
            let _lock = exit.lock();
            self.spot.cents.height.write()?;
        }

        Ok(())
    }
}
