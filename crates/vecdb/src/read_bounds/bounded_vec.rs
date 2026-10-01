use crate::{AnyExportableVec, Result, Version, i64_to_usize};

use super::{BoundedWriter, ReadBounds};

/// An exported vector read through explicit per-index bounds.
///
/// Every operation reinstalls these bounds for nested lazy inputs, including
/// after a thread handoff. This deliberately does not implement the unbounded
/// vector traits or expose the underlying vector.
#[derive(Clone, Copy)]
pub struct BoundedVec<'a> {
    source: &'a dyn AnyExportableVec,
    bounds: &'a ReadBounds,
    limit: usize,
}

impl<'a> BoundedVec<'a> {
    pub(super) fn new(
        source: &'a dyn AnyExportableVec,
        bounds: &'a ReadBounds,
        limit: usize,
    ) -> Self {
        Self {
            source,
            bounds,
            limit,
        }
    }

    pub fn name(&self) -> &str {
        self.source.name()
    }

    pub fn version(&self) -> Version {
        self.source.version()
    }

    pub fn value_type_to_string(&self) -> &'static str {
        self.source.value_type_to_string()
    }

    pub fn len(&self) -> usize {
        self.bounds.scope(|| self.source.len().min(self.limit))
    }

    fn range(&self, from: Option<usize>, to: Option<usize>) -> (usize, usize) {
        let len = self.len();
        let to = to.unwrap_or(len).min(len);
        (from.unwrap_or(0).min(to), to)
    }

    fn signed_range(&self, from: Option<i64>, to: Option<i64>) -> (usize, usize) {
        let len = self.len();
        let to = to.map(|to| i64_to_usize(to, len)).unwrap_or(len);
        let from = from.map(|from| i64_to_usize(from, len)).unwrap_or(0);
        (from.min(to), to)
    }

    pub fn range_count(&self, from: Option<i64>, to: Option<i64>) -> usize {
        let (from, to) = self.signed_range(from, to);
        to - from
    }

    pub fn range_weight(&self, from: Option<i64>, to: Option<i64>) -> usize {
        self.range_count(from, to)
            .saturating_mul(self.source.value_type_to_size_of())
    }

    #[cfg(feature = "serde")]
    pub fn write_json(
        &self,
        from: Option<usize>,
        to: Option<usize>,
        buf: &mut Vec<u8>,
    ) -> Result<()> {
        let (from, to) = self.range(from, to);
        self.bounds
            .scope(|| self.source.write_json(Some(from), Some(to), buf))
    }

    #[cfg(feature = "serde")]
    pub fn write_json_value_at(&self, index: usize, buf: &mut Vec<u8>) -> Result<()> {
        if index >= self.len() {
            return Ok(());
        }
        self.bounds
            .scope(|| self.source.write_json_value_at(index, buf))
    }

    pub fn write_csv_column(
        &self,
        from: Option<usize>,
        to: Option<usize>,
        buf: &mut String,
    ) -> Result<()> {
        let (from, to) = self.range(from, to);
        self.bounds
            .scope(|| self.source.write_csv_column(Some(from), Some(to), buf))
    }

    pub fn create_writer(&self, from: Option<i64>, to: Option<i64>) -> BoundedWriter<'a> {
        let (from, to) = self.signed_range(from, to);
        let writer = self.bounds.scope(|| {
            self.source
                .create_writer(Some(from as i64), Some(to as i64))
        });
        BoundedWriter::new(writer, self.bounds)
    }
}
