use bitview_types::{SeriesInfo, SeriesName};

use super::{Vecs, index_to_vec::IndexToVec};

/// Borrowed metadata resolution; materialize the response only when needed.
pub struct ResolvedSeriesInfo<'a> {
    vecs: &'a Vecs<'a>,
    name: &'a str,
    indexes: &'a IndexToVec<'a>,
    value_type: &'static str,
}

impl Vecs<'_> {
    pub(crate) fn resolve_series_info(
        &self,
        series: &SeriesName,
    ) -> Option<ResolvedSeriesInfo<'_>> {
        let normalized = series.normalize();
        let (&name, index_to_vec) = self.by_series.get_key_value(normalized.as_ref())?;
        let value_type = index_to_vec.first()?.vec().value_type_to_string();
        Some(ResolvedSeriesInfo {
            vecs: self,
            name,
            indexes: index_to_vec,
            value_type,
        })
    }
}

impl ResolvedSeriesInfo<'_> {
    pub fn into_info(self) -> SeriesInfo {
        SeriesInfo {
            description: self
                .vecs
                .series_position(self.name)
                .and_then(|index| self.vecs.description_search.description(index)),
            indexes: self.indexes.indexes().collect(),
            value_type: self.value_type.into(),
        }
    }
}
