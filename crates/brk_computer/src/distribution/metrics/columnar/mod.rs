mod additive;
mod amount;
mod amount_value;
mod cumulative;
mod exact;
mod rows;

use brk_cohort::{CohortContext, Filter};

pub(crate) use additive::{
    UTXOColumnarMetric, UTXOColumnarMetricWithoutAmount, UTXOColumnarMetricWithoutAmountOrType,
};
pub use amount::ColumnarAmount;
pub use amount_value::ColumnarAmountValue;
pub(crate) use cumulative::{
    CumulativeUTXOColumnarMetric, CumulativeUTXOColumnarMetricWithoutAmountOrType,
    CumulativeUTXOValueColumnarMetric, CumulativeUTXOValueColumnarMetricWithoutAmountOrType,
};
pub(crate) use exact::ExactUTXOColumnarMetric;
pub(crate) use rows::{UTXOAggregateRows, UTXORows};

pub(crate) fn utxo_metric_name(filter: &Filter, cohort_name: &str, metric: &str) -> String {
    let cohort_name = CohortContext::Utxo.full_name(filter, cohort_name);
    if cohort_name.is_empty() {
        metric.to_owned()
    } else {
        format!("{cohort_name}_{metric}")
    }
}
