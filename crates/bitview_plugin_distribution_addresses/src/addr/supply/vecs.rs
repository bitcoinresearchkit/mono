use bitview_cohort::{AddrTypeId, ByAddrType, WithAddrTypes};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazySpotValuePerBlock, import_cached};
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use derive_more::{Deref, DerefMut};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, AnyVec, Database, ReadableBoxedVec, Rw, StorageMode, WritableVec};

use super::AddrTypeToSupply;

#[derive(Deref, DerefMut, Traversable)]
pub struct AddrSupplyVecs<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub series: WithAddrTypes<LazySpotValuePerBlock>,
    #[traversable(hidden)]
    pub stored: WithAddrTypes<CachedSeries<Height, Sats, M>>,
}

impl AddrSupplyVecs {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let name = format!("{name}_addr_supply");
        let version = version + Version::ONE;
        let stored = WithAddrTypes {
            all: import_cached(db, &format!("{name}_sats"), version)?,
            by_addr_type: ByAddrType::try_from_fn(|id| {
                import_cached(db, &format!("{}_{name}_sats", id.name()), version)
            })?,
        };
        let build = |name: &str, source: &CachedSeries<Height, Sats>| {
            LazySpotValuePerBlock::from_sats_source(name, version, source, mappings, spot_price)
        };
        let series = WithAddrTypes {
            all: build(&name, &stored.all),
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
    pub fn push_supply(&mut self, values: &AddrTypeToSupply) {
        let mut total = Sats::default();
        for (target, &value) in self.stored.by_addr_type.values_mut().zip(values.values()) {
            total += value;
            target.push(value);
        }
        self.stored.all.push(total);
    }
}
