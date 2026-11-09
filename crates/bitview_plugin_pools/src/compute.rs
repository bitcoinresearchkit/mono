use bitview_compute::{prepare_computed, write_chunks};
use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_plugin_indexer::Indexer;
use bitview_primitives::{Index40, TxOutIndex};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Sats};
use rayon::prelude::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use vecdb::{AnyStoredVec, AnyVec, ReadableVec, VecIndex, Version, WritableVec};

use crate::{Dependencies, Vecs, pool_heights::PoolSnapshot};

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let start = context.start().height;
        let Dependencies {
            indexer,
            coinbase_sats,
            coinbase_cents,
            fee_sats,
            fee_cents,
        } = dependencies;
        let exit = context.exit();

        // Blocks and their totals publish together, once both are complete.
        let mut snapshot = self.heights.stage();
        self.compute_pool(indexer, start, exit, &mut snapshot)?;
        self.fill_rewards(
            coinbase_sats,
            coinbase_cents,
            fee_sats,
            fee_cents,
            &mut snapshot,
        );
        self.heights.publish(snapshot);
        Ok(())
    }
}

impl Vecs {
    /// Folds every new block's coinbase and fees into its pool's running totals.
    fn fill_rewards(
        &self,
        coinbase_sats: &impl ReadableVec<Height, Sats>,
        coinbase_cents: &impl ReadableVec<Height, Cents>,
        fee_sats: &impl ReadableVec<Height, Sats>,
        fee_cents: &impl ReadableVec<Height, Cents>,
        snapshot: &mut PoolSnapshot,
    ) {
        let version = coinbase_sats.version()
            + coinbase_cents.version()
            + fee_sats.version()
            + fee_cents.version();
        let to = self
            .block_pool
            .len()
            .min(coinbase_sats.len())
            .min(coinbase_cents.len())
            .min(fee_sats.len())
            .min(fee_cents.len());
        // A shorter source re-fills from its end, so no stale total survives.
        let from = snapshot.resume_totals(version, to);
        let (
            mut slugs,
            mut coinbase_sats_values,
            mut coinbase_cents_values,
            mut fee_sats_values,
            mut fee_cents_values,
        ) = Default::default();
        // Bounded chunks: long refills keep little scratch.
        for chunk in write_chunks(from..to) {
            let (start, end) = (chunk.start, chunk.end);
            self.block_pool
                .collect_range_into_at(start, end, &mut slugs);
            coinbase_sats.collect_range_into_at(start, end, &mut coinbase_sats_values);
            coinbase_cents.collect_range_into_at(start, end, &mut coinbase_cents_values);
            fee_sats.collect_range_into_at(start, end, &mut fee_sats_values);
            fee_cents.collect_range_into_at(start, end, &mut fee_cents_values);
            let blocks = (0..end - start).map(|i| {
                (
                    slugs[i],
                    [
                        u64::from(coinbase_sats_values[i]),
                        u64::from(coinbase_cents_values[i]),
                        u64::from(fee_sats_values[i]),
                        u64::from(fee_cents_values[i]),
                    ],
                )
            });
            snapshot.append_totals(blocks);
        }
    }

    fn compute_pool(
        &mut self,
        indexer: &Indexer,
        start: Height,
        exit: &Exit,
        snapshot: &mut PoolSnapshot,
    ) -> Result<()> {
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
            [&mut self.block_pool],
            dep_version,
            start.to_usize(),
            exit,
        )?;

        let first_txout_index = indexer.vecs().transactions.first_txout_index.reader();
        let output_type = indexer.vecs().outputs.output_type.reader();
        let type_index = indexer.vecs().outputs.type_index.reader();
        let addr_readers = indexer.vecs().addrs.addr_readers();

        let unknown = self.pools.unknown();

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

                // mempool.space's order: the first pool in the list that matches, by one of the
                // coinbase's addresses or by its tag.
                let by_addr = (*out_start..*out_end)
                    .map(TxOutIndex::from)
                    .filter_map(|txout_index| {
                        addr_readers
                            .get(output_type.get(txout_index), type_index.get(txout_index))
                            .and_then(|addr| self.pools.find_from_addr(&addr))
                    })
                    .min_by_key(|pool| pool.slug);
                let by_tag = self.pools.find_from_coinbase_tag(&coinbase_tag.to_string());
                by_addr
                    .into_iter()
                    .chain(by_tag)
                    .min_by_key(|pool| pool.slug)
                    .unwrap_or(unknown)
                    .slug
            })
            .collect::<Vec<_>>();

        for &slug in &pool_slugs {
            self.block_pool.push(slug);
        }

        let _lock = exit.lock();
        self.block_pool.write()?;
        snapshot.update(min, self.block_pool.version(), &pool_slugs);
        Ok(())
    }
}
