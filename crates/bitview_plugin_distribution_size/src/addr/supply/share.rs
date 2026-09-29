use bitview_cohort::{AddrTypeId, ByAddrType};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::RatioSats;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPercentPerBlock, import_cached};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, PartsPerMillion32, Sats, Version};
use vecdb::{
    AnyStoredVec, BinaryTransform, Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode,
    WritableVec,
};

use super::vecs::AddrSupplyVecs;

/// Share of a predicate-based supply category relative to total supply.
///
/// - `all`: category supply / circulating supply
/// - Per-type: type's category supply / type's total supply
#[derive(Traversable)]
pub struct AddrSupplyShareVecs<M: StorageMode = Rw> {
    pub all: LazyPercentPerBlock<PartsPerMillion32>,
    #[traversable(flatten)]
    pub by_addr_type: ByAddrType<LazyPercentPerBlock<PartsPerMillion32>>,
    #[traversable(hidden)]
    ppm: ByAddrType<CachedSeries<Height, PartsPerMillion32, M>>,
}

impl AddrSupplyShareVecs {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &MappingsVecs,
        supply: &AddrSupplyVecs,
        all_supply: &impl ReadableCloneableVec<Height, Sats>,
    ) -> Result<Self> {
        let name = format!("{name}_addr_supply_share");
        let all = LazyPercentPerBlock::from_ratio::<Sats, Sats, RatioSats<PartsPerMillion32>>(
            &name,
            version,
            &supply.all.sats.height,
            all_supply,
            mappings,
        );
        let ppm = ByAddrType::try_from_fn(|id| {
            import_cached(
                db,
                &format!("{}_{name}_ppm", id.name()),
                version + Version::ONE,
            )
        })?;
        let by_addr_type = AddrTypeId::series(|id, type_name| {
            LazyPercentPerBlock::from_height_source(
                &format!("{type_name}_{name}"),
                version,
                id.select(&ppm),
                mappings,
            )
        });

        Ok(Self {
            all,
            by_addr_type,
            ppm,
        })
    }

    pub fn reset_height(&mut self) -> Result<()> {
        for (_, target) in self.ppm.iter_mut() {
            target.reset()?;
        }
        Ok(())
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.ppm.iter_mut().map(|(_, v)| v as &mut dyn AnyStoredVec)
    }

    pub fn compute_rest(
        &mut self,
        max_from: Height,
        supply: &AddrSupplyVecs,
        type_supply_sats: &ByAddrType<&impl ReadableVec<Height, Sats>>,
        exit: &Exit,
    ) -> Result<()> {
        for &id in AddrTypeId::ALL {
            id.select_mut(&mut self.ppm).compute_transform2(
                max_from,
                &id.select(&supply.series.by_addr_type).sats.height,
                *id.select(type_supply_sats),
                |(height, category, total, _)| {
                    (
                        height,
                        RatioSats::<PartsPerMillion32>::apply(category, total),
                    )
                },
                exit,
            )?;
        }
        Ok(())
    }
}
