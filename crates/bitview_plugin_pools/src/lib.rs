use brk_error::Result;

use std::{collections::BTreeMap, path::Path};

use bitview_plugin::{ComputePlugin, Plugin, PluginGate, PluginId};
use bitview_traversable::Traversable;
use brk_indexer::Indexer;
use brk_types::{
    Addr, AddrBytes, Height, OutputType, POOL_ATTRIBUTION_VERSION, PoolSlug, Pools, TxOutIndex,
    pools,
};
use rayon::prelude::*;
use vecdb::{
    AnyStoredVec, AnyVec, BytesVec, Database, Exit, ImportableVec, ReadableVec, Rw, StorageMode,
    VecIndex, Version, WritableVec,
};

mod dependencies;
mod major;
mod minor;
mod pool_heights;

pub use dependencies::Dependencies;
use pool_heights::PoolHeights;

use bitview_compute::{
    CachedWindowStartVec, Windows,
    db_utils::{finalize_db, open_db},
};

pub const ID: PluginId = PluginId::new("pools");
const DB_NAME: &str = ID.as_str();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    plugin_gate: PluginGate,
    db: Database,
    pools: &'static Pools,

    /// Mining pool attributed to each block. BRK first scans address-bearing
    /// outputs of the coinbase transaction for a known pool payout address; if
    /// none matches, it performs case-insensitive substring matching against
    /// known coinbase tags. Unmatched blocks are classified as `unknown`.
    pub pool: M::Stored<BytesVec<Height, PoolSlug>>,
    #[traversable(skip)]
    pub pool_heights: PoolHeights,
    pub major: BTreeMap<PoolSlug, major::Vecs<M>>,
    pub minor: BTreeMap<PoolSlug, minor::Vecs>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Send + Sync,
{
    fn id(&self) -> PluginId {
        ID
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}

impl Vecs {
    pub fn forced_import(
        parent_path: &Path,
        parent_version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let db = open_db(parent_path, DB_NAME, 100_000)?;
        let pools = pools();

        let version = parent_version
            + Version::new(4)
            + POOL_ATTRIBUTION_VERSION
            + Version::new(pools.len() as u32);

        let pool = BytesVec::forced_import(&db, "pool", version)?;
        let pool_heights = PoolHeights::build(&pool);

        let mut major_map = BTreeMap::new();
        let mut minor_map = BTreeMap::new();

        for pool in pools.iter() {
            if pool.slug.is_major() {
                major_map.insert(
                    pool.slug,
                    major::Vecs::forced_import(
                        &db,
                        pool.slug,
                        pool_heights.clone(),
                        version,
                        indexes,
                        cached_starts,
                    )?,
                );
            } else {
                minor_map.insert(
                    pool.slug,
                    minor::Vecs::forced_import(
                        pool.slug,
                        pool_heights.clone(),
                        version,
                        indexes,
                        cached_starts,
                    ),
                );
            }
        }

        let this = Self {
            plugin_gate: Default::default(),
            pool,
            pool_heights,
            major: major_map,
            minor: minor_map,
            pools,
            db,
        };

        finalize_db(&this.db, &this)?;
        Ok(this)
    }

    fn compute_inner(
        &mut self,
        indexer: &Indexer,
        indexes: &bitview_plugin_indexes::Vecs,
        prices: &bitview_plugin_price::Vecs,
        mining: &bitview_plugin_mining::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        self.db.sync_bg_tasks()?;

        self.compute_pool(indexer, indexes, exit)?;

        self.major
            .par_iter_mut()
            .try_for_each(|(_, vecs)| vecs.compute(indexer, prices, mining, exit))?;

        let exit = exit.clone();
        self.db.run_bg(move |db| {
            let _lock = exit.lock();
            db.compact_deferred_default()
        });
        Ok(())
    }

    fn compute_pool(
        &mut self,
        indexer: &Indexer,
        indexes: &bitview_plugin_indexes::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        let dep_version: Version = [
            indexer.vecs().blocks.coinbase_tag.version(),
            indexer.vecs().transactions.first_tx_index.version(),
            indexer.vecs().transactions.first_txout_index.version(),
            indexes.tx_index.output_count.version(),
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
            tracing::warn!(
                "Pool version mismatch: vec_version={pool_vec_version:?} + dep={dep_version:?} = {expected:?}, stored computed={pool_computed:?}, len={}",
                self.pool.len()
            );
        }
        self.pool.validate_computed_version_or_reset(dep_version)?;

        let first_txout_index = indexer.vecs().transactions.first_txout_index.reader();
        let output_type = indexer.vecs().outputs.output_type.reader();
        let type_index = indexer.vecs().outputs.type_index.reader();
        let p2pk65 = indexer.vecs().addrs.p2pk65.bytes.reader();
        let p2pk33 = indexer.vecs().addrs.p2pk33.bytes.reader();
        let p2pkh = indexer.vecs().addrs.p2pkh.bytes.reader();
        let p2sh = indexer.vecs().addrs.p2sh.bytes.reader();
        let p2wpkh = indexer.vecs().addrs.p2wpkh.bytes.reader();
        let p2wsh = indexer.vecs().addrs.p2wsh.bytes.reader();
        let p2tr = indexer.vecs().addrs.p2tr.bytes.reader();
        let p2a = indexer.vecs().addrs.p2a.bytes.reader();

        let unknown = self.pools.get_unknown();

        let min = starting_height.to_usize().min(self.pool.len());

        // Cursors avoid per-height PcoVec page decompression.
        // Heights are sequential, tx_index values derived from them are monotonically
        // increasing, so both cursors only advance forward.
        let mut first_tx_index_cursor = indexer.vecs().transactions.first_tx_index.cursor();
        first_tx_index_cursor.advance(min);
        let mut output_count_cursor = indexes.tx_index.output_count.cursor();

        self.pool.truncate_if_needed_at(min)?;
        self.pool_heights.truncate(min);

        let len = indexer.vecs().blocks.coinbase_tag.len();
        let mut next_height = min;

        indexer.vecs().blocks.coinbase_tag.try_for_each_range_at(
            min,
            len,
            |coinbase_tag| -> Result<()> {
                let tx_index = first_tx_index_cursor.next().unwrap();
                let out_start = first_txout_index.get(tx_index);

                let ti = tx_index.to_usize();
                output_count_cursor.advance(ti - output_count_cursor.position());
                let output_count_val = output_count_cursor.next().unwrap();

                let pool = (*out_start..(*out_start + *output_count_val))
                    .map(TxOutIndex::from)
                    .find_map(|txout_index| {
                        let ot = output_type.get(txout_index);
                        let ti = usize::from(type_index.get(txout_index));
                        match ot {
                            OutputType::P2PK65 => Some(AddrBytes::from(p2pk65.get_at(ti))),
                            OutputType::P2PK33 => Some(AddrBytes::from(p2pk33.get_at(ti))),
                            OutputType::P2PKH => Some(AddrBytes::from(p2pkh.get_at(ti))),
                            OutputType::P2SH => Some(AddrBytes::from(p2sh.get_at(ti))),
                            OutputType::P2WPKH => Some(AddrBytes::from(p2wpkh.get_at(ti))),
                            OutputType::P2WSH => Some(AddrBytes::from(p2wsh.get_at(ti))),
                            OutputType::P2TR => Some(AddrBytes::from(p2tr.get_at(ti))),
                            OutputType::P2A => Some(AddrBytes::from(p2a.get_at(ti))),
                            _ => None,
                        }
                        .map(|bytes| Addr::try_from(&bytes).unwrap())
                        .and_then(|addr| self.pools.find_from_addr(&addr))
                    })
                    .or_else(|| self.pools.find_from_coinbase_tag(&coinbase_tag.as_str()))
                    .unwrap_or(unknown);

                self.pool.push(pool.slug);
                self.pool_heights.push(pool.slug, Height::from(next_height));
                next_height += 1;

                Ok(())
            },
        )?;

        let _lock = exit.lock();
        self.pool.write()?;
        Ok(())
    }
}

impl ComputePlugin for Vecs {
    type Dependencies<'a> = crate::Dependencies<'a>;
    type Output = ();

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        exit: &Exit,
    ) -> Result<Self::Output> {
        self.compute_inner(
            dependencies.indexer,
            dependencies.indexes,
            dependencies.price,
            dependencies.mining,
            exit,
        )
    }
}
