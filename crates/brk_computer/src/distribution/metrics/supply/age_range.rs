use brk_cohort::{AGE_RANGE_IDS, AgeRangeId, Filter};
use brk_error::Result;
use brk_types::{Height, Sats, Version};
use vecdb::{
    AnyStoredVec, AnyVec, ColumnId, ColumnarVec, Database, EagerVec, ImportableVec, PcoVec,
    ReadOnlyClone, ReadableColumnarVec, Rw, StorageMode, WritableVec,
};

use crate::{
    distribution::metrics::{AllSupplyCache, ImportConfig, SupplyCore},
    internal::LazySpotValuePerBlock,
};

type Source<M> = M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Sats>, AgeRangeId>>>;

pub(crate) struct AgeRangeSupplySources<M: StorageMode = Rw> {
    total: Source<M>,
    in_profit: Source<M>,
    in_loss: Source<M>,
}

impl AgeRangeSupplySources {
    pub(crate) fn forced_import(
        db: &Database,
        prefix: &str,
        version: Version,
    ) -> Result<Self> {
        Ok(Self {
            total: EagerVec::forced_import(
                db,
                &format!("{prefix}_age_range_supply_sats"),
                version,
            )?,
            in_profit: EagerVec::forced_import(
                db,
                &format!("{prefix}_age_range_supply_in_profit_sats"),
                version,
            )?,
            in_loss: EagerVec::forced_import(
                db,
                &format!("{prefix}_age_range_supply_in_loss_sats"),
                version,
            )?,
        })
    }

    pub(crate) fn all(&self, cfg: &ImportConfig) -> (SupplyCore, AllSupplyCache) {
        let total_name = cfg.name("supply");
        let total_source = self.total.read_only_clone().sum_columns(
            &format!("{total_name}_sats"),
            cfg.version,
            AGE_RANGE_IDS,
        );
        let all_supply = AllSupplyCache::new(total_source);
        let total = LazySpotValuePerBlock::from_sats_source(
            &total_name,
            cfg.version,
            all_supply.readable_boxed_clone(),
            cfg.indexes,
            cfg.spot_price,
        );
        let in_profit =
            Self::summed_spot(&self.in_profit, cfg, "supply_in_profit", AGE_RANGE_IDS);
        let in_loss = Self::summed_spot(&self.in_loss, cfg, "supply_in_loss", AGE_RANGE_IDS);

        (
            SupplyCore::from_lazy_all(cfg, total, in_profit, in_loss),
            all_supply,
        )
    }

    pub(crate) fn column(
        &self,
        cfg: &ImportConfig,
        column: AgeRangeId,
        all_supply: &AllSupplyCache,
    ) -> SupplyCore {
        SupplyCore::from_lazy(
            cfg,
            Self::column_spot(&self.total, cfg, "supply", column),
            Self::column_spot(&self.in_profit, cfg, "supply_in_profit", column),
            Self::column_spot(&self.in_loss, cfg, "supply_in_loss", column),
            all_supply,
        )
    }

    pub(crate) fn filtered(
        &self,
        cfg: &ImportConfig,
        filter: &Filter,
        all_supply: &AllSupplyCache,
    ) -> SupplyCore {
        let columns: Vec<_> = AGE_RANGE_IDS
            .into_iter()
            .filter(|column| filter.includes(column.filter()))
            .collect();

        SupplyCore::from_lazy(
            cfg,
            Self::summed_spot(&self.total, cfg, "supply", columns.iter().copied()),
            Self::summed_spot(
                &self.in_profit,
                cfg,
                "supply_in_profit",
                columns.iter().copied(),
            ),
            Self::summed_spot(&self.in_loss, cfg, "supply_in_loss", columns),
            all_supply,
        )
    }

    fn column_spot(
        source: &EagerVec<ColumnarVec<PcoVec<Height, Sats>, AgeRangeId>>,
        cfg: &ImportConfig,
        suffix: &str,
        column: AgeRangeId,
    ) -> LazySpotValuePerBlock {
        let name = cfg.name(suffix);
        let source = source.read_only_clone().column(
            &format!("{name}_sats"),
            cfg.version,
            column,
        );
        LazySpotValuePerBlock::from_sats_source(
            &name,
            cfg.version,
            source,
            cfg.indexes,
            cfg.spot_price,
        )
    }

    fn summed_spot(
        source: &EagerVec<ColumnarVec<PcoVec<Height, Sats>, AgeRangeId>>,
        cfg: &ImportConfig,
        suffix: &str,
        columns: impl IntoIterator<Item = AgeRangeId>,
    ) -> LazySpotValuePerBlock {
        let name = cfg.name(suffix);
        let source = source.read_only_clone().sum_columns(
            &format!("{name}_sats"),
            cfg.version,
            columns,
        );
        LazySpotValuePerBlock::from_sats_source(
            &name,
            cfg.version,
            source,
            cfg.indexes,
            cfg.spot_price,
        )
    }

    #[inline(always)]
    pub(crate) fn push(
        &mut self,
        total: <AgeRangeId as ColumnId>::Row<Sats>,
        in_profit: <AgeRangeId as ColumnId>::Row<Sats>,
        in_loss: <AgeRangeId as ColumnId>::Row<Sats>,
    ) {
        self.total.push(total);
        self.in_profit.push(in_profit);
        self.in_loss.push(in_loss);
    }

    pub(crate) fn stored_vecs_mut(&mut self) -> [&mut dyn AnyStoredVec; 3] {
        [&mut self.total, &mut self.in_profit, &mut self.in_loss]
    }

    pub(crate) fn len(&self) -> usize {
        self.total
            .len()
            .min(self.in_profit.len())
            .min(self.in_loss.len())
    }
}
