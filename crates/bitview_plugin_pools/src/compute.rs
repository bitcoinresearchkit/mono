use bitview_compute::{prepare_computed, write_chunks};
use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_plugin_blocks::Vecs as BlocksVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mining::Vecs as MiningVecs;
use bitview_plugin_transactions::Vecs as TransactionsVecs;
use bitview_primitives::{Index40, TxOutIndex};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Height;
use rayon::prelude::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use vecdb::{AnyStoredVec, AnyVec, ReadableVec, TypedVec, VecIndex, Version, WritableVec};

use crate::{
    Dependencies, Vecs,
    pool_heights::{Column, PoolSnapshot, TOTALS},
};

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies {
            indexer,
            blocks,
            transactions,
            mining,
            mappings,
        } = dependencies;
        let exit = context.exit();

        // Blocks and their totals publish together, once both are complete.
        let mut snapshot = self.heights.stage();
        self.compute_pool(indexer, exit, &mut snapshot)?;
        self.fill_totals(blocks, transactions, mining, &mut snapshot);
        self.heights.publish(snapshot);
        let from = indexer.compute_start().height;
        crate::concentration::compute(
            &mut self.concentration,
            &mut self.concentration_grouped,
            &self.block_pool,
            mappings,
            from,
            exit,
        )?;
        crate::antpool_group::compute(&mut self.antpool_group, &self.block_pool, from, exit)
    }
}

/// A chain-wide running total, read back as each block's value.
trait Running: AnyVec {
    /// Each block's value from `start` to `end`.
    fn block_values_into(&self, start: usize, end: usize, out: &mut Vec<u64>);
}

impl<V> Running for V
where
    V: ReadableVec<Height, V::T> + TypedVec<I = Height>,
    u64: From<V::T>,
{
    fn block_values_into(&self, start: usize, end: usize, out: &mut Vec<u64>) {
        out.clear();
        // One read from the total through the block before (zero before genesis).
        let mut previous = (start == 0).then_some(0);
        self.for_each_range_at(start.saturating_sub(1), end, |total| {
            let total = u64::from(total);
            if let Some(previous) = previous {
                out.push(total - previous);
            }
            previous = Some(total);
        });
    }
}

impl Vecs {
    /// Folds every new block's value of each chain-wide series in `Totals` into its pool's
    /// running totals.
    fn fill_totals(
        &self,
        blocks: &BlocksVecs,
        transactions: &TransactionsVecs,
        mining: &MiningVecs,
        snapshot: &mut PoolSnapshot,
    ) {
        let (coinbase, fees) = (
            &mining.rewards.coinbase.cumulative,
            &mining.rewards.fees.cumulative,
        );
        let locktime = &blocks.coinbase_locktime;
        // Dynamic dispatch is once per source per chunk, not per block.
        let source = |column| -> &dyn Running {
            match column {
                Column::Blocks => unreachable!("block counts are not summed from a source"),
                Column::CoinbaseSats => &coinbase.sats.height,
                Column::CoinbaseCents => &coinbase.cents.height,
                Column::FeeSats => &fees.sats.height,
                Column::FeeCents => &fees.cents.height,
                Column::EmptyBlocks => &blocks.empty_count.cumulative.height,
                Column::Bip54LocktimeBlocks => &locktime.bip54.cumulative.height,
                Column::NonzeroLocktimeBlocks => &locktime.nonzero.cumulative.height,
                Column::Under0p1SatVbTxs => {
                    &transactions
                        .fees
                        .ranges
                        .tx_count_by_fee_rate
                        .under_0p1_sat_vb
                        .cumulative
                        .height
                }
            }
        };
        let version = Column::SUMMED
            .iter()
            .map(|&column| source(column).version())
            .sum();
        let to = Column::SUMMED
            .iter()
            .fold(self.block_pool.len(), |to, &column| {
                to.min(source(column).len())
            });
        // A shorter source re-fills from its end, so no stale total survives.
        let from = snapshot.resume_totals(version, to);
        let mut slugs = Vec::new();
        let mut columns: [Vec<u64>; TOTALS] = Default::default();
        // Bounded chunks: long refills keep little scratch.
        for chunk in write_chunks(from..to) {
            let (start, end) = (chunk.start, chunk.end);
            self.block_pool
                .collect_range_into_at(start, end, &mut slugs);
            // The column selects the reader's slot; a missing or duplicate entry leaves one
            // empty and indexing it below panics.
            for column in Column::SUMMED {
                source(column).block_values_into(start, end, &mut columns[column as usize - 1]);
            }
            let blocks = (0..end - start)
                .map(|i| (slugs[i], std::array::from_fn(|column| columns[column][i])));
            snapshot.append_totals(blocks);
        }
    }

    fn compute_pool(
        &mut self,
        indexer: &Indexer,
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
            indexer.compute_start().height.to_usize(),
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
