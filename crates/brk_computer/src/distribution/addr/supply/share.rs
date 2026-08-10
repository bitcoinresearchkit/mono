use brk_cohort::{AddrTypeId, ByAddrType};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Height, PartsPerMillion32, Sats, Version};
use vecdb::{
    AnyStoredVec, BinaryTransform, Database, Exit, ReadOnlyClone, ReadableVec, Rw, StorageMode,
    WritableVec,
};

use crate::{
    distribution::metrics::AllSupplyCache,
    indexes,
    internal::{ColumnarPerBlock, LazyColumnPercentPerBlock, LazyPercentPerBlock, RatioSats},
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
    pub by_addr_type: ByAddrType<LazyColumnPercentPerBlock<PartsPerMillion32, AddrTypeId>>,
    #[traversable(hidden)]
    ppm: ColumnarPerBlock<PartsPerMillion32, AddrTypeId, (), M>,
}

impl AddrSupplyShareVecs {
    pub(crate) fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
        supply: &AddrSupplyVecs,
        all_supply: &AllSupplyCache,
    ) -> Result<Self> {
        let name = format!("{name}_addr_supply_share");
        let all = LazyPercentPerBlock::from_cached_ratio::<Sats, Sats, RatioSats<PartsPerMillion32>>(
            &name,
            version,
            &supply.all.sats.height,
            all_supply.cached_boxed_clone(),
            indexes,
        );
        let ppm =
            ColumnarPerBlock::forced_import(db, &format!("{name}_ppm_by_type"), version, |_| ())?;
        let source = ppm.height.read_only_clone();
        let by_addr_type = AddrTypeId::series(|column, type_name| {
            LazyColumnPercentPerBlock::new(
                &format!("{type_name}_{name}"),
                version,
                &source,
                column,
                indexes,
            )
        });

        Ok(Self {
            all,
            by_addr_type,
            ppm,
        })
    }

    pub(crate) fn reset_height(&mut self) -> Result<()> {
        self.ppm.height.reset()?;
        Ok(())
    }

    pub(crate) fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        self.ppm.stored_mut()
    }

    pub(crate) fn compute_rest(
        &mut self,
        max_from: Height,
        supply: &AddrSupplyVecs,
        type_supply_sats: &ByAddrType<&impl ReadableVec<Height, Sats>>,
        exit: &Exit,
    ) -> Result<()> {
        self.ppm.compute_matrix_columns2(
            max_from,
            &supply.height,
            |column| *column.select(type_supply_sats),
            |_, category, total| RatioSats::<PartsPerMillion32>::apply(category, total),
            exit,
        )
    }
}
