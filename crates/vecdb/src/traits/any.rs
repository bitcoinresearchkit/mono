use crate::{Stamp, Version, read_bounds, vec_region_name};

/// Converts an i64 index to usize, supporting negative indexing.
/// Negative indices count from the end.
pub fn i64_to_usize(i: i64, len: usize) -> usize {
    if i >= 0 {
        (i as usize).min(len)
    } else {
        let v = len as i64 + i;
        if v < 0 { 0 } else { v as usize }
    }
}

/// Common trait for all vectors providing metadata and utility methods.
pub trait AnyVec: Send + Sync {
    fn version(&self) -> Version;
    fn name(&self) -> &str;
    fn len(&self) -> usize;
    /// Whether existing values can be replaced while this vector is updated.
    #[inline]
    fn is_mutable(&self) -> bool {
        false
    }
    /// Length visible inside the current published read snapshot.
    #[inline]
    fn visible_len(&self) -> usize {
        read_bounds::visible_len(self.index_type_to_string(), self.len())
    }
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Returns the string representation of the index type.
    fn index_type_to_string(&self) -> &'static str;
    /// Returns the combined name of the vector.
    #[inline]
    fn region_name(&self) -> String {
        vec_region_name(self.name(), self.index_type_to_string())
    }
    /// Returns the list of region names used by this vector.
    fn region_names(&self) -> Vec<String>;
    /// Returns the size in bytes of the value type.
    fn value_type_to_size_of(&self) -> usize;
    /// Returns the short type name of the value type (e.g., "Sats", "StoredF64").
    fn value_type_to_string(&self) -> &'static str;
    /// Generates an ETag for this vector based on stamp and optional end index.
    fn etag(&self, stamp: Stamp, to: Option<i64>) -> String {
        let len = self.len();
        format!(
            "{}-{}-{}",
            to.map_or(len, |to| {
                if to.is_negative() {
                    len.saturating_sub(to.unsigned_abs() as usize)
                } else {
                    to as usize
                }
            }),
            usize::from(self.version()),
            u64::from(stamp),
        )
    }

    /// Converts an i64 index to usize, supporting negative indexing (Python-style).
    #[inline]
    fn i64_to_usize(&self, i: i64) -> usize {
        let len = self.len();
        i64_to_usize(i, len)
    }
}
