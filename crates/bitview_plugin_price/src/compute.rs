use bitview_compute::{prepare_computed, write_locked};
use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_plugin_indexer::Indexer;
use bitview_primitives::Float64;
use brk_error::{OptionData, Result};
use brk_exit::Exit;
use brk_oracle::{
    Config, Oracle, START_HEIGHT_FAST, START_HEIGHT_SLOW, bin_to_cents, cents_to_bin,
    pre_oracle_prices_from,
};
use brk_types::Height;
use tracing::info;
use vecdb::{AnyStoredVec, AnyVec, ReadableVec, VecIndex, WritableVec};

use crate::{Dependencies, Vecs, feed_blocks_for_warmup, feed_blocks_with};

impl Vecs {
    fn compute_prices(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        let starting_height = indexer.compute_start().height;

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
        let committed = prepare_computed(
            [
                &mut self.spot.cents.height as &mut dyn AnyStoredVec,
                &mut self.oracle_bin,
            ],
            source_version,
            starting_height.to_usize(),
            exit,
        )?;
        // The slow regime's shape profile is not stored, so resuming inside that regime
        // replays it from its start, once readers have left the rows it rewrites.
        if (START_HEIGHT_SLOW + 1..START_HEIGHT_FAST).contains(&committed) {
            indexer.rewrite_from(Height::from(START_HEIGHT_SLOW), || {
                let _lock = exit.lock();
                self.spot
                    .cents
                    .height
                    .truncate_if_needed_at(START_HEIGHT_SLOW)?;
                self.oracle_bin.truncate_if_needed_at(START_HEIGHT_SLOW)?;
                Ok(())
            })?;
        }

        let seed_len = total_heights.min(START_HEIGHT_SLOW);
        let seed_start = self.spot.cents.height.len();
        for cents in pre_oracle_prices_from(seed_start).take(seed_len.saturating_sub(seed_start)) {
            self.spot.cents.height.push(cents);
            self.oracle_bin
                .push(Float64::from(cents_to_bin(f64::from(cents))));
        }

        if self.spot.cents.height.len() >= total_heights {
            return Ok(());
        }

        let committed = self.spot.cents.height.len();
        // The retained oracle continues exactly where the stored prices end; otherwise rebuild it
        // from the last stored bin and its warmup blocks (the same state: the EMA is a fold of
        // the window, newest first).
        let mut oracle = match self.oracle.take() {
            Some((next, oracle)) if next == committed => oracle,
            _ => {
                let config = Config::for_height(committed);
                let ref_bin = f64::from(self.oracle_bin.collect_one_at(committed - 1).data()?);
                let warmup = config.window_size.min(committed - START_HEIGHT_SLOW);
                Oracle::from_checkpoint(ref_bin, config, |o| {
                    feed_blocks_for_warmup(o, indexer, (committed - warmup)..committed, None)
                })?
            }
        };

        let num_new = total_heights - committed;

        // Slow cold-start EMA up to START_HEIGHT_FAST, then switch to the fast
        // mature-market EMA. Steady-state runs start past START_HEIGHT_FAST and skip
        // the slow segment entirely.
        {
            let mut processed = 0usize;
            let mut next_progress = 10u8;
            let mut push_ref_bin = |ref_bin| {
                self.spot.cents.height.push(bin_to_cents(ref_bin));
                self.oracle_bin.push(Float64::from(ref_bin));

                processed += 1;
                let progress = (processed * 100 / num_new) as u8;
                while num_new >= 100 && progress >= next_progress {
                    info!("Oracle price computation: {next_progress}%");
                    next_progress += 10;
                }
            };

            for segment in Config::segments_for_range(committed..total_heights) {
                let end = segment.end;
                feed_blocks_with(&mut oracle, indexer, segment, None, |_, _, ref_bin| {
                    push_ref_bin(ref_bin)
                })?;
                if end == START_HEIGHT_FAST {
                    oracle.reconfigure(Config::default());
                }
            }
        }

        self.oracle = Some((total_heights, oracle));

        Ok(())
    }
}

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies { indexer } = dependencies;
        let exit = context.exit();

        self.compute_prices(indexer, exit)?;
        write_locked(
            [
                &mut self.spot.cents.height as &mut dyn AnyStoredVec,
                &mut self.oracle_bin,
            ],
            exit,
        )
    }
}
