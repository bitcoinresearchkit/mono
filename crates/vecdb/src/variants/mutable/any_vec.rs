use crate::{AnyVec, TypedVec, Version};

use super::MutableVec;

impl<V> AnyVec for MutableVec<V>
where
    V: AnyVec + TypedVec,
{
    #[inline]
    fn version(&self) -> Version {
        self.vec.version()
    }

    #[inline]
    fn name(&self) -> &str {
        self.vec.name()
    }

    #[inline]
    fn len(&self) -> usize {
        self.vec.len()
    }

    #[inline]
    fn is_mutable(&self) -> bool {
        true
    }

    #[inline]
    fn index_type_to_string(&self) -> &'static str {
        self.vec.index_type_to_string()
    }

    #[inline]
    fn region_names(&self) -> Vec<String> {
        let mut names = self.vec.region_names();
        if self.has_stored_holes {
            names.push(self.holes_region_name());
        }
        names
    }

    #[inline]
    fn value_type_to_size_of(&self) -> usize {
        self.vec.value_type_to_size_of()
    }

    #[inline]
    fn value_type_to_string(&self) -> &'static str {
        self.vec.value_type_to_string()
    }
}
