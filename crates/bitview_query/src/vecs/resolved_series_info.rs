use bitview_types::{SeriesInfo, SeriesName};

use super::{Vecs, series::Series};

/// Borrowed metadata resolution; materialize the response only when needed.
pub struct ResolvedSeriesInfo<'a> {
    series: &'a Series<'a>,
    value_type: &'static str,
}

impl Vecs<'_> {
    pub(crate) fn resolve_series_info(
        &self,
        series: &SeriesName,
    ) -> Option<ResolvedSeriesInfo<'_>> {
        let normalized = series.normalize();
        let series = self.by_series.get(normalized.as_ref())?;
        let value_type = series.first()?.vec().value_type_to_string();
        Some(ResolvedSeriesInfo { series, value_type })
    }
}

impl ResolvedSeriesInfo<'_> {
    pub fn into_info(self) -> SeriesInfo {
        SeriesInfo {
            description: self.series.description.clone(),
            indexes: self.series.indexes().collect(),
            nullable: self.series.nullable().collect(),
            value_type: self.value_type.into(),
        }
    }
}
