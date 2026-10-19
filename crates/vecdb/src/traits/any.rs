use crate::{Version, read_bounds};

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

/// Object-safe metadata every vector implements.
pub trait AnyVec: Send + Sync {
    /// Provenance for consumers: a stored vector's computed version when set,
    /// otherwise its schema version. Appending rows does not change provenance.
    fn version(&self) -> Version;
    fn name(&self) -> &str;
    fn len(&self) -> usize;
    /// Whether existing values can be replaced while this vector is updated. Required: a view
    /// over a mutable source must say so, or its rows would be served as final (HTTP caching).
    fn is_mutable(&self) -> bool;
    /// Length visible inside the current published read snapshot.
    #[inline]
    fn visible_len(&self) -> usize {
        read_bounds::visible_len(self.index_type_name(), self.len())
    }
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn index_type_name(&self) -> &'static str;
    /// Returns the list of region names used by this vector: none for views, which store
    /// nothing of their own.
    #[inline]
    fn region_names(&self) -> Vec<String> {
        Vec::new()
    }
    /// Size in bytes of one value.
    fn value_size(&self) -> usize;
    /// Short name of the value type (e.g. "Sats", "Ratio").
    fn value_type_name(&self) -> &'static str;
    /// [`crate::i64_to_usize`] with this vector's length.
    #[inline]
    fn i64_to_usize(&self, i: i64) -> usize {
        let len = self.len();
        i64_to_usize(i, len)
    }
}

/// Emits the [`AnyVec`] methods fixed by a vector's index and value types (`index_type_name`,
/// `value_size`, `value_type_name`) inside an `impl AnyVec` block.
#[macro_export]
macro_rules! any_vec_type_methods {
    ($index:ty, $value:ty) => {
        #[inline]
        fn index_type_name(&self) -> &'static str {
            <$index as $crate::PrintableIndex>::to_string()
        }

        #[inline]
        fn value_size(&self) -> usize {
            ::std::mem::size_of::<$value>()
        }

        #[inline]
        fn value_type_name(&self) -> &'static str {
            $crate::short_type_name::<$value>()
        }
    };
}
