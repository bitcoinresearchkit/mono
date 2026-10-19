use std::convert::Infallible;

use bitview_primitives::{
    PercentileId, RarityPercentileId, percentile_table, rarity_percentile_table,
};
use bitview_traversable::Traversable;

macro_rules! percentile_collection {
    (
        [$name:ident] $id:ident, $values:ident: [$ty:ty; $len:ident];
        $($variant:ident => $field:ident, $value:expr, $doc:tt;)*
    ) => {
        #[derive(Clone, Traversable)]
        pub struct $name<T> {
            $(#[doc = $doc] pub $field: T,)*
        }

        impl<T> $name<T> {
            pub fn try_from_fn<E>(mut f: impl FnMut($id) -> Result<T, E>) -> Result<Self, E> {
                Ok(Self {
                    $($field: f($id::$variant)?,)*
                })
            }

            pub fn from_fn(mut f: impl FnMut($id) -> T) -> Self {
                let Ok(this) = Self::try_from_fn(|id| Ok::<_, Infallible>(f(id)));
                this
            }

            pub fn get(&self, id: $id) -> &T {
                match id {
                    $($id::$variant => &self.$field,)*
                }
            }

            pub fn iter(&self) -> impl Iterator<Item = &T> {
                [$(&self.$field),*].into_iter()
            }

            pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
                [$(&mut self.$field),*].into_iter()
            }
        }
    };
}

percentile_table!(percentile_collection, ByPercentile);
rarity_percentile_table!(percentile_collection, RarityPercentiles);

impl<T> RarityPercentiles<T> {
    pub fn boundary_refs(&self) -> [&T; 10] {
        RarityPercentileId::BOUNDARIES.map(|id| self.get(id))
    }
}
