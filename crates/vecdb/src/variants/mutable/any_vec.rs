use crate::{AnyVec, TypedVec, Version};

use super::MutableVec;

impl<V> AnyVec for MutableVec<V>
where
    V: TypedVec,
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

    crate::any_vec_type_methods!(V::I, V::T);

    #[inline]
    fn region_names(&self) -> Vec<String> {
        let mut names = self.vec.region_names();
        if self.has_stored_holes {
            names.push(self.holes_region_name());
        }
        names
    }
}
