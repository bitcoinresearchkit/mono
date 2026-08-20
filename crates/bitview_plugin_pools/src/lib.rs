use brk_error::Result;

use std::collections::BTreeMap;

use bitview_plugin::{
    ComputePlugin, ImportContext, Plugin, PluginGate, PluginId, PluginStorage, UpdateContext,
};
use bitview_plugin_indexer::Indexer;
use bitview_traversable::Traversable;
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
mod has;
mod major;
mod minor;
mod pool_heights;

pub use dependencies::Dependencies;
pub use has::HasPools;
use pool_heights::PoolHeights;

use bitview_compute::{CachedWindowStartVec, Windows};

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("pools"), Version::new(13));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    plugin_gate: PluginGate,
    #[traversable(skip)]
    db: Database,
    #[traversable(skip)]
    pools: &'static Pools,

    /// Mining pool attributed to each block. BRK first scans address-bearing
    /// outputs of the coinbase transaction for a known pool payout address; if
    /// none matches, it performs case-insensitive substring matching against
    /// known coinbase tags. Unmatched blocks are classified as `unknown`.
    pub pool: M::Stored<BytesVec<Height, PoolSlug>>,
    #[traversable(skip)]
    pub heights: PoolHeights,
    pub major: BTreeMap<PoolSlug, major::Vecs<M>>,
    pub minor: BTreeMap<PoolSlug, minor::Vecs>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 100_000)?;
        let pools = pools();

        let version =
            STORAGE.schema_version() + POOL_ATTRIBUTION_VERSION + Version::new(pools.len() as u32);

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
                        mappings,
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
                        mappings,
                        cached_starts,
                    ),
                );
            }
        }

        let this = Self {
            plugin_gate: Default::default(),
            pool,
            heights: pool_heights,
            major: major_map,
            minor: minor_map,
            pools,
            db,
        };

        STORAGE.finalize_database(&this.db, &this)?;
        Ok(this)
    }

    fn compute_inner(
        &mut self,
        indexer: &Indexer,
        mappings: &bitview_plugin_mappings::Vecs,
        prices: &bitview_plugin_price::Vecs,
        mining: &bitview_plugin_mining::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        self.db.sync_bg_tasks()?;

        self.compute_pool(indexer, mappings, exit)?;

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
        mappings: &bitview_plugin_mappings::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        let dep_version: Version = [
            indexer.vecs().blocks.coinbase_tag.version(),
            indexer.vecs().transactions.first_tx_index.version(),
            indexer.vecs().transactions.first_txout_index.version(),
            mappings.tx_index.output_count.version(),
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
        let mut output_count_cursor = mappings.tx_index.output_count.cursor();

        self.pool.truncate_if_needed_at(min)?;
        self.heights.truncate(min);

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
                self.heights.push(pool.slug, Height::from(next_height));
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
    type Dependencies<'a> = Dependencies<'a>;
    type Output = ();

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<Self::Output> {
        self.compute_inner(
            dependencies.indexer,
            dependencies.mappings,
            dependencies.price,
            dependencies.mining,
            context.exit(),
        )
    }
}
