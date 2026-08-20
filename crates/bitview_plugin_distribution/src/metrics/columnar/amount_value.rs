use brk_error::Result;

use bitview_cohort::{Amount, AmountRange, AmountRangeId, CohortContext, Filter};
use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Sats, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode};

use bitview_compute::ColumnarValuePerBlockCumulativeRolling;

#[derive(Deref, DerefMut, Traversable)]
pub struct ColumnarAmountValue<S: Clone, M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub series: Amount<S>,
    pub values: ColumnarValuePerBlockCumulativeRolling<AmountRangeId, (), M>,
}

impl<S: Clone> ColumnarAmountValue<S> {
    pub fn forced_import(
        db: &Database,
        matrix_name: &str,
        context: CohortContext,
        metric: &str,
        version: Version,
        mut build: impl FnMut(
            &str,
            ReadableBoxedVec<Height, Sats>,
            ReadableBoxedVec<Height, Cents>,
        ) -> S,
    ) -> Result<Self> {
        let values = ColumnarValuePerBlockCumulativeRolling::forced_import(
            db,
            matrix_name,
            version,
            |_, _| (),
        )?;

        let series = Amount::new(|filter, cohort_name| {
            let name = Self::metric_name(context, &filter, cohort_name, metric);
            let amounts = AmountRangeId::matching(&filter);
            let (sats, cents) = match amounts {
                Some(amount) => values.sources(&format!("{name}_cumulative"), version, [amount]),
                None => values.sources(
                    &format!("{name}_cumulative"),
                    version,
                    AmountRangeId::included_by(&filter),
                ),
            };
            build(&name, sats, cents)
        });

        Ok(Self { series, values })
    }

    fn metric_name(
        context: CohortContext,
        filter: &Filter,
        cohort_name: &str,
        metric: &str,
    ) -> String {
        context.metric_name(filter, cohort_name, metric)
    }

    #[inline(always)]
    pub fn push_cumulative(&mut self, sats: &AmountRange<Sats>, cents: &AmountRange<Cents>) {
        self.values.push_block(sats.clone(), cents.clone());
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.values.collect_vecs_mut()
    }
}
