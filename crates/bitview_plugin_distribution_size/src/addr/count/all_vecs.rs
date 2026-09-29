use bitview_cohort::{AddrTypeId, ByAddrType, WithAddrTypes};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPerBlock, import_cached};
use brk_error::Result;
use brk_types::{Height, StoredU64, Version};
use derive_more::{Deref, DerefMut};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, AnyVec, Database, Ident, Rw, StorageMode, WritableVec};

use super::AddrTypeToAddrCount;

#[derive(Deref, DerefMut, Traversable)]
pub struct AddrCountsVecs<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub series: WithAddrTypes<LazyPerBlock<StoredU64>>,
    #[traversable(hidden)]
    pub stored: WithAddrTypes<CachedSeries<Height, StoredU64, M>>,
}

impl AddrCountsVecs {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &MappingsVecs,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        let stored = WithAddrTypes {
            all: import_cached(db, name, version)?,
            by_addr_type: ByAddrType::try_from_fn(|id| {
                import_cached(db, &format!("{}_{name}", id.name()), version)
            })?,
        };
        let build = |name: &str, source: &CachedSeries<Height, StoredU64>| {
            LazyPerBlock::from_height_source::<Ident>(name, version, source, mappings)
        };
        let series = WithAddrTypes {
            all: build(name, &stored.all),
            by_addr_type: AddrTypeId::series(|id, type_name| {
                build(
                    &format!("{type_name}_{name}"),
                    id.select(&stored.by_addr_type),
                )
            }),
        };
        Ok(Self { series, stored })
    }

    pub fn min_resume_len(&self) -> usize {
        self.stored
            .iter()
            .map(AnyVec::len)
            .min()
            .unwrap_or_default()
    }
    pub fn par_iter_height_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        self.stored
            .iter_mut()
            .map(|v| v as &mut dyn AnyStoredVec)
            .collect::<Vec<_>>()
            .into_par_iter()
    }
    pub fn reset_height(&mut self) -> Result<()> {
        for target in self.stored.iter_mut() {
            target.reset()?;
        }
        Ok(())
    }
    pub fn push_counts(&mut self, values: &AddrTypeToAddrCount) {
        let mut total = StoredU64::default();
        for (target, &value) in self.stored.by_addr_type.values_mut().zip(values.values()) {
            let value = StoredU64::from(value);
            total += value;
            target.push(value);
        }
        self.stored.all.push(total);
    }
}
