use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mining::Vecs as MiningVecs;
use bitview_primitives::{Index40, TxOutIndex};
use brk_error::Result;
use brk_exit::Exit;
use rayon::prelude::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use tracing::warn;
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
        let version = rewards.coinbase.block.sats.version()
            + rewards.coinbase.block.cents.version()
            + rewards.fees.block.sats.version()
            + rewards.fees.block.cents.version();
        let to = [
            self.pool.len(),
            rewards.coinbase.block.sats.len(),
            rewards.coinbase.block.cents.len(),
            rewards.fees.block.sats.len(),
            rewards.fees.block.cents.len(),
        ]
        .into_iter()
        .min()
        .unwrap_or_default();
        // A shorter source re-fills from its end, so no stale total survives.
        let from = self.heights.totals_start(version).min(to);
        let columns = [
            rewards
                .coinbase
                .block
                .sats
                .collect_range_at(from, to)
                .into_iter()
                .map(u64::from)
                .collect::<Vec<_>>(),
            rewards
                .coinbase
                .block
                .cents
                .collect_range_at(from, to)
                .into_iter()
                .map(u64::from)
                .collect(),
            rewards
                .fees
                .block
                .sats
                .collect_range_at(from, to)
                .into_iter()
                .map(u64::from)
                .collect(),
            rewards
                .fees
                .block
                .cents
                .collect_range_at(from, to)
                .into_iter()
                .map(u64::from)
                .collect(),
        ];
        let len = columns.iter().map(Vec::len).min().unwrap_or_default();
        let blocks = (0..len)
            .map(|index| columns.each_ref().map(|column| column[index]))
            .collect::<Vec<_>>();
        self.heights.fill_totals(from, &blocks, version);
    }

    fn compute_pool(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

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
        let pool_vec_version = self.pool.header().vec_version();
        let pool_computed = self.pool.header().computed_version();
        let expected = pool_vec_version + dep_version;
        if expected != pool_computed {
            warn!(
                "Pool version mismatch: vec_version={pool_vec_version:?} + dep={dep_version:?} = {expected:?}, stored computed={pool_computed:?}, len={}",
                self.pool.len()
            );
        }
        {
            let _lock = exit.lock();
            self.pool.validate_computed_version_or_reset(dep_version)?;
        }

        let first_txout_index = indexer.vecs().transactions.first_txout_index.reader();
        let output_type = indexer.vecs().outputs.output_type.reader();
        let type_index = indexer.vecs().outputs.type_index.reader();
        let addr_readers = indexer.vecs().addrs.addr_readers();

        let unknown = self.pools.get_unknown();

        let min = starting_height.to_usize().min(self.pool.len());

        {
            let _lock = exit.lock();
            self.pool.truncate_if_needed_at(min)?;
        }

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
                    .or_else(|| self.pools.find_from_coinbase_tag(&coinbase_tag.as_str()))
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
