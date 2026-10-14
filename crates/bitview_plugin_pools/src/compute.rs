use bitview_compute::{prepare_computed, write_chunks};
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mining::Vecs as MiningVecs;
use bitview_primitives::{Index40, TxOutIndex};
use brk_error::Result;
use brk_exit::Exit;
use rayon::prelude::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use vecdb::{AnyStoredVec, AnyVec, Database, ReadableVec, VecIndex, Version, WritableVec};

use crate::{Dependencies, Vecs};

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
        let Dependencies { indexer, mining } = dependencies;
        let exit = context.exit();

        self.compute_pool(indexer, exit)?;
        self.fill_rewards(mining);
        Ok(())
    }
}

impl Vecs {
    /// Folds every new block's coinbase and fees into its pool's running totals.
    fn fill_rewards(&self, mining: &MiningVecs) {
        let rewards = &mining.rewards;
        let coinbase = &rewards.coinbase.block;
        let fees = &rewards.fees.block;
        let version = coinbase.sats.version()
            + coinbase.cents.version()
            + fees.sats.version()
            + fees.cents.version();
        let to = self
            .pool
            .len()
            .min(coinbase.sats.len())
            .min(coinbase.cents.len())
            .min(fees.sats.len())
            .min(fees.cents.len());
        // A shorter source re-fills from its end, so no stale total survives.
        let from = self.heights.resume_totals(version, to);
        let (mut slugs, mut coinbase_sats, mut coinbase_cents, mut fee_sats, mut fee_cents) =
            Default::default();
        // Bounded chunks, each folded under one write lock: long refills keep little scratch.
        for chunk in write_chunks(from..to) {
            let (start, end) = (chunk.start, chunk.end);
            self.pool.collect_range_into_at(start, end, &mut slugs);
            coinbase
                .sats
                .collect_range_into_at(start, end, &mut coinbase_sats);
            coinbase
                .cents
                .collect_range_into_at(start, end, &mut coinbase_cents);
            fees.sats.collect_range_into_at(start, end, &mut fee_sats);
            fees.cents.collect_range_into_at(start, end, &mut fee_cents);
            let blocks = (0..end - start).map(|i| {
                (
                    slugs[i],
                    [
                        u64::from(coinbase_sats[i]),
                        u64::from(coinbase_cents[i]),
                        u64::from(fee_sats[i]),
                        u64::from(fee_cents[i]),
                    ],
                )
            });
            self.heights.append_totals(blocks);
        }
    }

    fn compute_pool(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        let dep_version: Version = [
            indexer.vecs().blocks.coinbase_tag.version(),
            indexer.vecs().transactions.first_tx_index.version(),
            indexer.vecs().transactions.first_txout_index.version(),
            indexer.vecs().outputs.output_type.version(),
            indexer.vecs().outputs.type_index.version(),
            indexer.vecs().addrs.p2pk65.bytes.version(),
            indexer.vecs().addrs.p2pk33.bytes.version(),
            indexer.vecs().addrs.p2pkh.bytes.version(),
            indexer.vecs().addrs.p2sh.bytes.version(),
            indexer.vecs().addrs.p2wpkh.bytes.version(),
            indexer.vecs().addrs.p2wsh.bytes.version(),
            indexer.vecs().addrs.p2tr.bytes.version(),
            indexer.vecs().addrs.p2a.bytes.version(),
        ]
        .into_iter()
        .sum();
        let min = prepare_computed(
            [&mut self.pool],
            dep_version,
            indexer.safe_lengths().height.to_usize(),
            exit,
        )?;

        let first_txout_index = indexer.vecs().transactions.first_txout_index.reader();
        let output_type = indexer.vecs().outputs.output_type.reader();
        let type_index = indexer.vecs().outputs.type_index.reader();
        let addr_readers = indexer.vecs().addrs.addr_readers();

        let unknown = self.pools.get_unknown();

        let len = indexer.vecs().blocks.coinbase_tag.len();
        let coinbase_tags = indexer.vecs().blocks.coinbase_tag.reader();
        let first_tx_indexes = indexer
            .vecs()
            .transactions
            .first_tx_index
            .collect_range_at(min, len);
        let output_len = indexer.vecs().outputs.output_type.len();
        let pool_slugs = first_tx_indexes
            .into_par_iter()
            .enumerate()
            .map(|(offset, tx_index)| {
                let out_start = first_txout_index.get(tx_index).get();
                let out_end = first_txout_index
                    .try_get(tx_index.incremented())
                    .map_or_else(|| TxOutIndex::from(output_len), Index40::get);
                let coinbase_tag = coinbase_tags.get_at(min + offset);

                (*out_start..*out_end)
                    .map(TxOutIndex::from)
                    .find_map(|txout_index| {
                        addr_readers
                            .get(output_type.get(txout_index), type_index.get(txout_index))
                            .and_then(|addr| self.pools.find_from_addr(&addr))
                    })
                    .or_else(|| self.pools.find_from_coinbase_tag(&coinbase_tag.to_string()))
                    .unwrap_or(unknown)
                    .slug
            })
            .collect::<Vec<_>>();

        for &slug in &pool_slugs {
            self.pool.push(slug);
        }

        let _lock = exit.lock();
        self.pool.write()?;
        self.heights.update(min, self.pool.version(), &pool_slugs);
        Ok(())
    }
}
