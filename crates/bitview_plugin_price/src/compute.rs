use bitview_compute::{WRITE_INTERVAL, prepare_computed, write_chunks, write_locked};
use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_plugin_indexer::Indexer;
use bitview_primitives::Float64;
use brk_error::{OptionData, Result};
use brk_exit::Exit;
use brk_oracle::{
    Config, START_HEIGHT_FAST, START_HEIGHT_SLOW, bin_to_cents, cents_to_bin,
    pre_oracle_prices_from,
};
use brk_types::Height;
use tracing::info;
use vecdb::{AnyStoredVec, AnyVec, ReadableVec, VecIndex, WritableVec};

use crate::{Dependencies, Vecs, feed_blocks_with, warm_oracle};

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

        // Saved every WRITE_INTERVAL heights, so a stopped pass resumes from its last save. A
        // chunk lies in one regime, the table before the oracle or one EMA: write_chunks splits
        // at START_HEIGHT_SLOW, a multiple of WRITE_INTERVAL, and segments_for_range at
        // START_HEIGHT_FAST, which isn't.
        const _: () = assert!(START_HEIGHT_SLOW.is_multiple_of(WRITE_INTERVAL));
        let start = self.spot.cents.height.len();
        // The retained oracle continues exactly where the stored prices end; otherwise it is
        // rebuilt from the last stored bin and its warmup blocks (the same state: the EMA is a
        // fold of the window, newest first).
        let mut oracle = self
            .oracle
            .take()
            .and_then(|(next, oracle)| (next == start).then_some(oracle));
        let mut logged = 0;
        for chunk in Config::segments_for_range(start..total_heights).flat_map(write_chunks) {
            if chunk.end <= START_HEIGHT_SLOW {
                for cents in pre_oracle_prices_from(chunk.start).take(chunk.len()) {
                    self.spot.cents.height.push(cents);
                    self.oracle_bin
                        .push(Float64::from(cents_to_bin(f64::from(cents))));
                }
            } else {
                let oracle = match &mut oracle {
                    Some(oracle) => oracle,
                    None => {
                        let ref_bin =
                            f64::from(self.oracle_bin.collect_one_at(chunk.start - 1).data()?);
                        oracle.insert(warm_oracle(indexer, ref_bin, chunk.start, None)?)
                    }
                };
                feed_blocks_with(oracle, indexer, chunk.clone(), None, |_, _, ref_bin| {
                    self.spot.cents.height.push(bin_to_cents(ref_bin));
                    self.oracle_bin.push(Float64::from(ref_bin));
                })?;
                // Slow cold-start EMA up to START_HEIGHT_FAST, then the fast mature-market one.
                if chunk.end == START_HEIGHT_FAST {
                    oracle.reconfigure(Config::default());
                }
                let tenths = (chunk.end - start) * 10 / (total_heights - start);
                if total_heights - start > WRITE_INTERVAL && tenths > logged {
                    logged = tenths;
                    info!("Oracle price computation: {}%", tenths * 10);
                }
            }
            write_locked(
                [
                    &mut self.spot.cents.height as &mut dyn AnyStoredVec,
                    &mut self.oracle_bin,
                ],
                exit,
            )?;
        }
        self.oracle = oracle.map(|oracle| (total_heights, oracle));

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
        self.compute_prices(indexer, context.exit())
    }
}
