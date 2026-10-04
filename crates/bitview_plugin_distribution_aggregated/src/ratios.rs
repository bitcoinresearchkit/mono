use bitview_cohort::AgeAggregateId;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::{Days, PartsPerMillion32, Ratio};
use bitview_transforms::{RatioCents, RatioCentsOrOne};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyWindowStartVec, PerBlock, PercentRollingWindows, RollingWindows, RollingWindowsFrom1w,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use crate::{
    activity::Activity, adjusted_sopr::AdjustedSopr, columns::Columns, realized::Realized,
};
#[derive(Traversable)]
pub struct Ratios<M: StorageMode = Rw> {
    pub adjusted_sopr: AdjustedSopr<M>,
    pub dormancy: RollingWindows<Days, M>,
    pub sopr: PerBlock<Ratio, M>,
    pub sopr_ratio_extended: RollingWindowsFrom1w<Ratio, M>,
    pub sell_side_risk_ratio: PercentRollingWindows<PartsPerMillion32, M>,
    pub profit_to_loss_ratio: RollingWindows<Ratio, M>,
}
impl Ratios {
    pub(crate) fn import(
        db: &Database,
        id: AgeAggregateId,
        v: Version,
        mappings: &Mappings,
        columns: &Columns,
        windows: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            adjusted_sopr: AdjustedSopr::import(db, id, v, columns, mappings, windows)?,
            dormancy: RollingWindows::import(db, &id.metric_name("dormancy"), v, mappings)?,
            sopr: PerBlock::import(db, &id.metric_name("sopr_24h"), v, mappings)?,
            sopr_ratio_extended: RollingWindowsFrom1w::import(
                db,
                &id.metric_name("sopr"),
                v,
                mappings,
            )?,
            sell_side_risk_ratio: PercentRollingWindows::import(
                db,
                &id.metric_name("sell_side_risk_ratio"),
                v,
                mappings,
            )?,
            profit_to_loss_ratio: RollingWindows::import(
                db,
                &id.metric_name("realized_profit_to_loss_ratio"),
                v,
                mappings,
            )?,
        })
    }
    pub(crate) fn compute(
        &mut self,
        from: Height,
        activity: &Activity,
        realized: &Realized,
        exit: &Exit,
    ) -> Result<()> {
        self.adjusted_sopr.compute(from, exit)?;
        self.sopr.compute_binary::<_, _, RatioCentsOrOne>(
            from,
            &activity.transfer_volume.sum._24h.cents.height,
            &realized.value_destroyed.sum._24h.cents.height,
            exit,
        )?;
        for ((target, cdd), volume) in self
            .dormancy
            .as_mut_array()
            .into_iter()
            .zip(activity.coindays_destroyed.sum.as_array())
            .zip(activity.transfer_volume.sum.0.as_array())
        {
            target.height.compute_transform2(
                from,
                &cdd.height,
                &volume.btc.height,
                |(h, cdd, btc, _)| {
                    (
                        h,
                        Days::new(if f64::from(btc) == 0.0 {
                            0.0
                        } else {
                            (f64::from(cdd) / f64::from(btc)) as f32
                        }),
                    )
                },
                exit,
            )?;
        }
        for ((target, created), destroyed) in self
            .sopr_ratio_extended
            .as_mut_array()
            .into_iter()
            .zip(
                activity
                    .transfer_volume
                    .sum
                    .0
                    .as_array()
                    .into_iter()
                    .skip(1),
            )
            .zip(realized.value_destroyed.sum.as_array().into_iter().skip(1))
        {
            target.compute_binary::<_, _, RatioCentsOrOne>(
                from,
                &created.cents.height,
                &destroyed.cents.height,
                exit,
            )?;
        }
        for (target, pnl) in self
            .sell_side_risk_ratio
            .as_mut_array()
            .into_iter()
            .zip(realized.gross_pnl.sum.as_array())
        {
            target.compute_binary::<_, _, RatioCents<PartsPerMillion32>>(
                from,
                &pnl.cents.height,
                &realized.cap.cents.height,
                exit,
            )?;
        }
        for ((target, profit), loss) in self
            .profit_to_loss_ratio
            .as_mut_array()
            .into_iter()
            .zip(realized.profit.sum.as_array())
            .zip(realized.loss.sum.as_array())
        {
            target.compute_binary::<_, _, RatioCentsOrOne>(
                from,
                &profit.cents.height,
                &loss.cents.height,
                exit,
            )?;
        }
        Ok(())
    }
    pub(crate) fn stored_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut result: Vec<&mut dyn AnyStoredVec> = vec![&mut self.sopr.height];
        result.extend(self.adjusted_sopr.stored_vecs_mut());
        result.extend(
            self.dormancy
                .as_mut_array()
                .into_iter()
                .map(|v| &mut v.height as &mut dyn AnyStoredVec),
        );
        result.extend(
            self.sopr_ratio_extended
                .as_mut_array()
                .into_iter()
                .map(|v| &mut v.height as &mut dyn AnyStoredVec),
        );
        result.extend(
            self.sell_side_risk_ratio
                .as_mut_array()
                .into_iter()
                .map(|v| &mut v.ppm.height as &mut dyn AnyStoredVec),
        );
        result.extend(
            self.profit_to_loss_ratio
                .as_mut_array()
                .into_iter()
                .map(|v| &mut v.height as &mut dyn AnyStoredVec),
        );
        result
    }
}
