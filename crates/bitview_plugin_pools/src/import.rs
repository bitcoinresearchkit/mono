use std::collections::BTreeMap;

use bitview_collections::Windows;
use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Hashrate, POOL_ATTRIBUTION_VERSION, pools};
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::Height;
use vecdb::{BytesVec, ImportableVec, ReadableCloneableVec, Version};

use crate::{STORAGE, Vecs, pool::PoolVecs, pool_heights::PoolHeights};

impl Vecs {
    /// `network` holds the network hash-rate estimates matching the pool windows (24 hours,
    /// then its 1-week, 1-month and 1-year averages).
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        network: &Windows<&impl ReadableCloneableVec<Height, Hashrate>>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 100_000)?;
        let pools = pools();

        let version =
            STORAGE.schema_version() + POOL_ATTRIBUTION_VERSION + Version::new(pools.len() as u32);

        let pool = BytesVec::import(&db, "pool", version)?;
        let heights = PoolHeights::build(&pool);
        let by_slug = pools
            .iter()
            .map(|pool| {
                let vecs = PoolVecs::new(
                    pool.slug,
                    &heights,
                    version,
                    mappings,
                    window_starts,
                    network,
                );
                (pool.slug, vecs)
            })
            .collect::<BTreeMap<_, _>>();

        let this = Self {
            pool,
            heights,
            by_slug,
            pools,
            db,
        };

        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}
