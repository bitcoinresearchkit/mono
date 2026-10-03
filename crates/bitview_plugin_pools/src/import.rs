use std::collections::BTreeMap;

use bitview_collections::Windows;
use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{POOL_ATTRIBUTION_VERSION, pools};
use vecdb::{BytesVec, ImportableVec, Version};

use crate::{STORAGE, Vecs, major, minor, pool_heights::PoolHeights};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
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
                        window_starts,
                    )?,
                );
            } else {
                minor_map.insert(
                    pool.slug,
                    minor::Vecs::new(
                        pool.slug,
                        pool_heights.clone(),
                        version,
                        mappings,
                        window_starts,
                    ),
                );
            }
        }

        let this = Self {
            pool,
            heights: pool_heights,
            major: major_map,
            minor: minor_map,
            pools,
            db,
        };

        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}
